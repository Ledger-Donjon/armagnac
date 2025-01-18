use crate::{
    arith::ArithError,
    instructions::{self, Instruction, InstructionSize},
    it_state::ItState,
};

#[derive(Debug)]
pub enum InstructionDecodeError {
    /// Instruction is unknown.
    Unknown,
    /// Instruction is undefined, which should trigger a Usage Fault.
    Undefined,
    /// Instruction has been matched but some bits don't have the correct value, making the
    /// instruction effect unpredictable as indicated in the ARM specification.
    Unpredictable,
}

pub enum InstructionPatternBit {
    /// Tested bit must be zero to match instruction code.
    OpcodeZero,
    /// Tested bit must be one to match instruction code.
    OpcodeOne,
    /// Tested bit is part of instruction arguments, and can be either zero or one.
    Arg,
    /// Tested bit is part of instruction arguments, but is expected to be zero.
    ArgZero,
    /// Tested bit is part of instruction arguments, but is expected to be one.
    ArgOne,
}

pub enum DecodeError {
    Other,
    Unpredictable,
    Undefined,
}

impl From<ArithError> for DecodeError {
    fn from(value: ArithError) -> Self {
        match value {
            ArithError::Unpredictable => DecodeError::Unpredictable,
        }
    }
}

type InstructionDecodingFunction =
    fn(usize, u32, ItState) -> Result<Box<dyn Instruction>, DecodeError>;

struct InstructionPattern(Vec<InstructionPatternBit>);

impl InstructionPattern {
    pub fn new(pattern: &str) -> Self {
        // Parse pattern string expression to build a simpler pattern vector with one element per
        // bit.
        let mut bits = Vec::new();
        let mut parenthesis = 0;
        for c in pattern.chars() {
            match parenthesis {
                0 => match c {
                    '0' => bits.push(InstructionPatternBit::OpcodeZero),
                    '1' => bits.push(InstructionPatternBit::OpcodeOne),
                    'x' => bits.push(InstructionPatternBit::Arg),
                    '(' => parenthesis = 1,
                    _ => panic!(),
                },
                1 => {
                    match c {
                        '0' => bits.push(InstructionPatternBit::ArgZero),
                        '1' => bits.push(InstructionPatternBit::ArgOne),
                        _ => panic!(),
                    }
                    parenthesis = 2
                }
                2 => {
                    assert_eq!(c, ')');
                    parenthesis = 0
                }
                _ => panic!(),
            }
        }
        assert_eq!(parenthesis, 0);
        assert!(bits.len() == 16 || bits.len() == 32);
        Self(bits)
    }

    /// Returns true if given instruction matches the pattern.
    ///
    /// # Arguments
    ///
    /// * `ins` - Instruction
    pub fn test(&self, ins: u32, size: InstructionSize) -> Result<bool, InstructionDecodeError> {
        if size != self.size() {
            return Ok(false);
        }
        let mut x = ins;
        let mut unpredictable = false;
        for p in self.0.iter().rev() {
            let bit = x & 1;
            match p {
                InstructionPatternBit::OpcodeZero => {
                    if bit != 0 {
                        return Ok(false);
                    }
                }
                InstructionPatternBit::OpcodeOne => {
                    if bit != 1 {
                        return Ok(false);
                    }
                }
                InstructionPatternBit::ArgZero => {
                    if bit != 0 {
                        unpredictable = true;
                    }
                }
                InstructionPatternBit::ArgOne => {
                    if bit != 1 {
                        unpredictable = true;
                    }
                }
                InstructionPatternBit::Arg => {}
            }
            x >>= 1
        }
        assert_eq!(x, 0);
        if unpredictable {
            Err(InstructionDecodeError::Unpredictable)
        } else {
            Ok(true)
        }
    }

    pub fn size(&self) -> InstructionSize {
        match self.0.len() {
            16 => InstructionSize::Ins16,
            32 => InstructionSize::Ins32,
            _ => panic!(),
        }
    }
}

struct DecoderEntry {
    /// All possible patterns which can match for the given instruction.
    /// Pattern number of index 0 corresponds to T1 encoding, pattern of index 1 to T2 encoding,
    /// etc.
    patterns: Vec<InstructionPattern>,
    /// Decoding function of the instruction
    decoder: InstructionDecodingFunction,
}

struct InstructionDecoder {
    entries: Vec<DecoderEntry>,
}

fn box_decoder<T: 'static + Instruction>(
    tn: usize,
    ins: u32,
    state: ItState,
) -> Result<Box<dyn 'static + Instruction>, DecodeError> {
    match T::try_decode(tn, ins, state) {
        Ok(x) => Ok(Box::new(x)),
        Err(e) => Err(e),
    }
}

impl InstructionDecoder {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Registers a new instruction type to the decoder.
    ///
    /// The decoder will fetch the corresponding instruction patterns and the decoding function to
    /// be called when a pattern matches.
    pub fn insert<T: 'static + Instruction>(&mut self) {
        self.entries.push(DecoderEntry {
            patterns: T::patterns()
                .iter()
                .map(|p| InstructionPattern::new(p))
                .collect(),
            decoder: box_decoder::<T>,
        });
    }

    /// Try to decode an [u32] into an [Instruction].
    ///
    /// # Arguments
    ///
    /// * `ins` - Opcode
    /// * `size` - Instruction size (16 bits or 32 bits)
    /// * `state` - Current processor IT block state, required for correct instruction decoding
    ///     (some encoding may lead to UNPREDICTABLE depending on the IT state).
    pub fn try_decode(
        &self,
        ins: u32,
        size: InstructionSize,
        state: ItState,
    ) -> Result<Box<dyn Instruction>, InstructionDecodeError> {
        for entry in &self.entries {
            for (i, pattern) in entry.patterns.iter().enumerate() {
                if pattern.test(ins, size.clone())? {
                    if let Ok(ins) = (entry.decoder)(i + 1, ins, state) {
                        return Ok(ins);
                    }
                }
            }
        }
        Err(InstructionDecodeError::Unknown)
    }
}

pub struct ArmV7InstructionDecoder(InstructionDecoder);

impl ArmV7InstructionDecoder {
    pub fn new() -> Self {
        let mut dec = InstructionDecoder::new();
        dec.insert::<instructions::add::AddImm>();
        dec.insert::<instructions::add::AddReg>();
        dec.insert::<instructions::add::AddSpPlusImm>();
        dec.insert::<instructions::add::AddSpPlusReg>();
        dec.insert::<instructions::and::AndImm>();
        dec.insert::<instructions::and::AndReg>();
        dec.insert::<instructions::asr::AsrImm>();
        dec.insert::<instructions::asr::AsrReg>();
        dec.insert::<instructions::b::B>();
        dec.insert::<instructions::bic::BicImm>();
        dec.insert::<instructions::bic::BicReg>();
        dec.insert::<instructions::bl::Bl>();
        dec.insert::<instructions::blx::Blx>();
        dec.insert::<instructions::bx::Bx>();
        dec.insert::<instructions::cbnz::Cbnz>();
        dec.insert::<instructions::clz::Clz>();
        dec.insert::<instructions::cmp::CmpImm>();
        dec.insert::<instructions::cmp::CmpReg>();
        dec.insert::<instructions::cps::Cps>();
        dec.insert::<instructions::dsb::Dsb>();
        dec.insert::<instructions::eor::EorImm>();
        dec.insert::<instructions::eor::EorReg>();
        dec.insert::<instructions::isb::Isb>();
        dec.insert::<instructions::it::It>();
        dec.insert::<instructions::ldm::Ldm>();
        dec.insert::<instructions::ldr::LdrImm>();
        dec.insert::<instructions::ldr::LdrImm>();
        dec.insert::<instructions::ldr::LdrLit>();
        dec.insert::<instructions::ldr::LdrReg>();
        dec.insert::<instructions::ldrb::LdrbImm>();
        dec.insert::<instructions::ldrb::LdrbReg>();
        dec.insert::<instructions::ldrd::LdrdImm>();
        dec.insert::<instructions::ldrd::LdrdLit>();
        dec.insert::<instructions::ldrh::LdrhImm>();
        dec.insert::<instructions::ldrh::LdrhLit>();
        dec.insert::<instructions::ldrh::LdrhReg>();
        dec.insert::<instructions::ldrh::Ldrht>();
        dec.insert::<instructions::lsl::LslImm>();
        dec.insert::<instructions::lsl::LslReg>();
        dec.insert::<instructions::lsr::LsrImm>();
        dec.insert::<instructions::lsr::LsrReg>();
        dec.insert::<instructions::mla::Mla>();
        dec.insert::<instructions::mov::MovImm>();
        dec.insert::<instructions::mov::MovReg>();
        dec.insert::<instructions::mrs::Mrs>();
        dec.insert::<instructions::msr::Msr>();
        dec.insert::<instructions::mul::Mul>();
        dec.insert::<instructions::mvn::MvnImm>();
        dec.insert::<instructions::mvn::MvnReg>();
        dec.insert::<instructions::nop::Nop>();
        dec.insert::<instructions::orr::OrrImm>();
        dec.insert::<instructions::orr::OrrReg>();
        dec.insert::<instructions::pop::Pop>();
        dec.insert::<instructions::push::Push>();
        dec.insert::<instructions::sev::Sev>();
        dec.insert::<instructions::rbit::Rbit>();
        dec.insert::<instructions::rev::Rev>();
        dec.insert::<instructions::rev16::Rev16>();
        dec.insert::<instructions::rsb::RsbImm>();
        dec.insert::<instructions::rsb::RsbReg>();
        dec.insert::<instructions::sdiv::Sdiv>();
        dec.insert::<instructions::stmdb::Stmdb>();
        dec.insert::<instructions::stm::Stm>();
        dec.insert::<instructions::str::StrImm>();
        dec.insert::<instructions::str::StrReg>();
        dec.insert::<instructions::str::StrdImm>();
        dec.insert::<instructions::strb::StrbImm>();
        dec.insert::<instructions::strb::StrbReg>();
        dec.insert::<instructions::strh::StrhImm>();
        dec.insert::<instructions::sub::SubImm>();
        dec.insert::<instructions::sub::SubReg>();
        dec.insert::<instructions::sub::SubSpMinusImm>();
        dec.insert::<instructions::ubfx::Ubfx>();
        dec.insert::<instructions::udiv::Udiv>();
        dec.insert::<instructions::svc::Svc>();
        dec.insert::<instructions::tbb::Tbb>();
        dec.insert::<instructions::tst::TstImm>();
        dec.insert::<instructions::tst::TstReg>();
        dec.insert::<instructions::uxtb::Uxtb>();
        dec.insert::<instructions::uxth::Uxth>();
        Self(dec)
    }

    pub fn try_decode(
        &self,
        ins: u32,
        size: InstructionSize,
        state: ItState,
    ) -> Result<Box<dyn Instruction>, InstructionDecodeError> {
        self.0.try_decode(ins, size, state)
    }
}

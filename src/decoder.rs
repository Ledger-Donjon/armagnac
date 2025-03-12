//! Instruction decoding module.

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

#[derive(Debug)]
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

pub struct BasicInstructionDecoder(InstructionDecoder);

impl BasicInstructionDecoder {
    pub fn new() -> Self {
        let mut dec = InstructionDecoder::new();
        dec.insert::<instructions::adc::AdcImm>();
        dec.insert::<instructions::adc::AdcReg>();
        dec.insert::<instructions::add::AddImm>();
        dec.insert::<instructions::add::AddReg>();
        dec.insert::<instructions::add::AddSpPlusImm>();
        dec.insert::<instructions::add::AddSpPlusReg>();
        dec.insert::<instructions::adr::Adr>();
        dec.insert::<instructions::and::AndImm>();
        dec.insert::<instructions::and::AndReg>();
        dec.insert::<instructions::asr::AsrImm>();
        dec.insert::<instructions::asr::AsrReg>();
        dec.insert::<instructions::b::B>();
        dec.insert::<instructions::bfc::Bfc>();
        dec.insert::<instructions::bfi::Bfi>();
        dec.insert::<instructions::bic::BicImm>();
        dec.insert::<instructions::bic::BicReg>();
        dec.insert::<instructions::bl::Bl>();
        dec.insert::<instructions::blx::Blx>();
        dec.insert::<instructions::bx::Bx>();
        dec.insert::<instructions::cbnz::Cbnz>();
        dec.insert::<instructions::clz::Clz>();
        dec.insert::<instructions::cmn::CmnImm>();
        dec.insert::<instructions::cmn::CmnReg>();
        dec.insert::<instructions::cmp::CmpImm>();
        dec.insert::<instructions::cmp::CmpReg>();
        dec.insert::<instructions::cps::Cps>();
        dec.insert::<instructions::dsb::Dsb>();
        dec.insert::<instructions::eor::EorImm>();
        dec.insert::<instructions::eor::EorReg>();
        dec.insert::<instructions::isb::Isb>();
        dec.insert::<instructions::it::It>();
        dec.insert::<instructions::ldm::Ldm>();
        dec.insert::<instructions::ldmdb::Ldmdb>();
        dec.insert::<instructions::ldr::LdrImm>();
        dec.insert::<instructions::ldr::LdrImm>();
        dec.insert::<instructions::ldr::LdrLit>();
        dec.insert::<instructions::ldr::LdrReg>();
        dec.insert::<instructions::ldrb::LdrbImm>();
        dec.insert::<instructions::ldrb::LdrbReg>();
        dec.insert::<instructions::ldrb::LdrbLit>();
        dec.insert::<instructions::ldrd::LdrdImm>();
        dec.insert::<instructions::ldrd::LdrdLit>();
        dec.insert::<instructions::ldrh::LdrhImm>();
        dec.insert::<instructions::ldrh::LdrhLit>();
        dec.insert::<instructions::ldrh::LdrhReg>();
        dec.insert::<instructions::ldrh::Ldrht>();
        dec.insert::<instructions::ldrsb::LdrsbImm>();
        dec.insert::<instructions::ldrsb::LdrsbLit>();
        dec.insert::<instructions::ldrsb::LdrsbReg>();
        dec.insert::<instructions::ldrsh::LdrshImm>();
        dec.insert::<instructions::ldrsh::LdrshLit>();
        dec.insert::<instructions::ldrsh::LdrshReg>();
        dec.insert::<instructions::lsl::LslImm>();
        dec.insert::<instructions::lsl::LslReg>();
        dec.insert::<instructions::lsr::LsrImm>();
        dec.insert::<instructions::lsr::LsrReg>();
        dec.insert::<instructions::mla::Mla>();
        dec.insert::<instructions::mls::Mls>();
        dec.insert::<instructions::mov::MovImm>();
        dec.insert::<instructions::mov::MovReg>();
        dec.insert::<instructions::movt::Movt>();
        dec.insert::<instructions::mrs::Mrs>();
        dec.insert::<instructions::msr::Msr>();
        dec.insert::<instructions::mul::Mul>();
        dec.insert::<instructions::mvn::MvnImm>();
        dec.insert::<instructions::mvn::MvnReg>();
        dec.insert::<instructions::nop::Nop>();
        dec.insert::<instructions::orn::OrnImm>();
        dec.insert::<instructions::orn::OrnReg>();
        dec.insert::<instructions::orr::OrrImm>();
        dec.insert::<instructions::orr::OrrReg>();
        dec.insert::<instructions::pop::Pop>();
        dec.insert::<instructions::push::Push>();
        dec.insert::<instructions::sbc::SbcImm>();
        dec.insert::<instructions::sbc::SbcReg>();
        dec.insert::<instructions::sev::Sev>();
        dec.insert::<instructions::rbit::Rbit>();
        dec.insert::<instructions::rev::Rev>();
        dec.insert::<instructions::rev16::Rev16>();
        dec.insert::<instructions::ror::RorImm>();
        dec.insert::<instructions::ror::RorReg>();
        dec.insert::<instructions::rrx::Rrx>();
        dec.insert::<instructions::rsb::RsbImm>();
        dec.insert::<instructions::rsb::RsbReg>();
        dec.insert::<instructions::sbfx::Sbfx>();
        dec.insert::<instructions::sdiv::Sdiv>();
        dec.insert::<instructions::stmdb::Stmdb>();
        dec.insert::<instructions::stm::Stm>();
        dec.insert::<instructions::str::StrImm>();
        dec.insert::<instructions::str::StrReg>();
        dec.insert::<instructions::str::StrdImm>();
        dec.insert::<instructions::strb::StrbImm>();
        dec.insert::<instructions::strb::StrbReg>();
        dec.insert::<instructions::strh::StrhImm>();
        dec.insert::<instructions::strh::StrhReg>();
        dec.insert::<instructions::sub::SubImm>();
        dec.insert::<instructions::sub::SubReg>();
        dec.insert::<instructions::sub::SubSpMinusImm>();
        dec.insert::<instructions::svc::Svc>();
        dec.insert::<instructions::sxtb::Sxtb>();
        dec.insert::<instructions::sxth::Sxth>();
        dec.insert::<instructions::tbb::Tbb>();
        dec.insert::<instructions::teq::TeqImm>();
        dec.insert::<instructions::teq::TeqReg>();
        dec.insert::<instructions::tst::TstImm>();
        dec.insert::<instructions::tst::TstReg>();
        dec.insert::<instructions::ubfx::Ubfx>();
        dec.insert::<instructions::udiv::Udiv>();
        dec.insert::<instructions::umlal::Umlal>();
        dec.insert::<instructions::umull::Umull>();
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

#[cfg(test)]
mod tests {
    use std::{fs::File, io::{BufRead, BufReader}};
    use super::BasicInstructionDecoder;
    use crate::{arm::Mnemonic, instructions::InstructionSize, it_state::ItState};

    #[test]
    fn test_dissassembly() {
        let file = File::open("src/test_decoder.txt").unwrap();
        let buf_reader = BufReader::new(file);
        let decoder = BasicInstructionDecoder::new();

        for line in buf_reader.lines().map(|l| l.unwrap()) {
            // Skip comment lines
            if &line[..1] == "#" {
                continue;
            }

            let pos = line.find(" ").unwrap();
            let bytes = hex::decode(&line[..pos]).unwrap();
            let mnemonic = &line[pos + 1..];

            let halfword = u16::from_le_bytes(bytes[..2].try_into().unwrap());
            let size = InstructionSize::from_halfword(halfword);
            let ins: u32 = match size {
                InstructionSize::Ins16 => {
                    assert_eq!(bytes.len(), 2);
                    halfword as u32
                }
                InstructionSize::Ins32 => {
                    assert_eq!(bytes.len(), 4);
                    (halfword as u32) << 16 | u16::from_le_bytes(bytes[2..4].try_into().unwrap()) as u32
                }
            };

            let state = ItState::new();
            let ins = decoder.try_decode(ins, size, state).unwrap();
            assert_eq!(ins.mnemonic(0x1000), mnemonic);
        }
    }
}
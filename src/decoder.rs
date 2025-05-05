//! Instruction decoding module.
//!
//! Before executing an instruction, Armagnac has to decode it. First, the emulator fetches the
//! next instruction half-word from the memory. From this value, the emulator finds out if the
//! next instruction is encoded on 16-bit or 32-bit. For 32-bit instructions, it then fetches the
//! second halfword. Once the whole instruction code has been fetched, the emulator has to find to
//! which instruction it matches. This is done by using an intruction decoder - a struct which
//! implements the [InstructionDecode] trait.
//!
//! The instruction decoder implementation can be selected by changing
//! [crate::arm::ArmProcessor::instruction_decoder].

use crate::{
    arith::ArithError,
    arm::ArmVersion,
    instructions::{self, Instruction, InstructionSize},
    it_state::ItState,
};
use std::{fmt::Display, rc::Rc};

/// Any struct which implement this trait can be used by the emulator to decode instructions.
///
/// Depending on the emulation requirements, different decoding strategies may be implemened.
pub trait InstructionDecode {
    /// Tries to decode the given `ins` instruction raw code, which can be 16 bit or 32 bit wide
    /// depending on the `size` argument.
    ///
    /// If the processor is currently in an IT (If-Then) block, decoding may be different
    /// (typically an instruction may not set condition flags in the APSR register during an IT
    /// block). This can be decided from the `state` argument.
    ///
    /// When decoding is successful, an object which implements the [Instruction] trait is
    /// returned and can be applied to a processor state using [Instruction::execute] method.
    ///
    /// Note: currently this method cannot mutate the decoder, preventing cache or heuristics based
    /// optimisations.
    fn try_decode(
        &self,
        ins: u32,
        size: InstructionSize,
        state: ItState,
    ) -> Result<Rc<dyn Instruction>, InstructionDecodeError>;
}

/// Possible instruction decoding errors returned by [InstructionDecode] implementations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstructionDecodeError {
    /// Instruction is unknown or not yet supported.
    Unknown,
    /// Instruction is undefined, which should trigger a Usage Fault.
    /// This concerns instructions explicitely described as "undefined" by the Arm Architecture
    /// Reference Manual.
    Undefined,
    /// Instruction has been matched but some bits don't have the correct value, making the
    /// instruction effect unpredictable as indicated in the Arm Architecture Reference Manual.
    Unpredictable,
}

/// Indicates how one bit of an instruction code must be tested during instruction decoding.
/// Corresponds to the possible values in instruction patterns as can be seen in the Arm
/// Architecture Reference Manual: "0", "1", "(0)", "(1)" and "x" for bits of instruction
/// arguments.
#[derive(Clone, Copy)]
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
    fn(usize, u32, ItState) -> Result<Rc<dyn Instruction>, DecodeError>;

/// An instruction pattern which can be used to test if an opcode matches a
/// given instruction.
#[derive(Clone)]
pub struct InstructionPattern {
    /// Indicates how each bit of an instruction code must be tested during pattern matching.
    bits: Vec<InstructionPatternBit>,
    /// Bit mask for testing opcode value.
    test_mask: u32,
    /// Expected opcode value after masking.
    test_value: u32,
    /// Bit mask for testing if an instruction is unpredictable.
    unp_mask: u32,
    /// Expected arguments value after masking. If not matched, the instruction is unpredictable.
    unp_value: u32,
}

impl InstructionPattern {
    /// Create a new pattern matcher from a string expression.
    ///
    /// Expression string follows the syntax used in the Arm Architecture Reference manual: each
    /// bit can be defined as:
    /// - "0": instruction bit must be 0,
    /// - "1": instruction bit must be 1,
    /// - "x": instruction bit is part of an instruction argument.
    /// - "(0)": instruction bit is part of an instruction argument, but is expected to be 0,
    ///   otherwise the instruction (if matched) is unpredictable.
    /// - "(1)": instruction bit is part of an instruction argument, but is expected to be 1,
    ///   otherwise the instruction (if matched) is unpredictable.
    ///
    /// For example, the following creates a pattern to match the MRS instruction:
    ///
    /// ```
    /// # use armagnac::decoder::InstructionPattern;
    /// # use armagnac::instructions::InstructionSize;
    /// let pattern = InstructionPattern::new("11110011111(0)(1)(1)(1)(1)10(0)0xxxxxxxxxxxx");
    /// let code = 0xf3ef8308;
    /// assert!(pattern.test(code, InstructionSize::Ins32).is_ok());
    /// ```
    ///
    /// The pattern string must have 16 or 32 elements. If incorrect, this method will panic.
    pub fn new(pattern: &str) -> Self {
        // Parse pattern string expression to build a simpler pattern vector with one element per
        // bit.
        let mut bits = Vec::new();
        bits.reserve_exact(
            pattern
                .chars()
                .filter(|&c| c == '0' || c == '1' || c == 'x')
                .count(),
        );
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

        // Calculate the testing masks
        let mut test_mask = 0;
        let mut test_value = 0;
        let mut unp_mask = 0;
        let mut unp_value = 0;
        for bit in bits.iter() {
            let (tm, tv, um, uv) = match bit {
                InstructionPatternBit::OpcodeZero => (1, 0, 0, 0),
                InstructionPatternBit::OpcodeOne => (1, 1, 0, 0),
                InstructionPatternBit::Arg => (0, 0, 0, 0),
                InstructionPatternBit::ArgZero => (0, 0, 1, 0),
                InstructionPatternBit::ArgOne => (0, 0, 1, 1),
            };
            test_mask = (test_mask << 1) | tm;
            test_value = (test_value << 1) | tv;
            unp_mask = (unp_mask << 1) | um;
            unp_value = (unp_value << 1) | uv;
        }

        Self {
            bits,
            test_mask,
            test_value,
            unp_mask,
            unp_value,
        }
    }

    /// Returns whether given instruction code matches the pattern.
    /// Returns [InstructionDecodeError::Unpredictable] if the instruction matches but some of its
    /// argument bits are incorrect.
    pub fn test(&self, ins: u32, size: InstructionSize) -> Result<bool, InstructionDecodeError> {
        if ins & self.test_mask != self.test_value {
            return Ok(false);
        }
        // Testing this after checking the value is more efficient (rejecting from test_value has
        // higher probability).
        if size != self.size() {
            return Ok(false);
        }
        // Instruction matches, but we now need to check that the "(0)" and "(1)" arguments bits
        // matches as well.
        // Note this can only be done after the two previous tests.
        if ins & self.unp_mask != self.unp_value {
            return Err(InstructionDecodeError::Unpredictable);
        }
        return Ok(true);
    }

    /// Number of bits of the instruction code this pattern is supposed to match.
    pub fn size(&self) -> InstructionSize {
        match self.bits.len() {
            16 => InstructionSize::Ins16,
            32 => InstructionSize::Ins32,
            _ => panic!(),
        }
    }
}

impl Display for InstructionPattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut result = String::new();
        for bit in self.bits.iter() {
            result.push_str(match bit {
                InstructionPatternBit::OpcodeZero => "0",
                InstructionPatternBit::OpcodeOne => "1",
                InstructionPatternBit::Arg => "x",
                InstructionPatternBit::ArgZero => "(0)",
                InstructionPatternBit::ArgOne => "(1)",
            })
        }
        write!(f, "{}", result)
    }
}

/// Associates an instruction decoding function to the corresponding multiple instruction patterns.
#[derive(Clone)]
pub struct BasicDecoderEntry {
    /// All possible patterns which can match for the given instruction.
    /// Pattern number of index 0 corresponds to T1 encoding, pattern of index 1 to T2 encoding,
    /// etc.
    pub patterns: Vec<(usize, InstructionPattern)>,
    /// Decoding function of the instruction.
    pub decoder: InstructionDecodingFunction,
}

/// A very simple and unoptimized instruction decoder.
///
/// Decoder initialization is fast, but decoding is in `O(N)` where `N` is the total number of
/// instruction patterns. This makes this decoder suitable for short emulations.
///
/// This decoder can be reused to build a faster/optimized decoder on top of it. A good example of
/// this is [Lut16InstructionDecoder].
pub struct BasicInstructionDecoder {
    pub entries: Vec<BasicDecoderEntry>,
}

fn rc_decoder<T: 'static + Instruction>(
    tn: usize,
    ins: u32,
    state: ItState,
) -> Result<Rc<dyn Instruction>, DecodeError> {
    match T::try_decode(tn, ins, state) {
        Ok(x) => Ok(Rc::new(x)),
        Err(e) => Err(e),
    }
}

impl BasicInstructionDecoder {
    pub fn new(version: ArmVersion) -> Self {
        let mut dec = Self {
            entries: Vec::new(),
        };
        dec.insert::<instructions::adc::AdcImm>(version);
        dec.insert::<instructions::adc::AdcReg>(version);
        dec.insert::<instructions::add::AddImm>(version);
        dec.insert::<instructions::add::AddReg>(version);
        dec.insert::<instructions::add::AddSpPlusImm>(version);
        dec.insert::<instructions::add::AddSpPlusReg>(version);
        dec.insert::<instructions::adr::Adr>(version);
        dec.insert::<instructions::and::AndImm>(version);
        dec.insert::<instructions::and::AndReg>(version);
        dec.insert::<instructions::asr::AsrImm>(version);
        dec.insert::<instructions::asr::AsrReg>(version);
        dec.insert::<instructions::b::B>(version);
        dec.insert::<instructions::bfc::Bfc>(version);
        dec.insert::<instructions::bfi::Bfi>(version);
        dec.insert::<instructions::bic::BicImm>(version);
        dec.insert::<instructions::bic::BicReg>(version);
        dec.insert::<instructions::bkpt::Bkpt>(version);
        dec.insert::<instructions::bl::Bl>(version);
        dec.insert::<instructions::blx::Blx>(version);
        dec.insert::<instructions::bx::Bx>(version);
        dec.insert::<instructions::cbnz::Cbnz>(version);
        dec.insert::<instructions::clz::Clz>(version);
        dec.insert::<instructions::cmn::CmnImm>(version);
        dec.insert::<instructions::cmn::CmnReg>(version);
        dec.insert::<instructions::cmp::CmpImm>(version);
        dec.insert::<instructions::cmp::CmpReg>(version);
        dec.insert::<instructions::cps::Cps>(version);
        dec.insert::<instructions::dmb::Dmb>(version);
        dec.insert::<instructions::dsb::Dsb>(version);
        dec.insert::<instructions::eor::EorImm>(version);
        dec.insert::<instructions::eor::EorReg>(version);
        dec.insert::<instructions::isb::Isb>(version);
        dec.insert::<instructions::it::It>(version);
        dec.insert::<instructions::ldm::Ldm>(version);
        dec.insert::<instructions::ldmdb::Ldmdb>(version);
        dec.insert::<instructions::ldr::LdrImm>(version);
        dec.insert::<instructions::ldr::LdrImm>(version);
        dec.insert::<instructions::ldr::LdrLit>(version);
        dec.insert::<instructions::ldr::LdrReg>(version);
        dec.insert::<instructions::ldrb::LdrbImm>(version);
        dec.insert::<instructions::ldrb::LdrbReg>(version);
        dec.insert::<instructions::ldrb::LdrbLit>(version);
        dec.insert::<instructions::ldrbt::Ldrbt>(version);
        dec.insert::<instructions::ldrd::LdrdImm>(version);
        dec.insert::<instructions::ldrd::LdrdLit>(version);
        dec.insert::<instructions::ldrh::LdrhImm>(version);
        dec.insert::<instructions::ldrh::LdrhLit>(version);
        dec.insert::<instructions::ldrh::LdrhReg>(version);
        dec.insert::<instructions::ldrt::Ldrt>(version);
        dec.insert::<instructions::ldrht::Ldrht>(version);
        dec.insert::<instructions::ldrsb::LdrsbImm>(version);
        dec.insert::<instructions::ldrsb::LdrsbLit>(version);
        dec.insert::<instructions::ldrsb::LdrsbReg>(version);
        dec.insert::<instructions::ldrsh::LdrshImm>(version);
        dec.insert::<instructions::ldrsh::LdrshLit>(version);
        dec.insert::<instructions::ldrsh::LdrshReg>(version);
        dec.insert::<instructions::lsl::LslImm>(version);
        dec.insert::<instructions::lsl::LslReg>(version);
        dec.insert::<instructions::lsr::LsrImm>(version);
        dec.insert::<instructions::lsr::LsrReg>(version);
        dec.insert::<instructions::mla::Mla>(version);
        dec.insert::<instructions::mls::Mls>(version);
        dec.insert::<instructions::mov::MovImm>(version);
        dec.insert::<instructions::mov::MovReg>(version);
        dec.insert::<instructions::mov::MovRegShiftReg>(version);
        dec.insert::<instructions::movt::Movt>(version);
        dec.insert::<instructions::mrs::Mrs>(version);
        dec.insert::<instructions::msr::Msr>(version);
        dec.insert::<instructions::mul::Mul>(version);
        dec.insert::<instructions::mvn::MvnImm>(version);
        dec.insert::<instructions::mvn::MvnReg>(version);
        dec.insert::<instructions::nop::Nop>(version);
        dec.insert::<instructions::orn::OrnImm>(version);
        dec.insert::<instructions::orn::OrnReg>(version);
        dec.insert::<instructions::orr::OrrImm>(version);
        dec.insert::<instructions::orr::OrrReg>(version);
        dec.insert::<instructions::pop::Pop>(version);
        dec.insert::<instructions::push::Push>(version);
        dec.insert::<instructions::sbc::SbcImm>(version);
        dec.insert::<instructions::sbc::SbcReg>(version);
        dec.insert::<instructions::sev::Sev>(version);
        dec.insert::<instructions::qadd::Qadd>(version);
        dec.insert::<instructions::qadd8::Qadd8>(version);
        dec.insert::<instructions::qadd16::Qadd16>(version);
        dec.insert::<instructions::qdadd::Qdadd>(version);
        dec.insert::<instructions::qdsub::Qdsub>(version);
        dec.insert::<instructions::qsub::Qsub>(version);
        dec.insert::<instructions::qsub16::Qsub16>(version);
        dec.insert::<instructions::qsub8::Qsub8>(version);
        dec.insert::<instructions::rbit::Rbit>(version);
        dec.insert::<instructions::rev::Rev>(version);
        dec.insert::<instructions::rev16::Rev16>(version);
        dec.insert::<instructions::revsh::Revsh>(version);
        dec.insert::<instructions::ror::RorImm>(version);
        dec.insert::<instructions::ror::RorReg>(version);
        dec.insert::<instructions::rrx::Rrx>(version);
        dec.insert::<instructions::rsb::RsbImm>(version);
        dec.insert::<instructions::rsb::RsbReg>(version);
        dec.insert::<instructions::sadd8::Sadd8>(version);
        dec.insert::<instructions::sbfx::Sbfx>(version);
        dec.insert::<instructions::sdiv::Sdiv>(version);
        dec.insert::<instructions::ssat::Ssat>(version);
        dec.insert::<instructions::stmdb::Stmdb>(version);
        dec.insert::<instructions::stm::Stm>(version);
        dec.insert::<instructions::str::StrImm>(version);
        dec.insert::<instructions::str::StrReg>(version);
        dec.insert::<instructions::strd::StrdImm>(version);
        dec.insert::<instructions::strb::StrbImm>(version);
        dec.insert::<instructions::strb::StrbReg>(version);
        dec.insert::<instructions::strh::StrhImm>(version);
        dec.insert::<instructions::strh::StrhReg>(version);
        dec.insert::<instructions::sub::SubImm>(version);
        dec.insert::<instructions::sub::SubReg>(version);
        dec.insert::<instructions::sub::SubSpMinusImm>(version);
        dec.insert::<instructions::sub::SubSpMinusReg>(version);
        dec.insert::<instructions::svc::Svc>(version);
        dec.insert::<instructions::sxtb::Sxtb>(version);
        dec.insert::<instructions::sxth::Sxth>(version);
        dec.insert::<instructions::tbb::Tbb>(version);
        dec.insert::<instructions::teq::TeqImm>(version);
        dec.insert::<instructions::teq::TeqReg>(version);
        dec.insert::<instructions::tst::TstImm>(version);
        dec.insert::<instructions::tst::TstReg>(version);
        dec.insert::<instructions::ubfx::Ubfx>(version);
        dec.insert::<instructions::udf::Udf>(version);
        dec.insert::<instructions::udiv::Udiv>(version);
        dec.insert::<instructions::umlal::Umlal>(version);
        dec.insert::<instructions::umull::Umull>(version);
        dec.insert::<instructions::uxtb::Uxtb>(version);
        dec.insert::<instructions::uxth::Uxth>(version);
        dec
    }

    pub fn insert<T: 'static + Instruction>(&mut self, version: ArmVersion) {
        let mut patterns = Vec::new();
        for pattern in T::patterns().iter() {
            if pattern.versions.iter().any(|&v| v == version) {
                patterns.push((pattern.tn, InstructionPattern::new(pattern.expression)));
            }
        }
        if patterns.len() > 0 {
            self.entries.push(BasicDecoderEntry {
                patterns,
                decoder: rc_decoder::<T>,
            });
        }
    }
}

impl InstructionDecode for BasicInstructionDecoder {
    fn try_decode(
        &self,
        ins: u32,
        size: InstructionSize,
        state: ItState,
    ) -> Result<Rc<dyn Instruction>, InstructionDecodeError> {
        for entry in &self.entries {
            for (tn, pattern) in entry.patterns.iter() {
                if pattern.test(ins, size)? {
                    if let Ok(ins) = (entry.decoder)(*tn, ins, state) {
                        return Ok(ins);
                    }
                }
            }
        }
        Err(InstructionDecodeError::Unknown)
    }
}

/// Instruction decoder with a look-up-table for 16-bit instruction decoding.
///
/// This has 16-bit instruction decoding O(1) complexity, but has a strong cost during
/// initialization to generate the table. The table is initialized using a given base instruction
/// decoder.
///
/// The look-up table is only valid outside IT blocks. 16-bit instructions inside an IT block are
/// resolved using the given base decoder.
///
/// 32-bit wide instructions are resolved using the given base instruction decoder.
pub struct Lut16InstructionDecoder {
    base_decoder: BasicInstructionDecoder,
    lut16: Vec<Result<Rc<dyn Instruction>, InstructionDecodeError>>,
}

impl Lut16InstructionDecoder {
    pub fn new(version: ArmVersion) -> Self {
        let base_decoder = BasicInstructionDecoder::new(version);
        let lut16 = (0..=u16::MAX)
            .map(|i| base_decoder.try_decode(i as u32, InstructionSize::Ins16, ItState::new()))
            .collect();
        Self {
            base_decoder,
            lut16,
        }
    }
}

impl InstructionDecode for Lut16InstructionDecoder {
    fn try_decode(
        &self,
        ins: u32,
        size: InstructionSize,
        state: ItState,
    ) -> Result<Rc<dyn Instruction>, InstructionDecodeError> {
        match size {
            InstructionSize::Ins16 => {
                if state.0 == 0 {
                    self.lut16[ins as usize].clone()
                } else {
                    self.base_decoder.try_decode(ins, size, state)
                }
            }
            InstructionSize::Ins32 => self.base_decoder.try_decode(ins, size, state),
        }
    }
}

/// Divides instruction matching search space in groups, indexed by the most significant bits of
/// their encoding.
///
/// Instruction matching is `in O(N / (head_bit_count ^ 2))` (if instructions most significant bits
/// are uniformely distributed, which is not the case for Arm 32 bit encodings).
pub struct GroupedInstructionDecoder {
    /// Number of bits used for grouping.
    /// Always greater than 0 and lower than 32.
    head_bit_count: u8,
    /// Groups of instruction and patterns.
    /// There is always `2^head_bit_count` groups.
    entries: Vec<Vec<(InstructionPattern, usize, InstructionDecodingFunction)>>,
}

impl GroupedInstructionDecoder {
    pub fn new(head_bit_count: u8) -> Self {
        debug_assert!((head_bit_count > 0) && (head_bit_count < 32));
        let mut entries = Vec::new();
        entries.resize_with(1 << head_bit_count, || Vec::new());
        Self {
            head_bit_count,
            entries,
        }
    }

    pub fn try_from_basic_decoder(head_bit_count: u8, version: ArmVersion) -> Result<Self, ()> {
        let mut result = Self::new(head_bit_count);
        let basic_decoder = BasicInstructionDecoder::new(version);
        for entry in basic_decoder.entries {
            result.try_insert_from_decoder_entry(&entry)?;
        }
        Ok(result)
    }

    pub fn try_insert(
        &mut self,
        pattern: &InstructionPattern,
        tn: usize,
        f: InstructionDecodingFunction,
    ) -> Result<(), ()> {
        let mut group = 0;
        for pattern_bit in pattern.bits[0..self.head_bit_count as usize].iter() {
            let bit = match pattern_bit {
                InstructionPatternBit::OpcodeZero => 0,
                InstructionPatternBit::OpcodeOne => 1,
                _ => return Err(()),
            };
            group = (group << 1) | bit;
        }
        self.entries[group].push((pattern.clone(), tn, f));
        Ok(())
    }

    pub fn try_insert_from_decoder_entry(&mut self, entry: &BasicDecoderEntry) -> Result<(), ()> {
        for (tn, pattern) in entry.patterns.iter() {
            self.try_insert(pattern, *tn, entry.decoder)?
        }
        Ok(())
    }
}

impl InstructionDecode for GroupedInstructionDecoder {
    fn try_decode(
        &self,
        ins: u32,
        size: InstructionSize,
        state: ItState,
    ) -> Result<Rc<dyn Instruction>, InstructionDecodeError> {
        let group = ins >> (size.bit_count() - self.head_bit_count as usize);
        for (pattern, tn, f) in self.entries[group as usize].iter() {
            if pattern.test(ins, size)? {
                if let Ok(ins) = (f)(*tn, ins, state) {
                    return Ok(ins);
                }
            }
        }
        Err(InstructionDecodeError::Unknown)
    }
}

/// A decoder mixing a [Lut16InstructionDecoder] for 16 bit encodings, and a
/// [GroupedInstructionDecoder] for 32 bit encodings. This is the fastest decoder provided by
/// Armagnac, but has a very long initialization time due to the generation of the look-up table.
pub struct Lut16AndGrouped32InstructionDecoder {
    /// The decoder used for 16 bit encodings.
    lut_decoder: Lut16InstructionDecoder,
    /// The decoder used for 32 bit encodings.
    group_decoder: GroupedInstructionDecoder,
}

impl Lut16AndGrouped32InstructionDecoder {
    pub fn new(version: ArmVersion) -> Self {
        let lut_decoder = Lut16InstructionDecoder::new(version);
        let mut group_decoder = GroupedInstructionDecoder::new(5);
        for entry in lut_decoder.base_decoder.entries.iter() {
            for (tn, pattern) in entry
                .patterns
                .iter()
                .filter(|(_, p)| p.size() == InstructionSize::Ins32)
            {
                group_decoder
                    .try_insert(pattern, *tn, entry.decoder)
                    .unwrap()
            }
        }
        Self {
            lut_decoder,
            group_decoder,
        }
    }
}

impl InstructionDecode for Lut16AndGrouped32InstructionDecoder {
    fn try_decode(
        &self,
        ins: u32,
        size: InstructionSize,
        state: ItState,
    ) -> Result<Rc<dyn Instruction>, InstructionDecodeError> {
        match size {
            InstructionSize::Ins16 => self.lut_decoder.try_decode(ins, size, state),
            InstructionSize::Ins32 => self.group_decoder.try_decode(ins, size, state),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        BasicInstructionDecoder, Lut16AndGrouped32InstructionDecoder, Lut16InstructionDecoder,
    };
    use crate::{
        arm::{ArmProcessor, ArmVersion::V7M},
        decoder::InstructionDecode,
        instructions::{InstructionSize, Mnemonic},
        it_state::ItState,
    };
    use rand::Rng;
    use std::{
        any::Any,
        fs::File,
        io::{BufRead, BufReader},
    };

    #[test]
    fn test_dissassembly() {
        let file = File::open("src/test_decoder.txt").unwrap();
        let buf_reader = BufReader::new(file);
        let decoder = BasicInstructionDecoder::new(V7M);
        let mut proc = ArmProcessor::new(V7M, 0);
        let mut pc = 0x1000;

        for line in buf_reader.lines().map(|l| l.unwrap()) {
            // Skip comment lines
            if &line[..1] == "#" {
                continue;
            }

            let pos = line.find(" ").unwrap();
            let bytes = hex::decode(&line[..pos]).unwrap();
            let mnemonic = &line[9..];

            let halfword = u16::from_le_bytes(bytes[..2].try_into().unwrap());
            let size = InstructionSize::from_halfword(halfword);
            let ins: u32 = match size {
                InstructionSize::Ins16 => {
                    assert_eq!(bytes.len(), 2);
                    halfword as u32
                }
                InstructionSize::Ins32 => {
                    assert_eq!(bytes.len(), 4);
                    (halfword as u32) << 16
                        | u16::from_le_bytes(bytes[2..4].try_into().unwrap()) as u32
                }
            };

            let mut state = proc.registers.psr.it_state();
            let cond = state.current_condition();
            let ins = decoder.try_decode(ins, size, state).unwrap();
            let got_mnemonic = ins.mnemonic(pc, cond);
            if got_mnemonic != mnemonic {
                println!("Mnemonic generation failed:");
                println!("  Expected: {mnemonic}");
                println!("  Got     : {got_mnemonic}");
                panic!();
            }
            state.advance();
            proc.registers.psr.set_it_state(state);

            // We need to execute the instruction in the case it is an IT. This way we get the
            // correct it state for decoding the next instruction when we are inside an IT block
            // llvm-objdump does set the condition in the name of instruction following IT
            // instructions, we need to do the same.
            let _result = ins.execute(&mut proc);

            pc += size.byte_count() as u32;
        }
    }

    fn test_decoder(
        a: &dyn InstructionDecode,
        b: &dyn InstructionDecode,
        ins: u32,
        size: InstructionSize,
        it: ItState,
    ) {
        let ins_a = a.try_decode(ins, size, it);
        let ins_b = b.try_decode(ins, size, it);
        match (ins_a, ins_b) {
            (Ok(ins_a), Ok(ins_b)) => {
                assert_eq!(ins_a.type_id(), ins_b.type_id())
            }
            (Err(err_a), Err(err_b)) => {
                assert_eq!(err_a, err_b)
            }
            _ => panic!(),
        }
    }

    /// Checks that [Lut16InstructionDecoder] always decodes the same as [BasicInstructionDecoder].
    #[test]
    fn test_instruction_decoders() {
        let dec_a = BasicInstructionDecoder::new(V7M);
        let dec_b = Lut16InstructionDecoder::new(V7M);
        let dec_c = Lut16AndGrouped32InstructionDecoder::new(V7M);
        let it = ItState::new();

        for i in 0..=u16::MAX {
            test_decoder(&dec_a, &dec_b, i as u32, InstructionSize::Ins16, it);
            test_decoder(&dec_a, &dec_c, i as u32, InstructionSize::Ins16, it);
        }

        // For 32 bit encodings we cannot test the whole space, so pick a high number of random
        // tests.
        let mut rng = rand::rng();
        for _ in 0..=100000 {
            let ins = rng.random();
            test_decoder(&dec_a, &dec_b, ins, InstructionSize::Ins32, it);
            test_decoder(&dec_a, &dec_c, ins, InstructionSize::Ins32, it);
        }
    }
}

use crate::{
    arm::{Arm7Processor, RunError},
    condition::Condition,
    decoder::DecodeError,
    it_state::ItState,
    registers::RegisterIndex,
};

pub mod adc;
pub mod add;
pub mod adr;
pub mod and;
pub mod asr;
pub mod b;
pub mod bfc;
pub mod bic;
pub mod bl;
pub mod blx;
pub mod bx;
pub mod cbnz;
pub mod clz;
pub mod cmn;
pub mod cmp;
pub mod cps;
pub mod dsb;
pub mod eor;
pub mod isb;
pub mod it;
pub mod ldm;
pub mod ldmdb;
pub mod ldr;
pub mod ldrb;
pub mod ldrd;
pub mod ldrh;
pub mod ldrsb;
pub mod ldrsh;
pub mod lsl;
pub mod lsr;
pub mod mla;
pub mod mls;
pub mod mov;
pub mod mrs;
pub mod msr;
pub mod mul;
pub mod mvn;
pub mod nop;
pub mod orn;
pub mod orr;
pub mod pop;
pub mod push;
pub mod rbit;
pub mod rev;
pub mod rev16;
pub mod ror;
pub mod rsb;
pub mod sbc;
pub mod sbfx;
pub mod sdiv;
pub mod sev;
pub mod stm;
pub mod stmdb;
pub mod str;
pub mod strb;
pub mod strh;
pub mod sub;
pub mod svc;
pub mod sxtb;
pub mod sxth;
pub mod tbb;
pub mod tst;
pub mod ubfx;
pub mod udiv;
pub mod umlal;
pub mod umull;
pub mod uxtb;
pub mod uxth;

/// All instructions must implement this trait in order to be integrated into the emulator.
pub trait Instruction {
    /// Returns a list of string patterns the instruction can match.
    ///
    /// Patterns must define each bit of the instruction with following possible symbols:
    /// - "0" when a bit must be zero,
    /// - "1" when a bit must be one,
    /// - "x" for bits part of instruction arguments (registers indexes, immediate values, etc.),
    /// - "(0)" for bits part of instruction arguments, but expected to be zero,
    /// - "(1)" for bits part of instruction arguments, but expected to be one.
    ///
    /// In the returned array, element 0 is the pattern for T1 encoding, element 1 for T2 encoding,
    /// etc.
    ///
    /// Patterns format matches the patterns indicated in the ARM Architecture Reference Manual,
    /// making easy to define them with little room for mistakes.
    ///
    /// Patterns can have 16 or 32 symbols. If not, the instruction decoder is expected to panic.
    ///
    /// For instance, the pattern "0001110xxxxxxxxx" matches the ADD instruction T1 encoding.
    fn patterns() -> &'static [&'static str]
    where
        Self: Sized;

    /// Tries to decode a 16-bit or 32-bit instruction.
    ///
    /// `tn` corresponds to the matched encoding (1 for T1, 2 for T2, etc.) and `ins` is the
    /// instruction value. The current IT state is also passed as parameter, as an instruction can
    /// have a different behavior whether it is executed inside an IT block or not.
    ///
    /// If an instruction matches a pattern but the arguments values lead to another instruction
    /// instead, [DecodeError::Other] is to be returned so the instruction decoder will try to find
    /// another matching instruction. Depending on instruction arguments
    /// [DecodeError::Unpredictable] or [DecodeError::Undefined] may be returned as well. Those
    /// errors always matches the precise description of the ARM Technical Reference Manual.
    ///
    /// [other], [unpredictable] and [undefined] methods can be used as shorthands for testing
    /// conditions and returning corresponding errors, as in the following example:
    ///
    /// ```
    /// rd = ins.reg4(0);
    /// unpredictable(rd.is_sp_or_pc())?;
    /// ```
    fn try_decode(tn: usize, ins: u32, state: ItState) -> Result<Self, DecodeError>
    where
        Self: Sized;

    /// Returns instruction execution own condition, or None if the condition in the IT state is to
    /// be followed. Only the B instruction in T1 and T3 encodings should implement this, all other
    /// instructions rely on the blanket implementation which returns [None].
    fn condition(&self) -> Option<Condition> {
        None
    }

    fn execute(&self, proc: &mut Arm7Processor) -> Result<bool, RunError>;

    fn name(&self) -> String;

    fn args(&self, pc: u32) -> String;
}

/// If condition passes, returns [DecodeError::Unpredictable] error.
pub fn unpredictable(cond: bool) -> Result<(), DecodeError> {
    if cond {
        Err(DecodeError::Unpredictable)
    } else {
        Ok(())
    }
}

/// If condition passes, returns [DecodeError::Undefined] error.
pub fn undefined(cond: bool) -> Result<(), DecodeError> {
    if cond {
        Err(DecodeError::Undefined)
    } else {
        Ok(())
    }
}

/// If condition passes, returns [DecodeError::Other] error.
pub fn other(cond: bool) -> Result<(), DecodeError> {
    if cond {
        Err(DecodeError::Other)
    } else {
        Ok(())
    }
}

/// Possible instruction sizes for ARM Thumb.
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum InstructionSize {
    /// 16-bit instruction.
    Ins16 = 2,
    /// 32-bit instruction.
    Ins32 = 4,
}

impl InstructionSize {
    /// Returns thumb instruction size based on first halfword value.
    pub fn from_halfword(halfword: u16) -> Self {
        match (halfword & 0xf800) >> 11 {
            0b11101..=0b11111 => InstructionSize::Ins32,
            _ => InstructionSize::Ins16,
        }
    }
}

pub trait DecodeHelper {
    fn reg3(&self, lsb_index: u8) -> RegisterIndex;
    fn reg4(&self, lsb_index: u8) -> RegisterIndex;
    fn imm1(&self, lsb_index: u8) -> u32;
    fn imm2(&self, lsb_index: u8) -> u32;
    fn imm3(&self, lsb_index: u8) -> u32;
    fn imm4(&self, lsb_index: u8) -> u32;
    fn imm5(&self, lsb_index: u8) -> u32;
    fn imm8(&self, lsb_index: u8) -> u32;
    fn imm12(&self, lsb_index: u8) -> u32;
    fn puw(&self) -> (bool, bool, bool);
}

impl DecodeHelper for u32 {
    fn reg3(&self, lsb_index: u8) -> RegisterIndex {
        RegisterIndex::new_main(self >> lsb_index & 0x7)
    }

    fn reg4(&self, lsb_index: u8) -> RegisterIndex {
        RegisterIndex::new_main(self >> lsb_index & 0xf)
    }

    fn imm1(&self, lsb_index: u8) -> u32 {
        self >> lsb_index & 1
    }

    fn imm2(&self, lsb_index: u8) -> u32 {
        self >> lsb_index & 3
    }

    fn imm3(&self, lsb_index: u8) -> u32 {
        self >> lsb_index & 7
    }

    fn imm4(&self, lsb_index: u8) -> u32 {
        self >> lsb_index & 0xf
    }

    fn imm5(&self, lsb_index: u8) -> u32 {
        self >> lsb_index & 0x1f
    }

    fn imm8(&self, lsb_index: u8) -> u32 {
        self >> lsb_index & 0xff
    }

    fn imm12(&self, lsb_index: u8) -> u32 {
        self >> lsb_index & 0xfff
    }

    /// Extracts `index`, `add` and `wback` flags respectively from bits 10, 9 and 8.
    fn puw(&self) -> (bool, bool, bool) {
        (self & 1 << 10 != 0, self & 1 << 9 != 0, self & 1 << 8 != 0)
    }
}

/// Returns "{rd}" if rd is equal to rn, else "{rd}, {rn}".
///
/// This is a convenient method used for many instruction arguments formatting.
pub fn rdn_args_string(rd: RegisterIndex, rn: RegisterIndex) -> String {
    if rd == rn {
        format!("{rd}")
    } else {
        format!("{rd}, {rn}")
    }
}

/// Returns instruction indexing argument string.
///
/// Depending on the `index`, `add`, `wback` and `imm` parameters, can be one of
/// - "[Rn]"
/// - "[Rn, #imm]"
/// - "[Rn], #imm"
/// - "[Rn, #imm]!"
pub fn indexing_args(rn: RegisterIndex, imm: u32, index: bool, add: bool, wback: bool) -> String {
    let neg = if add { "" } else { "-" };
    match (index, wback) {
        (true, false) => {
            let imm = if imm != 0 {
                format!(", #{}{}", neg, imm)
            } else {
                "".into()
            };
            format!("[{}{}]", rn, imm)
        }
        (true, true) => format!("[{rn}, #{neg}{imm}]!"),
        (false, true) => format!("[{rn}], #{neg}{imm}"),
        (false, false) => panic!(),
    }
}

/// Utility trait to call either addition or subtraction between two values depending on a
/// condition.
///
/// This is to be used by many instructions which have a bit indicating if an offset is to be added
/// to a base value, or subtracted.
pub trait AddOrSub {
    fn add_or_sub(&self, rhs: Self, add: bool) -> Self;
    fn wrapping_add_or_sub(&self, rhs: Self, add: bool) -> Self;
}

impl AddOrSub for u32 {
    fn add_or_sub(&self, rhs: Self, add: bool) -> Self {
        if add {
            self + rhs
        } else {
            self - rhs
        }
    }

    fn wrapping_add_or_sub(&self, rhs: Self, add: bool) -> Self {
        if add {
            self.wrapping_add(rhs)
        } else {
            self.wrapping_sub(rhs)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        instructions::{indexing_args, rdn_args_string, InstructionSize},
        registers::RegisterIndex,
    };

    #[test]
    fn test_instruction_size() {
        for i in 0..u16::MAX {
            assert_eq!(
                InstructionSize::from_halfword(i),
                if i >= 0xe800 {
                    InstructionSize::Ins32
                } else {
                    InstructionSize::Ins16
                }
            )
        }
    }

    #[test]
    fn test_rdn_args_string() {
        assert_eq!(rdn_args_string(RegisterIndex::R0, RegisterIndex::R0), "r0");
        assert_eq!(
            rdn_args_string(RegisterIndex::R0, RegisterIndex::R1),
            "r0, r1"
        );
    }

    #[test]
    fn test_indexing_args() {
        assert_eq!(
            indexing_args(RegisterIndex::R1, 0, true, true, false),
            "[r1]"
        );
        assert_eq!(
            indexing_args(RegisterIndex::R1, 0, true, false, false),
            "[r1]"
        );
        assert_eq!(
            indexing_args(RegisterIndex::R1, 12, true, true, false),
            "[r1, #12]"
        );
        assert_eq!(
            indexing_args(RegisterIndex::R1, 12, true, false, false),
            "[r1, #-12]"
        );

        assert_eq!(
            indexing_args(RegisterIndex::R1, 0, true, true, true),
            "[r1, #0]!"
        );
        assert_eq!(
            indexing_args(RegisterIndex::R1, 0, true, false, true),
            "[r1, #-0]!"
        );
        assert_eq!(
            indexing_args(RegisterIndex::R1, 12, true, true, true),
            "[r1, #12]!"
        );
        assert_eq!(
            indexing_args(RegisterIndex::R1, 12, true, false, true),
            "[r1, #-12]!"
        );

        assert_eq!(
            indexing_args(RegisterIndex::R1, 0, false, true, true),
            "[r1], #0"
        );
        assert_eq!(
            indexing_args(RegisterIndex::R1, 0, false, false, true),
            "[r1], #-0"
        );
        assert_eq!(
            indexing_args(RegisterIndex::R1, 12, false, true, true),
            "[r1], #12"
        );
        assert_eq!(
            indexing_args(RegisterIndex::R1, 12, false, false, true),
            "[r1], #-12"
        );
    }
}

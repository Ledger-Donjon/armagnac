//! ARM instructions implementation module.
//!
//! Each implemented instruction has its own submodule, holding a struct for each variant
//! (immediate, register, etc.) of the instruction. For example, the AND instruction is implemented
//! in the [and] module by the [and::AndImm] for the Immediate variant and [and::AndReg] for the
//! Register variant.

use crate::{
    arm::{ArmProcessor, ArmVersion, Effect, RunError},
    condition::Condition,
    decoder::DecodeError,
    it_state::ItState,
    registers::RegisterIndex,
};
use std::rc::Rc;

pub mod adc;
pub mod add;
pub mod adr;
pub mod and;
pub mod asr;
pub mod b;
pub mod bfc;
pub mod bfi;
pub mod bic;
pub mod bkpt;
pub mod bl;
pub mod blx;
pub mod bx;
pub mod cbnz;
pub mod clz;
pub mod cmn;
pub mod cmp;
pub mod cps;
pub mod dmb;
pub mod dsb;
pub mod eor;
pub mod isb;
pub mod it;
pub mod ldm;
pub mod ldmdb;
pub mod ldr;
pub mod ldrb;
pub mod ldrbt;
pub mod ldrd;
pub mod ldrh;
pub mod ldrht;
pub mod ldrsb;
pub mod ldrsh;
pub mod ldrt;
pub mod lsl;
pub mod lsr;
pub mod mla;
pub mod mls;
pub mod mov;
pub mod movt;
pub mod mrs;
pub mod msr;
pub mod mul;
pub mod mvn;
pub mod nop;
pub mod orn;
pub mod orr;
pub mod pop;
pub mod push;
pub mod qadd;
pub mod qadd16;
pub mod qadd8;
pub mod qdadd;
pub mod qdsub;
pub mod qsub;
pub mod qsub16;
pub mod qsub8;
pub mod rbit;
pub mod rev;
pub mod rev16;
pub mod revsh;
pub mod ror;
pub mod rrx;
pub mod rsb;
pub mod sadd16;
pub mod sadd8;
pub mod sbc;
pub mod sbfx;
pub mod sdiv;
pub mod sev;
pub mod smlal;
pub mod smull;
pub mod ssat;
pub mod stm;
pub mod stmdb;
pub mod str;
pub mod strb;
pub mod strd;
pub mod strh;
pub mod sub;
pub mod svc;
pub mod sxtb;
pub mod sxth;
pub mod tbb;
pub mod teq;
pub mod tst;
pub mod ubfx;
pub mod udf;
pub mod udiv;
pub mod umlal;
pub mod umull;
pub mod usat;
pub mod usat16;
pub mod uxtb;
pub mod uxth;
pub mod r#yield;

/// Defines how to match an instruction encoding.
pub struct Pattern {
    /// Instruction encoding as specified in the Arm Architecture Reference Manual.
    pub encoding: Encoding,
    /// Arm architecture versions supporting the current instruction encoding.
    pub versions: &'static [ArmVersion],
    /// Regular expression used to test if the corresponding instruction matches bytes being
    /// decoded.
    ///
    /// Pattern expression must define each bit of the instruction with following possible symbols:
    /// - "0" when a bit must be zero,
    /// - "1" when a bit must be one,
    /// - "x" for bits part of instruction arguments (registers indexes, immediate values, etc.),
    /// - "(0)" for bits part of instruction arguments, but expected to be zero,
    /// - "(1)" for bits part of instruction arguments, but expected to be one.
    ///
    /// Patterns format matches the patterns indicated in the Arm Architecture Reference Manual,
    /// making easy to define them with little room for mistakes.
    ///
    /// Patterns can have 16 or 32 symbols. If not, the instruction decoder is expected to panic.
    ///
    /// For instance, the pattern "0001110xxxxxxxxx" matches the ADD instruction T1 encoding.
    pub expression: &'static str,
}

/// Possible qualifiers used in instruction mnemonics.
/// Returned by [Instruction::qualifier].
pub enum Qualifier {
    /// Mnemonic has no qualifier.
    None,
    /// Mnemonic has ".n" narrow qualifier.
    Narrow,
    /// Mnemonic has ".w" wide qualifier.
    Wide,
}

/// All instructions must implement this trait in order to be integrated into the emulator.
pub trait Instruction {
    /// Returns a list patterns the instruction can match. Each pattern is defined by its encoding
    /// index, the architectures supporting the instruction/encoding, and a regular expression for
    /// matching the bytes to be decoded.
    fn patterns() -> &'static [Pattern]
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
    /// # use armagnac::instructions::DecodeHelper;
    /// # use armagnac::instructions::unpredictable;
    /// # use armagnac::decoder::DecodeError;
    /// # let ins = 0u32;
    /// let rd = ins.reg4(0);
    /// unpredictable(rd.is_sp_or_pc())?;
    /// # Ok::<(), DecodeError>(())
    /// ```
    fn try_decode(encoding: Encoding, ins: u32, state: ItState) -> Result<Self, DecodeError>
    where
        Self: Sized;

    /// Returns instruction execution own condition, or [None] if the condition in the IT state is
    /// to be followed. Only the B instruction in T1 and T3 encodings, and BKPT instruction, should
    /// implement this, all other instructions rely on the blanket implementation which returns
    /// [None].
    fn condition(&self) -> Option<Condition> {
        None
    }

    /// Execute the instruction and updates given `proc` processor state.
    ///
    /// The instruction may request special treatment by returning some effect. In particular,
    /// instructions modifying the PC register shall return `Ok(Effect::Branch)` to prevent PC
    /// being automatically incremented following instruction execution.
    ///
    /// The execution of an instruction may also result in an error, for instance if an invalid
    /// memory access is performed.
    fn execute(&self, proc: &mut ArmProcessor) -> Result<Effect, RunError>;

    /// Returns the name of the instruction, in lowercase, to be shown in its mnemonic.
    ///
    /// For some instructions, the name can be dynamic. A good example is the Branch instruction
    /// which includes the branching condition (e.g. "beq", "bne", ...).
    ///
    /// Returned string does not include "s" suffix when instruction updates the condition flags,
    /// and does not include optional qualifier suffixes (typically ".w" wide qualifier).
    fn name(&self) -> String;

    /// Returns assembly qualifiers, such as "n" (narrow) or "w" (wide). This is used for printing
    /// instruction mnemonic.
    fn qualifier(&self) -> Qualifier {
        Qualifier::None
    }

    /// Returns whether this instruction updates flags or not. This is used for printing
    /// instruction mnemonic.
    ///
    /// Blanket implementation returns false.
    fn sets_flags(&self) -> bool {
        false
    }

    /// Formats and returns arguments strings to be shown in the instruction mnemonic.
    ///
    /// Here is a typical example:
    /// ```
    /// # use armagnac::registers::RegisterIndex;
    /// # struct Demo {
    /// #     rd: RegisterIndex,
    /// #     rn: RegisterIndex
    /// # }
    /// # impl Demo {
    /// #     fn args(&self) -> String {
    /// format!("{}, {}", self.rd, self.rn)
    /// #     }
    /// # }
    /// ```
    ///
    /// The current PC value is passed as parameter for instructions whose effect is relative to
    /// the PC value, though it can be ignored otherwise.
    ///
    /// Some formatting helper methods are provided and can be used: see [rdn_args_string],
    /// [indexing_args] or [crate::arith::Shift::arg_string].
    ///
    /// We have chosen to follow formatting output of `llvm-objdump` even when it might be
    /// different from the Arm Architecture Reference Manual. Doing this makes easier the
    /// generation of test vectors for formatting by relying on `llvm-objdump` as a reference
    /// we can trust. Currently, following disassembly options is passed to `llvm-objdump`:
    /// `--no-print-imm-hex`
    fn args(&self, pc: u32) -> String;
}

/// Possible instruction encodings.
///
/// An Arm instruction can have multiple different encodings, denoted in the reference manual as
/// 'T1', 'T2', 'A1', etc... During instruction decoding, a first pass determines to which encoding
/// an instruction corresponds to. Then the proper decoding routine which extracts and decodes
/// the instruction arguments, is called.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Encoding {
    T1,
    T2,
    T3,
    T4,
    A1,
    DontCare,
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
    /// This is used by the instruction decoder.
    pub fn from_halfword(halfword: u16) -> Self {
        match (halfword & 0xf800) >> 11 {
            0b11101..=0b11111 => InstructionSize::Ins32,
            _ => InstructionSize::Ins16,
        }
    }

    /// Size of instruction in bits.
    pub fn bit_count(&self) -> usize {
        (*self as usize) * 8
    }

    /// Size of instruction in bytes.
    pub fn byte_count(&self) -> usize {
        *self as usize
    }
}

/// Trait to facilitate fields and arguments extraction from an instruction encoding.
pub trait DecodeHelper {
    /// Extracts a 3-bit register index at `lsb_index` bit position.
    fn reg3(&self, lsb_index: u8) -> RegisterIndex;

    /// Extracts a 4-bit register index at `lsb_index` bit position.
    fn reg4(&self, lsb_index: u8) -> RegisterIndex;

    /// Extracts a single bit integer value at `lsb_index` bit position.
    fn imm1(&self, lsb_index: u8) -> u32;

    /// Extracts a 2-bit integer value at `lsb_index` bit position.
    fn imm2(&self, lsb_index: u8) -> u32;

    /// Extracts a 3-bit integer value at `lsb_index` bit position.
    fn imm3(&self, lsb_index: u8) -> u32;

    /// Extracts a 4-bit integer value at `lsb_index` bit position.
    fn imm4(&self, lsb_index: u8) -> u32;

    /// Extracts a 5-bit integer value at `lsb_index` bit position.
    fn imm5(&self, lsb_index: u8) -> u32;

    /// Extracts a 7-bit integer value at `lsb_index` bit position.
    fn imm7(&self, lsb_index: u8) -> u32;

    /// Extracts a 8-bit integer value at `lsb_index` bit position.
    fn imm8(&self, lsb_index: u8) -> u32;

    /// Extracts a 12-bit integer value at `lsb_index` bit position.
    fn imm12(&self, lsb_index: u8) -> u32;

    /// Extracts `index`, `add` and `wback` flags respectively from bits 10, 9 and 8.
    fn puw(&self) -> (bool, bool, bool);
}

impl DecodeHelper for u32 {
    fn reg3(&self, lsb_index: u8) -> RegisterIndex {
        RegisterIndex::new_main((self >> lsb_index) & 0x7)
    }

    fn reg4(&self, lsb_index: u8) -> RegisterIndex {
        RegisterIndex::new_main((self >> lsb_index) & 0xf)
    }

    fn imm1(&self, lsb_index: u8) -> u32 {
        (self >> lsb_index) & 1
    }

    fn imm2(&self, lsb_index: u8) -> u32 {
        (self >> lsb_index) & 3
    }

    fn imm3(&self, lsb_index: u8) -> u32 {
        (self >> lsb_index) & 7
    }

    fn imm4(&self, lsb_index: u8) -> u32 {
        (self >> lsb_index) & 0xf
    }

    fn imm5(&self, lsb_index: u8) -> u32 {
        (self >> lsb_index) & 0x1f
    }

    fn imm7(&self, lsb_index: u8) -> u32 {
        (self >> lsb_index) & 0x7f
    }

    fn imm8(&self, lsb_index: u8) -> u32 {
        (self >> lsb_index) & 0xff
    }

    fn imm12(&self, lsb_index: u8) -> u32 {
        (self >> lsb_index) & 0xfff
    }

    fn puw(&self) -> (bool, bool, bool) {
        (
            self & (1 << 10) != 0,
            self & (1 << 9) != 0,
            self & (1 << 8) != 0,
        )
    }
}

#[macro_export]
macro_rules! qualifier_wide_match {
    ($tn:expr, $tns:pat) => {
        match $tn {
            $tns => crate::instructions::Qualifier::Wide,
            _ => crate::instructions::Qualifier::None,
        }
    };
}

/// Returns "{rd}" if cond is `true`, else "{rd}, {rn}".
/// Panics if `cond` is `true` but `rd` and `rn` differ.
///
/// This is a convenient method used for many instruction arguments formatting.
pub fn rdn_args_string(rd: RegisterIndex, rn: RegisterIndex, cond: bool) -> String {
    if cond {
        assert_eq!(rd, rn);
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
///
/// If `imm` is null and `imm_explicit` is false, then the immediate offset is not printed.
pub fn indexing_args(
    rn: RegisterIndex,
    imm: u32,
    imm_explicit: bool,
    index: bool,
    add: bool,
    wback: bool,
) -> String {
    let neg = if add { "" } else { "-" };
    match (index, wback) {
        (true, false) => {
            let imm = if (imm != 0) || imm_explicit {
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
    /// Returns `self + rhs` if `add` is `true`, `self - rhs` otherwise.
    fn add_or_sub(&self, rhs: Self, add: bool) -> Self;

    /// Returns `self.wrapping_add(rhs)` if `add` is `true`, `self.wrapping_sub(rhs)` otherwise.
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

pub trait Mnemonic {
    fn mnemonic(&self, pc: u32, cond: Option<Condition>) -> String;
}

impl Mnemonic for Rc<dyn Instruction> {
    fn mnemonic(&self, pc: u32, cond: Option<Condition>) -> String {
        let mut name = self.name();
        if self.sets_flags() {
            name += "s";
        }
        if let Some(cond) = cond {
            name += &cond.to_string()
        }
        match self.qualifier() {
            Qualifier::None => {}
            Qualifier::Narrow => name += ".n",
            Qualifier::Wide => name += ".w",
        }
        let args = self.args(pc);
        if args.len() > 0 {
            format!("{:<8} {}", name, self.args(pc))
        } else {
            name
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        instructions::{indexing_args, rdn_args_string, DecodeHelper, InstructionSize},
        registers::RegisterIndex,
    };

    #[test]
    fn test_decode_helper() {
        // Test reg3
        for i in 0..=7 {
            for j in 0..=29 {
                assert_eq!((i << j).reg3(j), RegisterIndex::new_main(i));
            }
        }

        // Test reg4
        for i in 0..=15 {
            for j in 0..=28 {
                assert_eq!((i << j).reg4(j), RegisterIndex::new_main(i));
            }
        }

        // Test imm1
        for i in 0..=1 {
            for j in 0..=31 {
                assert_eq!((i << j).imm1(j), i);
            }
        }

        // Test imm2
        for i in 0..=3 {
            for j in 0..=30 {
                assert_eq!((i << j).imm2(j), i);
            }
        }

        // Test imm3
        for i in 0..=7 {
            for j in 0..=29 {
                assert_eq!((i << j).imm3(j), i);
            }
        }

        // Test imm4
        for i in 0..=15 {
            for j in 0..=28 {
                assert_eq!((i << j).imm4(j), i);
            }
        }

        // Test imm5
        for i in 0..=31 {
            for j in 0..=27 {
                assert_eq!((i << j).imm5(j), i);
            }
        }

        // Test imm8
        for i in 0..=255 {
            for j in 0..=24 {
                assert_eq!((i << j).imm8(j), i);
            }
        }

        // Test imm12
        for i in 0..=4095 {
            for j in 0..=20 {
                assert_eq!((i << j).imm12(j), i);
            }
        }

        // Test puw
        for i in 0..7 {
            assert_eq!(
                (i << 8).puw(),
                (i & 0b100 != 0, i & 0b010 != 0, i & 0b001 != 0)
            )
        }
    }

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

        assert_eq!(InstructionSize::Ins16.bit_count(), 16);
        assert_eq!(InstructionSize::Ins32.bit_count(), 32);
    }

    #[test]
    fn test_rdn_args_string() {
        assert_eq!(
            rdn_args_string(RegisterIndex::R0, RegisterIndex::R0, true),
            "r0"
        );
        assert_eq!(
            rdn_args_string(RegisterIndex::R0, RegisterIndex::R0, false),
            "r0, r0"
        );
        assert_eq!(
            rdn_args_string(RegisterIndex::R0, RegisterIndex::R1, false),
            "r0, r1"
        );
    }

    #[test]
    fn test_indexing_args() {
        assert_eq!(
            indexing_args(RegisterIndex::R1, 0, false, true, true, false),
            "[r1]"
        );
        assert_eq!(
            indexing_args(RegisterIndex::R1, 0, false, true, false, false),
            "[r1]"
        );
        assert_eq!(
            indexing_args(RegisterIndex::R1, 0, true, true, true, false),
            "[r1, #0]"
        );
        assert_eq!(
            indexing_args(RegisterIndex::R1, 12, false, true, true, false),
            "[r1, #12]"
        );
        assert_eq!(
            indexing_args(RegisterIndex::R1, 12, false, true, false, false),
            "[r1, #-12]"
        );

        assert_eq!(
            indexing_args(RegisterIndex::R1, 0, false, true, true, true),
            "[r1, #0]!"
        );
        assert_eq!(
            indexing_args(RegisterIndex::R1, 0, false, true, false, true),
            "[r1, #-0]!"
        );
        assert_eq!(
            indexing_args(RegisterIndex::R1, 12, false, true, true, true),
            "[r1, #12]!"
        );
        assert_eq!(
            indexing_args(RegisterIndex::R1, 12, false, true, false, true),
            "[r1, #-12]!"
        );

        assert_eq!(
            indexing_args(RegisterIndex::R1, 0, false, false, true, true),
            "[r1], #0"
        );
        assert_eq!(
            indexing_args(RegisterIndex::R1, 0, false, false, false, true),
            "[r1], #-0"
        );
        assert_eq!(
            indexing_args(RegisterIndex::R1, 12, false, false, true, true),
            "[r1], #12"
        );
        assert_eq!(
            indexing_args(RegisterIndex::R1, 12, false, false, false, true),
            "[r1], #-12"
        );
    }
}

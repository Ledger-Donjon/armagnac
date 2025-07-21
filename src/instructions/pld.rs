//! Implements PLD (Preload Data) instructions.
//!
//! Emulation of this instruction has no effect and is similar to NOP instruction.

use super::{
    Encoding::{self, T1, T2},
    Pattern,
};
use crate::{
    arith::Shift,
    core::{
        ArmVersion::{V7EM, V7M, V8M},
        Effect, ItState, Processor, RunError,
    },
    decoder::DecodeError,
    helpers::BitAccess,
    instructions::{indexing_args, other, unpredictable, DecodeHelper, Instruction},
    registers::RegisterIndex,
};

/// PLD (immediate) instruction.
///
/// Preload Data.
pub struct PldImm {
    /// Base register.
    rn: RegisterIndex,
    /// True to add offset, false to subtract.
    add: bool,
    /// Immediate offset applied to Rn.
    imm32: u32,
    /// Encoding.
    encoding: Encoding,
}

impl Instruction for PldImm {
    fn patterns() -> &'static [Pattern] {
        &[
            Pattern {
                encoding: T1,
                versions: &[V7M, V7EM, V8M],
                expression: "111110001001xxxx1111xxxxxxxxxxxx",
            },
            Pattern {
                encoding: T2,
                versions: &[V7M, V7EM, V8M],
                expression: "111110000001xxxx11111100xxxxxxxx",
            },
        ]
    }

    fn try_decode(encoding: Encoding, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        let rn = ins.reg4(16);
        other(rn.is_pc())?; // PLD (literal)
        Ok(match encoding {
            T1 => Self {
                rn,
                add: true,
                imm32: ins.imm12(0),
                encoding,
            },
            T2 => Self {
                rn,
                add: false,
                imm32: ins.imm8(0),
                encoding,
            },
            _ => panic!(),
        })
    }

    fn execute(&self, _proc: &mut Processor) -> Result<Effect, RunError> {
        Ok(Effect::None)
    }

    fn name(&self) -> String {
        "pld".into()
    }

    fn args(&self, _pc: u32) -> String {
        // #0 and #-0 offsets generate different instructions. Therefore for encoding T2 we want
        // the sign to be explicit.
        indexing_args(
            self.rn,
            self.imm32,
            self.encoding == T2,
            true,
            self.add,
            false,
        )
    }
}

/// PLD (literal) instruction.
///
/// Preload Data.
pub struct PldLit {
    /// Label offset.
    imm32: u32,
    /// True to add offset, false to subtract.
    add: bool,
}

impl Instruction for PldLit {
    fn patterns() -> &'static [Pattern] {
        &[Pattern {
            encoding: T1,
            versions: &[V7M, V7EM, V8M],
            expression: "11111000x00111111111xxxxxxxxxxxx",
        }]
    }

    fn try_decode(encoding: Encoding, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        debug_assert_eq!(encoding, T1);
        Ok(Self {
            imm32: ins.imm12(0),
            add: ins.bit(23),
        })
    }

    fn execute(&self, _proc: &mut Processor) -> Result<Effect, RunError> {
        Ok(Effect::None)
    }

    fn name(&self) -> String {
        "pld".into()
    }

    fn args(&self, _pc: u32) -> String {
        indexing_args(RegisterIndex::Pc, self.imm32, true, true, self.add, false)
    }
}

/// PLD (register) instruction.
///
/// Preload Data.
pub struct PldReg {
    /// Base register.
    rn: RegisterIndex,
    /// Offset register.
    rm: RegisterIndex,
    /// Shift to be applied to Rm.
    shift: Shift,
}

impl Instruction for PldReg {
    fn patterns() -> &'static [Pattern] {
        &[Pattern {
            encoding: T1,
            versions: &[V7M, V7EM, V8M],
            expression: "111110000001xxxx1111000000xxxxxx",
        }]
    }

    fn try_decode(encoding: Encoding, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        debug_assert_eq!(encoding, T1);
        let rn = ins.reg4(16);
        let rm = ins.reg4(0);
        other(rn.is_pc())?; // PLD (literal)
        unpredictable(rm.is_sp_or_pc())?;
        Ok(Self {
            rn,
            rm,
            shift: Shift::lsl(ins.imm2(4)),
        })
    }

    fn execute(&self, _proc: &mut Processor) -> Result<Effect, RunError> {
        Ok(Effect::None)
    }

    fn name(&self) -> String {
        "pld".into()
    }

    fn args(&self, _pc: u32) -> String {
        format!("[{}, {}{}]", self.rn, self.rm, self.shift.arg_string())
    }
}

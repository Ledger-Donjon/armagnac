//! Implements PLI (Preload Instruction) instruction.

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

use super::{
    Encoding::{self, T1, T2, T3},
    Pattern,
};

/// PLI (immediate, literal) instruction.
///
/// Preload Instruction.
pub struct PliImmLit {
    /// Base register.
    rn: RegisterIndex,
    /// Offset from base.
    imm32: u32,
    /// True to add offset, false to subtract.
    add: bool,
    /// Encoding.
    encoding: Encoding,
}

impl Instruction for PliImmLit {
    fn patterns() -> &'static [Pattern] {
        &[
            Pattern {
                encoding: T1,
                versions: &[V7M, V7EM, V8M],
                expression: "111110011001xxxx1111xxxxxxxxxxxx",
            },
            Pattern {
                encoding: T2,
                versions: &[V7M, V7EM, V8M],
                expression: "111110010001xxxx11111100xxxxxxxx",
            },
            Pattern {
                encoding: T3,
                versions: &[V7M, V7EM, V8M],
                expression: "11111001x00111111111xxxxxxxxxxxx",
            },
        ]
    }

    fn try_decode(encoding: Encoding, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        Ok(match encoding {
            T1 => {
                let rn = ins.reg4(16);
                other(rn.is_pc())?; // Encoding T3
                Self {
                    rn,
                    imm32: ins.imm12(0),
                    add: true,
                    encoding,
                }
            }
            T2 => {
                let rn = ins.reg4(16);
                other(rn.is_pc())?; // Encoding T3
                Self {
                    rn,
                    imm32: ins.imm8(0),
                    add: false,
                    encoding,
                }
            }
            T3 => Self {
                rn: RegisterIndex::Pc,
                imm32: ins.imm12(0),
                add: ins.bit(23),
                encoding,
            },
            _ => panic!(),
        })
    }

    fn execute(&self, _proc: &mut Processor) -> Result<Effect, RunError> {
        Ok(Effect::None)
    }

    fn name(&self) -> String {
        "pli".into()
    }

    fn args(&self, _pc: u32) -> String {
        indexing_args(
            self.rn,
            self.imm32,
            matches!(self.encoding, T2 | T3),
            true,
            self.add,
            false,
        )
    }
}

/// PLI (register) instruction.
///
/// Preload Instruction.
pub struct PliReg {
    /// Base register.
    rn: RegisterIndex,
    /// Offset register.
    rm: RegisterIndex,
    /// Shift to be applied to Rm.
    shift: Shift,
}

impl Instruction for PliReg {
    fn patterns() -> &'static [Pattern] {
        &[Pattern {
            encoding: T1,
            versions: &[V7M, V7EM, V8M],
            expression: "111110010001xxxx1111000000xxxxxx",
        }]
    }

    fn try_decode(encoding: Encoding, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        debug_assert_eq!(encoding, T1);
        let rn = ins.reg4(16);
        let rm = ins.reg4(0);
        other(rn.is_pc())?; // PLI (immediate, literal)
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
        "pli".into()
    }

    fn args(&self, _pc: u32) -> String {
        format!("[{}, {}{}]", self.rn, self.rm, self.shift.arg_string())
    }
}

//! Implements MUL (Multiply) instruction.

use super::{unpredictable, DecodeHelper, Instruction};
use super::{
    ArmVersion::{V6M, V7M, V8M},
    Pattern,
};
use crate::{
    arm::{ArmProcessor, RunError},
    decoder::DecodeError,
    it_state::ItState,
    registers::RegisterIndex,
};

/// MUL instruction.
pub struct Mul {
    /// Destination register.
    rd: RegisterIndex,
    /// First operand register.
    rn: RegisterIndex,
    /// Second operand register.
    rm: RegisterIndex,
    /// True if condition flags are updated.
    set_flags: bool,
}

impl Instruction for Mul {
    fn patterns() -> &'static [Pattern] {
        &[
            Pattern {
                tn: 1,
                versions: &[V6M, V7M, V8M],
                expression: "0100001101xxxxxx",
            },
            Pattern {
                tn: 2,
                versions: &[V7M, V8M],
                expression: "111110110000xxxx1111xxxx0000xxxx",
            },
        ]
    }

    fn try_decode(tn: usize, ins: u32, state: ItState) -> Result<Self, DecodeError> {
        Ok(match tn {
            1 => {
                let rdm = ins.reg3(0);
                Self {
                    rd: rdm,
                    rn: ins.reg3(3),
                    rm: rdm,
                    set_flags: !state.in_it_block(),
                }
            }
            2 => {
                let rd = ins.reg4(8);
                let rn = ins.reg4(16);
                let rm = ins.reg4(0);
                unpredictable(rd.is_sp_or_pc() || rn.is_sp_or_pc() || rm.is_sp_or_pc())?;
                Self {
                    rd,
                    rn,
                    rm,
                    set_flags: false,
                }
            }
            _ => panic!(),
        })
    }

    fn execute(&self, proc: &mut ArmProcessor) -> Result<bool, RunError> {
        let op1 = proc[self.rn] as i32;
        let op2 = proc[self.rm] as i32;
        let result = op1.wrapping_mul(op2) as u32;
        proc.set(self.rd, result);
        if self.set_flags {
            proc.registers.psr.set_nz(result);
        }
        Ok(false)
    }

    fn name(&self) -> String {
        "mul".into()
    }

    fn sets_flags(&self) -> bool {
        self.set_flags
    }

    fn args(&self, _pc: u32) -> String {
        format!("{}, {}, {}", self.rd, self.rn, self.rm)
    }
}

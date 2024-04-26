//! Implements MUL instruction.

use crate::{
    arm::{Arm7Processor, RunError},
    decoder::DecodeError,
    instructions::rdn_args_string,
    it_state::ItState,
    registers::RegisterIndex,
};

use super::{unpredictable, DecodeHelper, Instruction};

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
    fn patterns() -> &'static [&'static str] {
        &["0100001101xxxxxx", "111110110000xxxx1111xxxx0000xxxx"]
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

    fn execute(&self, proc: &mut Arm7Processor) -> Result<bool, RunError> {
        let op1 = proc[self.rn].val() as i32;
        let op2 = proc[self.rm].val() as i32;
        let result = op1.wrapping_mul(op2) as u32;
        proc[self.rd].set_val(result);
        if self.set_flags {
            proc.registers.apsr.set_nz(result);
        }
        Ok(false)
    }

    fn name(&self) -> String {
        if self.set_flags { "muls" } else { "mul" }.into()
    }

    fn args(&self, _pc: u32) -> String {
        format!("{}, {}", rdn_args_string(self.rd, self.rn), self.rm)
    }
}

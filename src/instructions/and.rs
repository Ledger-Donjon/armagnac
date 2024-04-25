//! Implements AND (immediate) instruction.

use crate::{
    arith::thumb_expand_imm_optc,
    arm::{Arm7Processor, RunError},
    decoder::DecodeError,
    instructions::rdn_args_string,
    it_state::ItState,
    registers::RegisterIndex,
};

use super::{other, reg, unpredictable, Instruction};

/// AND immediate instruction.
pub struct AndImm {
    /// Destination register.
    rd: RegisterIndex,
    /// First operand register.
    rn: RegisterIndex,
    /// Second operand.
    imm32: u32,
    /// True if condition flags are updated.
    set_flags: bool,
    /// Carry.
    carry: Option<bool>,
}

impl Instruction for AndImm {
    fn patterns() -> &'static [&'static str] {
        &["11110x00000xxxxx0xxxxxxxxxxxxxxx"]
    }

    fn try_decode(tn: usize, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        Ok(match tn {
            1 => {
                let rd = reg(ins >> 8 & 0xf);
                let rn = reg(ins >> 16 & 0xf);
                let imm12 = ((ins >> 26 & 1) << 11) | ((ins >> 12 & 7) << 8) | (ins & 0xff);
                let (imm32, carry) = thumb_expand_imm_optc(imm12)?;
                let set_flags = ins >> 20 & 1 != 0;
                other(rd.is_pc() && set_flags)?; // TST (immediate)
                unpredictable(rd.is_sp_or_pc() || rn.is_sp_or_pc())?;
                Self {
                    rd,
                    rn,
                    imm32,
                    set_flags: ins >> 20 & 1 != 0,
                    carry,
                }
            }
            _ => panic!(),
        })
    }

    fn execute(&self, proc: &mut Arm7Processor) -> Result<bool, RunError> {
        let result = proc.registers[self.rn].val() & self.imm32;
        proc.registers[self.rd].set_val(result);
        if self.set_flags {
            proc.registers.apsr.set_nz(result).set_c_opt(self.carry);
        }
        Ok(false)
    }

    fn name(&self) -> String {
        if self.set_flags { "ands" } else { "and" }.into()
    }

    fn args(&self, _pc: u32) -> String {
        format!("{}, #{}", rdn_args_string(self.rd, self.rn), self.imm32)
    }
}

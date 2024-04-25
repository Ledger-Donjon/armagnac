//! Implements BIC (immediate) and BIC (register) instructions.

use crate::{
    arith::{shift_c, thumb_expand_imm_optc, Shift},
    arm::{Arm7Processor, RunError},
    decoder::DecodeError,
    instructions::rdn_args_string,
    it_state::ItState,
    registers::RegisterIndex,
};

use super::{reg, unpredictable, Instruction};

/// BIC (immediate) instruction.
pub struct BicImm {
    /// Destination register.
    rd: RegisterIndex,
    /// First operand register.
    rn: RegisterIndex,
    /// Second operand.
    imm32: u32,
    /// Carry.
    carry: Option<bool>,
    /// True if condition flags are updated.
    set_flags: bool,
}

impl Instruction for BicImm {
    fn patterns() -> &'static [&'static str] {
        &["11110x00001xxxxx0xxxxxxxxxxxxxxx"]
    }

    fn try_decode(tn: usize, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        Ok(match tn {
            1 => {
                let rd = reg(ins >> 8 & 0xf);
                let rn = reg(ins >> 16 & 0xf);
                unpredictable(rd.is_sp_or_pc() || rn.is_sp_or_pc())?;
                let imm12 = (ins >> 26 & 1) << 11 | (ins >> 12 & 7) << 8 | ins & 0xff;
                let (imm32, carry) = thumb_expand_imm_optc(imm12)?;
                Self {
                    rd,
                    rn,
                    imm32,
                    carry,
                    set_flags: ins >> 20 & 1 != 0,
                }
            }
            _ => panic!(),
        })
    }

    fn execute(&self, proc: &mut Arm7Processor) -> Result<bool, RunError> {
        let result = proc.registers[self.rn].val() & !self.imm32;
        proc.registers[self.rd].set_val(result);
        if self.set_flags {
            proc.registers.apsr.set_nz(result).set_c_opt(self.carry);
        }
        Ok(false)
    }

    fn name(&self) -> String {
        if self.set_flags { "bics" } else { "bic" }.into()
    }

    fn args(&self, _pc: u32) -> String {
        format!("{}, #{}", rdn_args_string(self.rd, self.rn), self.imm32)
    }
}

/// BIC (register) instruction.
pub struct BicReg {
    /// Destination register.
    rd: RegisterIndex,
    /// First operand register.
    rn: RegisterIndex,
    /// Second operand register.
    rm: RegisterIndex,
    /// Shift to be applied to Rm.
    shift: Shift,
    /// True if condition flags are updated.
    set_flags: bool,
}

impl Instruction for BicReg {
    fn patterns() -> &'static [&'static str] {
        &["0100001110xxxxxx", "11101010001xxxxx(0)xxxxxxxxxxxxxxx"]
    }

    fn try_decode(tn: usize, ins: u32, state: ItState) -> Result<Self, DecodeError> {
        Ok(match tn {
            1 => {
                let rdn = reg(ins & 7);
                Self {
                    rd: rdn,
                    rn: rdn,
                    rm: reg(ins >> 3 & 7),
                    shift: Shift::lsl(0),
                    set_flags: !state.in_it_block(),
                }
            }
            2 => {
                let rd = reg(ins >> 8 & 0xf);
                let rn = reg(ins >> 16 & 0xf);
                let rm = reg(ins & 0xf);
                unpredictable(rd.is_sp_or_pc() || rn.is_sp_or_pc() || rm.is_sp_or_pc())?;
                let shift = Shift::from_bits(ins >> 4 & 3, (ins >> 12 & 7) << 2 | ins >> 6 & 3);
                Self {
                    rd,
                    rn,
                    rm,
                    shift,
                    set_flags: ins >> 20 & 1 != 0,
                }
            }
            _ => panic!(),
        })
    }

    fn execute(&self, proc: &mut crate::arm::Arm7Processor) -> Result<bool, RunError> {
        let carry_in = proc.registers.apsr.c();
        let (shifted, carry) = shift_c(proc.registers[self.rm].val(), self.shift, carry_in);
        let result = proc.registers[self.rn].val() & !shifted;
        proc.registers[self.rd].set_val(result);
        if self.set_flags {
            proc.registers.apsr.set_nz(result).set_c(carry);
        }
        Ok(false)
    }

    fn name(&self) -> String {
        if self.set_flags { "bics" } else { "bic" }.into()
    }

    fn args(&self, _pc: u32) -> String {
        format!(
            "{}, {}{}",
            rdn_args_string(self.rd, self.rn),
            self.rm,
            self.shift.arg_string()
        )
    }
}

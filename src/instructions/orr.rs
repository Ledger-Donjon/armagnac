//! Implements ORR (immediate) and ORR (register) instructions.

use core::panic;

use crate::{
    arith::{shift_c, thumb_expand_imm_optc, Shift},
    arm::{Arm7Processor, RunError},
    decoder::DecodeError,
    instructions::{other, rdn_args_string, reg, unpredictable, ItState},
    registers::RegisterIndex,
};

use super::Instruction;

/// ORR (immediate) instruction.
pub struct OrrImm {
    /// Destination register.
    rd: RegisterIndex,
    /// First operand register.
    rn: RegisterIndex,
    /// Second operand value.
    imm32: u32,
    /// Carry.
    carry: Option<bool>,
    /// True if condition flags are updated.
    set_flags: bool,
}

impl Instruction for OrrImm {
    fn patterns() -> &'static [&'static str] {
        &["11110x00010xxxxx0xxxxxxxxxxxxxxx"]
    }

    fn try_decode(tn: usize, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        debug_assert_eq!(tn, 1);
        let rd = reg(ins >> 8 & 0xf);
        let rn = reg(ins >> 16 & 0xf);
        other(rn.is_pc())?; // MOV (immediate)
        unpredictable(rd.is_sp_or_pc() || rn.is_sp())?;
        let imm12 = (ins >> 26 & 1) << 11 | (ins >> 12 & 7) << 8 | ins & 0xff;
        let (imm32, carry) = thumb_expand_imm_optc(imm12)?;
        Ok(Self {
            rd,
            rn,
            imm32,
            carry,
            set_flags: ins >> 20 & 1 != 0,
        })
    }

    fn execute(&self, proc: &mut Arm7Processor) -> Result<bool, RunError> {
        let result = proc.registers[self.rn] | self.imm32;
        proc.registers[self.rd] = result;
        if self.set_flags {
            proc.registers.apsr.set_nz(result).set_c_opt(self.carry);
        }
        Ok(false)
    }

    fn name(&self) -> String {
        if self.set_flags { "orrs" } else { "orr" }.into()
    }

    fn args(&self, _pc: u32) -> String {
        format!("{}, #{}", rdn_args_string(self.rd, self.rn), self.imm32)
    }
}

/// ORR (register) instruction.
pub struct OrrReg {
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

impl Instruction for OrrReg {
    fn patterns() -> &'static [&'static str] {
        &["0100001100xxxxxx", "11101010010xxxxx(0)xxxxxxxxxxxxxxx"]
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
                let shift = Shift::from_bits(ins >> 4 & 3, (ins >> 12 & 7) << 2 | ins >> 6 & 3);
                unpredictable(rd.is_sp_or_pc() || rn.is_sp() || rm.is_sp_or_pc())?;
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

    fn execute(&self, proc: &mut Arm7Processor) -> Result<bool, RunError> {
        let carry_in = proc.registers.apsr.c();
        let (shifted, carry) = shift_c(proc.registers[self.rm], self.shift, carry_in);
        let result = proc.registers[self.rn] | shifted;
        proc.registers[self.rd] = result;
        if self.set_flags {
            proc.registers.apsr.set_nz(result).set_c(carry);
        }
        Ok(false)
    }

    fn name(&self) -> String {
        if self.set_flags { "orrs" } else { "orr" }.into()
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

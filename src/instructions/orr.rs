//! Implements ORR (immediate) and ORR (register) instructions.

use core::panic;

use crate::{
    arith::{shift_c, thumb_expand_imm_optc, Shift},
    arm::{Arm7Processor, RunError},
    decoder::DecodeError,
    helpers::BitAccess,
    instructions::{other, rdn_args_string, unpredictable, DecodeHelper, ItState},
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
        let rd = ins.reg4(8);
        let rn = ins.reg4(16);
        other(rn.is_pc())?; // MOV (immediate)
        unpredictable(rd.is_sp_or_pc() || rn.is_sp())?;
        let imm12 = ins.imm1(26) << 11 | ins.imm3(12) << 8 | ins.imm8(0);
        let (imm32, carry) = thumb_expand_imm_optc(imm12)?;
        Ok(Self {
            rd,
            rn,
            imm32,
            carry,
            set_flags: ins.bit(20),
        })
    }

    fn execute(&self, proc: &mut Arm7Processor) -> Result<bool, RunError> {
        let result = proc.registers[self.rn] | self.imm32;
        proc.registers.set(self.rd, result);
        if self.set_flags {
            proc.registers.xpsr.set_nz(result).set_c_opt(self.carry);
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
                let rdn = ins.reg3(0);
                Self {
                    rd: rdn,
                    rn: rdn,
                    rm: ins.reg3(3),
                    shift: Shift::lsl(0),
                    set_flags: !state.in_it_block(),
                }
            }
            2 => {
                let rd = ins.reg4(8);
                let rn = ins.reg4(16);
                let rm = ins.reg4(0);
                other(rn.is_pc())?; // MOV (register)
                let shift = Shift::from_bits(ins.imm2(4), ins.imm3(12) << 2 | ins.imm2(6));
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
        let carry_in = proc.registers.xpsr.c();
        let (shifted, carry) = shift_c(proc.registers[self.rm], self.shift, carry_in);
        let result = proc.registers[self.rn] | shifted;
        proc.registers.set(self.rd, result);
        if self.set_flags {
            proc.registers.xpsr.set_nz(result).set_c(carry);
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

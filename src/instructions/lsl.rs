//! Implements LSL (immediate) and LSL (register) instructions.

use crate::{
    arith::{shift_c, Shift},
    arm::{Arm7Processor, RunError},
    decoder::DecodeError,
    instructions::rdn_args_string,
    it_state::ItState,
    registers::RegisterIndex,
};

use super::{other, reg, unpredictable, Instruction};

/// LSL (immediate) instruction.
pub struct LslImm {
    /// Destination register.
    rd: RegisterIndex,
    /// First operand register.
    rm: RegisterIndex,
    /// Shift to be applied to Rm.
    shift: u8,
    /// True if condition flags are updated.
    set_flags: bool,
}

impl Instruction for LslImm {
    fn patterns() -> &'static [&'static str] {
        &["00000xxxxxxxxxxx", "11101010010x1111(0)xxxxxxxxx00xxxx"]
    }

    fn try_decode(tn: usize, ins: u32, state: ItState) -> Result<Self, DecodeError> {
        Ok(match tn {
            1 => {
                let imm5 = ins >> 6 & 0x1f;
                other(imm5 == 0)?; // MOV (register)
                Self {
                    rd: reg(ins & 7),
                    rm: reg(ins >> 3 & 7),
                    shift: imm5 as u8,
                    set_flags: !state.in_it_block(),
                }
            }
            2 => {
                let rd = reg(ins >> 8 & 0xf);
                let rm = reg(ins & 0xf);
                let imm5 = ins >> 12 & 7 | ins >> 6 & 3;
                other(imm5 == 0)?; // MOV (register)
                unpredictable(rd.is_sp_or_pc() || rm.is_sp_or_pc())?;
                Self {
                    rd,
                    rm,
                    shift: imm5 as u8,
                    set_flags: ins >> 20 & 1 != 0,
                }
            }
            _ => panic!(),
        })
    }

    fn execute(&self, proc: &mut Arm7Processor) -> Result<bool, RunError> {
        let carry_in = proc.registers.xpsr.c();
        let shift = Shift::lsl(self.shift as u32);
        let (result, c) = shift_c(proc.registers[self.rm], shift, carry_in);
        proc.registers.set(self.rd, result);
        if self.set_flags {
            proc.registers.xpsr.set_nz(result).set_c(c);
        }
        Ok(false)
    }

    fn name(&self) -> String {
        if self.set_flags { "lsls" } else { "lsl" }.into()
    }

    fn args(&self, _pc: u32) -> String {
        format!("{}, {}, #{}", self.rd, self.rm, self.shift)
    }
}

/// LSL (register) instruction.
pub struct LslReg {
    /// Destination register.
    rd: RegisterIndex,
    /// First operand register.
    rn: RegisterIndex,
    /// Shift amount register.
    rm: RegisterIndex,
    /// True if condition flags are updated.
    set_flags: bool,
}

impl Instruction for LslReg {
    fn patterns() -> &'static [&'static str] {
        &["0100000010xxxxxx", "11111010000xxxxx1111xxxx0000xxxx"]
    }

    fn try_decode(tn: usize, ins: u32, state: ItState) -> Result<Self, DecodeError> {
        Ok(match tn {
            1 => {
                let rdn = reg(ins & 7);
                Self {
                    rd: rdn,
                    rn: rdn,
                    rm: reg(ins >> 3 & 7),
                    set_flags: !state.in_it_block(),
                }
            }
            2 => {
                let rd = reg(ins >> 8 & 0xf);
                let rn = reg(ins >> 16 & 0xf);
                let rm = reg(ins & 0xf);
                unpredictable(rd.is_sp_or_pc() || rn.is_sp_or_pc() || rm.is_sp_or_pc())?;
                Self {
                    rd,
                    rn,
                    rm,
                    set_flags: ins >> 20 & 1 != 0,
                }
            }
            _ => panic!(),
        })
    }

    fn execute(&self, proc: &mut Arm7Processor) -> Result<bool, RunError> {
        let shift_n = proc.registers[self.rm] & 0xff;
        let carry_in = proc.registers.xpsr.c();
        let shift = Shift::lsl(shift_n);
        let (result, c) = shift_c(proc.registers[self.rn], shift, carry_in);
        proc.registers.set(self.rd, result);
        if self.set_flags {
            proc.registers.xpsr.set_nz(result).set_c(c);
        }
        Ok(false)
    }

    fn name(&self) -> String {
        if self.set_flags { "lsls" } else { "lsl" }.into()
    }

    fn args(&self, _pc: u32) -> String {
        format!("{}, {}", rdn_args_string(self.rd, self.rn), self.rm)
    }
}

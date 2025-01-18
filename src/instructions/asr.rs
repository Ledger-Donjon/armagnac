use std::fmt::format;

use crate::{
    arith::{shift_c, Shift},
    arm::{Arm7Processor, RunError},
    decoder::DecodeError,
    helpers::BitAccess,
    it_state::ItState,
    registers::RegisterIndex,
};

use super::{rdn_args_string, unpredictable, DecodeHelper, Instruction};

/// ASR (immediate) instruction.
pub struct AsrImm {
    /// Destination register.
    rd: RegisterIndex,
    /// First operand register.
    rm: RegisterIndex,
    /// Shift to be applied to Rm.
    shift: u8,
    /// True if condition flags are updated.
    set_flags: bool,
}

impl Instruction for AsrImm {
    fn patterns() -> &'static [&'static str] {
        &["00010xxxxxxxxxxx", "11101010010x1111(0)xxxxxxxxx10xxxx"]
    }

    fn try_decode(tn: usize, ins: u32, state: ItState) -> Result<Self, DecodeError> {
        Ok(match tn {
            1 => Self {
                rd: ins.reg3(0),
                rm: ins.reg3(3),
                shift: Shift::from_bits(2, ins.imm5(6)).n as u8,
                set_flags: !state.in_it_block(),
            },
            2 => {
                let rm = ins.reg4(0);
                let rd = ins.reg4(8);
                unpredictable(rd.is_sp_or_pc() || rm.is_sp_or_pc())?;
                let imm5 = ins.imm3(12) << 2 | ins.imm2(6);
                Self {
                    rd,
                    rm,
                    shift: imm5 as u8,
                    set_flags: ins.bit(20),
                }
            }
            _ => panic!(),
        })
    }

    fn execute(&self, proc: &mut Arm7Processor) -> Result<bool, RunError> {
        let carry_in = proc.registers.xpsr.c();
        let (result, carry) = shift_c(
            proc.registers[self.rm],
            Shift::asr(self.shift as u32),
            carry_in,
        );
        proc.registers.set(self.rd, result);
        if self.set_flags {
            proc.registers.xpsr.set_nz(result).set_c(carry);
        }
        Ok(false)
    }

    fn name(&self) -> String {
        if self.set_flags { "asrs" } else { "asr" }.into()
    }

    fn args(&self, _pc: u32) -> String {
        format!("{}, {}, #{}", self.rd, self.rm, self.shift)
    }
}

/// ASR (register) instruction.
pub struct AsrReg {
    /// Destination register.
    rd: RegisterIndex,
    /// First operand register.
    rn: RegisterIndex,
    /// Shift amount register.
    rm: RegisterIndex,
    /// True if condition flags are updated.
    set_flags: bool,
}

impl Instruction for AsrReg {
    fn patterns() -> &'static [&'static str] {
        &["0100000100xxxxxx", "11111010010xxxxx1111xxxx0000xxxx"]
    }

    fn try_decode(tn: usize, ins: u32, state: ItState) -> Result<Self, DecodeError> {
        Ok(match tn {
            1 => {
                let rdn = ins.reg3(0);
                Self {
                    rd: rdn,
                    rn: rdn,
                    rm: ins.reg3(3),
                    set_flags: !state.in_it_block(),
                }
            }
            2 => {
                let rm = ins.reg4(0);
                let rd = ins.reg4(8);
                let rn = ins.reg4(16);
                unpredictable(rd.is_sp_or_pc() || rn.is_sp_or_pc() || rm.is_sp_or_pc())?;
                Self {
                    rd,
                    rn,
                    rm,
                    set_flags: ins.bit(20),
                }
            }
            _ => panic!(),
        })
    }

    fn execute(&self, proc: &mut Arm7Processor) -> Result<bool, RunError> {
        let shift_n = proc.registers[self.rm] & 0xff;
        let carry_in = proc.registers.xpsr.c();
        let (result, carry) = shift_c(proc.registers[self.rn], Shift::asr(shift_n), carry_in);
        proc.registers.set(self.rd, result);
        if self.set_flags {
            proc.registers.xpsr.set_nz(result).set_c(carry);
        }
        Ok(false)
    }

    fn name(&self) -> String {
        if self.set_flags { "asrs" } else { "asr" }.into()
    }

    fn args(&self, _pc: u32) -> String {
        format!("{}, {}", rdn_args_string(self.rd, self.rn), self.rm)
    }
}

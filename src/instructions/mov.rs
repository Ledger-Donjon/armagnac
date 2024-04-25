//! Implements MOV (immediate), MOV (register).

use core::panic;

use crate::{
    arith::thumb_expand_imm_optc,
    arm::{Arm7Processor, RunError},
    decoder::DecodeError,
    instructions::ItState,
    registers::RegisterIndex,
};

use super::{reg, unpredictable, DecodeHelper, Instruction};

/// MOV (immediate) instruction.
pub struct MovImm {
    /// Destination register.
    rd: RegisterIndex,
    /// Immediate value to be placed in Rd.
    imm32: u32,
    /// True if condition flags are updated.
    set_flags: bool,
    /// Carry.
    carry: Option<bool>,
}

impl Instruction for MovImm {
    fn patterns() -> &'static [&'static str] {
        &[
            "00100xxxxxxxxxxx",
            "11110x00010x11110xxxxxxxxxxxxxxx",
            "11110x100100xxxx0xxxxxxxxxxxxxxx",
        ]
    }

    fn try_decode(tn: usize, ins: u32, state: ItState) -> Result<Self, DecodeError> {
        Ok(match tn {
            1 => Self {
                rd: ins.reg3(8),
                imm32: ins.imm8(0),
                set_flags: !state.in_it_block(),
                carry: None,
            },
            2 => {
                let rd = ins.reg4(8);
                let imm12 = (ins >> 26 & 1) << 11 | (ins >> 12 & 7) << 8 | ins & 0xff;
                let (imm32, carry) = thumb_expand_imm_optc(imm12)?;
                unpredictable(rd.is_sp_or_pc())?;
                Self {
                    rd,
                    imm32,
                    set_flags: ins >> 20 & 1 != 0,
                    carry,
                }
            }
            3 => {
                let rd = ins.reg4(8);
                let imm32 = (ins >> 16 & 0xf) << 12
                    | (ins >> 26 & 1) << 11
                    | (ins >> 12 & 7) << 8
                    | ins & 0xff;
                unpredictable(rd.is_sp_or_pc())?;
                Self {
                    rd,
                    imm32,
                    set_flags: false,
                    carry: None,
                }
            }
            _ => panic!(),
        })
    }

    fn execute(&self, proc: &mut Arm7Processor) -> Result<bool, RunError> {
        proc.registers[self.rd].set_val(self.imm32);
        if self.set_flags {
            proc.registers.apsr.set_nz(self.imm32).set_c_opt(self.carry);
        }
        Ok(false)
    }

    fn name(&self) -> String {
        if self.set_flags { "movs" } else { "mov" }.into()
    }

    fn args(&self, _pc: u32) -> String {
        format!("{}, #{}", self.rd, self.imm32)
    }
}

/// MOV (register) instruction.
pub struct MovReg {
    /// Destination register.
    rd: RegisterIndex,
    /// Source register.
    rm: RegisterIndex,
    /// True if condition flags are updated.
    set_flags: bool,
}

impl Instruction for MovReg {
    fn patterns() -> &'static [&'static str] {
        &[
            "01000110xxxxxxxx",
            "0000000000xxxxxx",
            "11101010010x1111(0)000xxxx0000xxxx",
        ]
    }

    fn try_decode(tn: usize, ins: u32, state: ItState) -> Result<Self, DecodeError> {
        Ok(match tn {
            1 => {
                let rd = reg(ins & 7 | (ins >> 7 & 1) << 3);
                unpredictable(rd.is_pc() && state.in_it_block_not_last())?;
                Self {
                    rd,
                    rm: ins.reg4(3),
                    set_flags: false,
                }
            }
            2 => {
                unpredictable(state.in_it_block())?;
                Self {
                    rd: ins.reg3(0),
                    rm: ins.reg3(3),
                    set_flags: true,
                }
            }
            3 => {
                let rd = reg(ins >> 8 & 0xf);
                let rm = reg(ins & 0xf);
                let set_flags = ins >> 20 & 1 != 0;
                unpredictable(set_flags && (rd.is_sp_or_pc() || rm.is_sp_or_pc()))?;
                unpredictable(
                    !set_flags && (rd.is_pc() || rm.is_pc() || (rd.is_sp() && rm.is_sp())),
                )?;
                Self { rd, rm, set_flags }
            }
            _ => panic!(),
        })
    }

    fn execute(&self, proc: &mut Arm7Processor) -> Result<bool, RunError> {
        let result = proc.registers[self.rm].val();
        if self.rd.is_pc() {
            proc.alu_write_pc(result);
            Ok(true)
        } else {
            proc.registers[self.rd].set_val(result);
            if self.set_flags {
                proc.registers.apsr.set_nz(result);
            }
            Ok(false)
        }
    }

    fn name(&self) -> String {
        if self.set_flags { "movs" } else { "mov" }.into()
    }

    fn args(&self, _pc: u32) -> String {
        format!("{}, {}", self.rd, self.rm)
    }
}

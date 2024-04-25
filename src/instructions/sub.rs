//! Implements SUB (immediate), SUB (register), SUB (SP minus immediate) and SUB (SP minus
//! register) instructions.

use crate::{
    arith::{add_with_carry, shift_c, thumb_expand_imm, Shift},
    arm::{Arm7Processor, RunError},
    decoder::DecodeError,
    instructions::{rdn_args_string, ItState},
    registers::RegisterIndex,
};

use super::{other, reg, unpredictable, DecodeHelper, Instruction};

/// SUB (immediate) instruction.
pub struct SubImm {
    /// Destination register.
    rd: RegisterIndex,
    /// First operand register.
    rn: RegisterIndex,
    /// Value to be subtracted.
    imm32: u32,
    /// True if condition flags are updated.
    set_flags: bool,
}

impl Instruction for SubImm {
    fn patterns() -> &'static [&'static str] {
        &[
            "0001111xxxxxxxxx",
            "00111xxxxxxxxxxx",
            "11110x01101xxxxx0xxxxxxxxxxxxxxx",
            "11110x101010xxxx0xxxxxxxxxxxxxxx",
        ]
    }

    fn try_decode(tn: usize, ins: u32, state: ItState) -> Result<Self, DecodeError> {
        Ok(match tn {
            1 => Self {
                rd: reg(ins & 7),
                rn: reg(ins >> 3 & 7),
                imm32: ins >> 6 & 7,
                set_flags: !state.in_it_block(),
            },
            2 => {
                let rdn = reg(ins >> 8 & 7);
                Self {
                    rd: rdn,
                    rn: rdn,
                    imm32: ins & 0xff,
                    set_flags: !state.in_it_block(),
                }
            }
            3 => {
                let rd = reg(ins >> 8 & 0xf);
                let rn = reg(ins >> 16 & 0xf);
                let set_flags = ins >> 20 & 1 != 0;
                other(rd.is_pc() && set_flags)?; // CMP (immediate)
                other(rn.is_sp())?; // SUB (SP minus immediate)
                unpredictable(rd.is_sp_or_pc() || rn.is_pc())?;
                let imm12 = (ins >> 26 & 1) << 11 | (ins >> 12 & 7) << 8 | ins & 0xff;
                let imm32 = thumb_expand_imm(imm12)?;
                Self {
                    rd,
                    rn,
                    imm32,
                    set_flags,
                }
            }
            4 => {
                let rd = reg(ins >> 8 & 0xf);
                let rn = reg(ins >> 16 & 0xf);
                other(rn.is_pc())?; // ADR
                other(rn.is_sp())?; // SUB (SP minus immediate)
                unpredictable(rd.is_sp_or_pc())?;
                let imm32 = (ins >> 26 & 1) << 11 | (ins >> 12 & 7) << 8 | ins & 0xff;
                Self {
                    rd,
                    rn,
                    imm32,
                    set_flags: false,
                }
            }
            _ => panic!(),
        })
    }

    fn execute(&self, proc: &mut Arm7Processor) -> Result<bool, RunError> {
        let rn = proc.registers[self.rn].val();
        let (result, carry, overflow) = add_with_carry(rn, !self.imm32, true);
        proc.registers[self.rd].set_val(result);
        if self.set_flags {
            proc.registers
                .apsr
                .set_nz(result)
                .set_c(carry)
                .set_v(overflow);
        }
        Ok(false)
    }

    fn name(&self) -> String {
        if self.set_flags { "subs" } else { "sub" }.into()
    }

    fn args(&self, _pc: u32) -> String {
        format!("{}, #{}", rdn_args_string(self.rd, self.rn), self.imm32)
    }
}

/// SUB (register) instruction.
pub struct SubReg {
    /// Destination register.
    rd: RegisterIndex,
    /// First operand register.
    rn: RegisterIndex,
    /// Second operand register.
    rm: RegisterIndex,
    /// Shift to apply to Rm.
    shift: Shift,
    /// True if condition flags are updated.
    set_flags: bool,
}

impl Instruction for SubReg {
    fn patterns() -> &'static [&'static str] {
        &["0001101xxxxxxxxx", "11101011101xxxxx(0)xxxxxxxxxxxxxxx"]
    }

    fn try_decode(tn: usize, ins: u32, state: ItState) -> Result<Self, DecodeError> {
        Ok(match tn {
            1 => Self {
                rd: ins.reg3(0),
                rn: ins.reg3(3),
                rm: ins.reg3(6),
                shift: Shift::lsl(0),
                set_flags: !state.in_it_block(),
            },
            2 => {
                let rm = ins.reg4(0);
                let rd = ins.reg4(8);
                let rn = ins.reg4(16);
                let s = ins >> 20 & 1 != 0;
                other(rd.is_pc() && s)?; // CMP (register)
                other(rn.is_sp())?; // SUB (SP minus register)
                unpredictable(rd.is_sp_or_pc() || rn.is_pc() || rm.is_sp_or_pc())?;
                Self {
                    rd,
                    rn,
                    rm,
                    shift: Shift::from_bits(ins.imm2(4), ins.imm3(12) << 2 | ins.imm2(6)),
                    set_flags: s,
                }
            }
            _ => panic!(),
        })
    }

    fn execute(&self, proc: &mut Arm7Processor) -> Result<bool, RunError> {
        let rn = proc.registers[self.rn].val();
        let carry_in = proc.registers.apsr.c();
        let (shifted, _) = shift_c(proc.registers[self.rm].val(), self.shift, carry_in);
        let (result, carry, overflow) = add_with_carry(rn, !shifted, true);
        proc.registers[self.rd].set_val(result);
        if self.set_flags {
            proc.registers
                .apsr
                .set_nz(result)
                .set_c(carry)
                .set_v(overflow);
        }
        Ok(false)
    }

    fn name(&self) -> String {
        if self.set_flags { "subs" } else { "sub" }.into()
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

/// SUB (SP minus immediate) instruction.
pub struct SubSpMinusImm {
    /// Destination register.
    rd: RegisterIndex,
    /// Value to be subtracted.
    imm32: u32,
    /// True if condition flags are updated.
    set_flags: bool,
}

impl Instruction for SubSpMinusImm {
    fn patterns() -> &'static [&'static str] {
        &[
            "101100001xxxxxxx",
            "11110x01101x1101xxxxxxxxxxxxxxxx",
            "11110x10101011010xxxxxxxxxxxxxxx",
        ]
    }

    fn try_decode(tn: usize, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        Ok(match tn {
            1 => Self {
                rd: RegisterIndex::Sp,
                imm32: (ins & 0x7f) << 2,
                set_flags: false,
            },
            2 => {
                let rd = reg(ins >> 8 & 0xf);
                let imm12 = (ins >> 26 & 1) << 11 | (ins >> 12 & 7) << 8 | ins & 0xff;
                let imm32 = thumb_expand_imm(imm12)?;
                let set_flags = ins >> 20 & 1 != 0;
                other(rd.is_pc() && set_flags)?; // CMP (immediate)
                unpredictable(rd.is_pc())?;
                Self {
                    rd,
                    imm32,
                    set_flags: ins >> 20 & 1 != 0,
                }
            }
            3 => {
                let rd = reg(ins >> 8 & 0xf);
                let imm32 = (ins >> 26 & 1) << 11 | (ins >> 12 & 7) << 8 | ins & 0xff;
                unpredictable(rd.is_pc())?;
                Self {
                    rd,
                    imm32,
                    set_flags: ins >> 20 & 1 != 0,
                }
            }
            _ => panic!(),
        })
    }

    fn execute(&self, proc: &mut Arm7Processor) -> Result<bool, RunError> {
        let (result, carry, overflow) = add_with_carry(proc.sp(), !self.imm32, true);
        proc.registers[self.rd].set_val(result);
        if self.set_flags {
            proc.registers
                .apsr
                .set_nz(result)
                .set_c(carry)
                .set_v(overflow);
        }
        Ok(false)
    }

    fn name(&self) -> String {
        if self.set_flags { "subs" } else { "sub" }.into()
    }

    fn args(&self, _pc: u32) -> String {
        format!(
            "{}, #{}",
            rdn_args_string(self.rd, RegisterIndex::Pc),
            self.imm32
        )
    }
}

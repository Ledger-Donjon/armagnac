//! Implements ADD instruction.

use core::panic;

use crate::{
    arith::{add_with_carry, shift_c, thumb_expand_imm, Shift, ShiftType},
    arm::{Arm7Processor, RunError},
    decoder::DecodeError,
    helpers::BitAccess,
    instructions::rdn_args_string,
    it_state::ItState,
    registers::RegisterIndex,
};

use super::{other, unpredictable, DecodeHelper, Instruction};

/// ADD (immediate) instruction.
pub struct AddImm {
    /// Destination register.
    rd: RegisterIndex,
    /// First operand register.
    rn: RegisterIndex,
    /// Value to be added.
    imm32: u32,
    /// True if condition flags are updated.
    set_flags: bool,
}

impl Instruction for AddImm {
    fn patterns() -> &'static [&'static str] {
        &[
            "0001110xxxxxxxxx",
            "00110xxxxxxxxxxx",
            "11110x01000xxxxx0xxxxxxxxxxxxxxx",
            "11110x100000xxxx0xxxxxxxxxxxxxxx",
        ]
    }

    fn try_decode(tn: usize, ins: u32, state: ItState) -> Result<AddImm, DecodeError> {
        Ok(match tn {
            1 => Self {
                rd: ins.reg3(0),
                rn: ins.reg3(3),
                imm32: ins >> 6 & 7,
                set_flags: !state.in_it_block(),
            },
            2 => {
                let rdn = ins.reg3(8);
                Self {
                    rd: rdn,
                    rn: rdn,
                    imm32: ins & 0xff,
                    set_flags: !state.in_it_block(),
                }
            }
            3 => {
                let set_flags = ins >> 20 & 1 != 0;
                let rd = ins.reg4(8);
                let rn = ins.reg4(16);
                let imm12 = (ins >> 26 & 1) << 11 | (ins >> 12 & 7) << 8 | ins & 0xff;
                let imm32 = thumb_expand_imm(imm12)?;
                other(rd.is_pc() && set_flags)?;
                other(rn.is_sp())?; // ADD (SP plus immediate)
                unpredictable(rd.is_sp_or_pc() || rn.is_pc())?;
                Self {
                    rd,
                    rn,
                    imm32,
                    set_flags,
                }
            }
            4 => {
                let rd = ins.reg4(8);
                let rn = ins.reg4(16);
                if rn.is_sp_or_pc() {
                    return Err(DecodeError::Other); // ADR or ADD (SP plus immediate)
                }
                if rd.is_sp_or_pc() {
                    return Err(DecodeError::Unpredictable);
                }
                Self {
                    rd,
                    rn,
                    imm32: (ins >> 26 & 1) << 11 | (ins >> 12 & 7) << 8 | ins & 0xff,
                    set_flags: false,
                }
            }
            _ => panic!(),
        })
    }

    fn execute(&self, proc: &mut Arm7Processor) -> Result<bool, RunError> {
        let (r, c, v) = add_with_carry(proc.registers[self.rn], self.imm32, false);
        proc.registers.set(self.rd, r);
        if self.set_flags {
            proc.registers.xpsr.set_nz(r).set_c(c).set_v(v);
        }
        Ok(false)
    }

    fn name(&self) -> String {
        if self.set_flags { "adds" } else { "add" }.into()
    }

    fn args(&self, _pc: u32) -> String {
        let rdn = rdn_args_string(self.rd, self.rn);
        format!("{rdn}, #{}", self.imm32)
    }
}

/// ADD (register) instruction.
pub struct AddReg {
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

impl Instruction for AddReg {
    fn patterns() -> &'static [&'static str] {
        &[
            "0001100xxxxxxxxx",
            "01000100xxxxxxxx",
            "11101011000xxxxx(0)xxxxxxxxxxxxxxx",
        ]
    }

    fn try_decode(tn: usize, ins: u32, state: ItState) -> Result<Self, DecodeError> {
        Ok(match tn {
            1 => Self {
                rd: (ins & 7).into(),
                rn: (ins >> 3 & 7).into(),
                rm: (ins >> 6 & 7).into(),
                shift: Shift::lsl(0),
                set_flags: !state.in_it_block(),
            },
            2 => {
                let rm = ins.reg4(3);
                let rdn = RegisterIndex::new_main((ins >> 7 & 1) << 3 | ins & 7);
                if rdn.is_sp() || rm.is_sp() {
                    return Err(DecodeError::Other); // ADD (SP plus register)
                }
                if rdn.is_pc() && state.in_it_block_not_last() {
                    return Err(DecodeError::Unpredictable);
                }
                if rdn.is_pc() && rm.is_pc() {
                    return Err(DecodeError::Unpredictable);
                }
                Self {
                    rd: rdn,
                    rn: rdn,
                    rm,
                    shift: Shift::lsl(0),
                    set_flags: false,
                }
            }
            3 => {
                let rd = ins.reg4(8);
                let rn = ins.reg4(16);
                let rm = ins.reg4(0);
                let set_flags = ins >> 20 & 1 != 0;
                if rd.is_pc() && set_flags {
                    return Err(DecodeError::Other); // CMN (register)
                }
                if rn.is_sp() {
                    return Err(DecodeError::Other); // ADD (SP plus register)
                }
                if rd.is_sp_or_pc() || rn.is_pc() || rm.is_sp_or_pc() {
                    return Err(DecodeError::Unpredictable);
                }
                let shift = Shift::from_bits(ins >> 4 & 3, ((ins >> 12 & 7) << 2) | (ins >> 6 & 3));
                Self {
                    rd,
                    rn,
                    rm,
                    shift,
                    set_flags,
                }
            }
            _ => panic!(),
        })
    }

    fn execute(&self, proc: &mut crate::arm::Arm7Processor) -> Result<bool, RunError> {
        let carry_in = proc.registers.xpsr.c();
        let (shifted, _) = shift_c(proc.registers[self.rm], self.shift, carry_in);
        let (r, c, v) = add_with_carry(proc.registers[self.rn], shifted, false);
        if self.rd.is_pc() {
            proc.alu_write_pc(r);
            Ok(true)
        } else {
            proc.registers.set(self.rd, r);
            if self.set_flags {
                proc.registers.xpsr.set_nz(r).set_c(c).set_v(v);
            }
            Ok(false)
        }
    }

    fn name(&self) -> String {
        if self.set_flags { "adds" } else { "add" }.into()
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

/// ADD (SP plus immediate) instruction.
pub struct AddSpPlusImm {
    /// Destination register.
    rd: RegisterIndex,
    /// Value to be added.
    imm32: u32,
    /// True if condition flags are updated.
    set_flags: bool,
}

impl Instruction for AddSpPlusImm {
    fn patterns() -> &'static [&'static str] {
        &[
            "10101xxxxxxxxxxx",
            "101100000xxxxxxx",
            "11110x01000x11010xxxxxxxxxxxxxxx",
            "11110x10000011010xxxxxxxxxxxxxxx",
        ]
    }

    fn try_decode(tn: usize, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        Ok(match tn {
            1 => Self {
                rd: (ins >> 8 & 7).into(),
                imm32: (ins & 0xff) << 2,
                set_flags: false,
            },
            2 => Self {
                rd: RegisterIndex::Sp,
                imm32: (ins & 0x7f) << 2,
                set_flags: false,
            },
            3 => {
                let rd = RegisterIndex::new_main(ins >> 8 & 0xf);
                let set_flags = ins >> 20 & 1 != 0;
                other(rd.is_pc() && set_flags)?; // CMN (immediate)
                unpredictable(rd.is_pc())?;
                let imm12 = ins.imm1(26) << 11 | ins.imm3(12) << 8 | ins.imm8(0);
                let imm32 = thumb_expand_imm(imm12)?;
                Self {
                    rd: rd,
                    imm32,
                    set_flags,
                }
            }
            4 => {
                let rd = RegisterIndex::new_main(ins >> 8 & 0xf);
                if rd.is_pc() {
                    return Err(DecodeError::Unpredictable);
                }
                Self {
                    rd,
                    imm32: ins.imm1(26) << 11 | ins.imm3(12) << 8 | ins.imm8(0),
                    set_flags: false,
                }
            }
            _ => panic!(),
        })
    }

    fn execute(&self, proc: &mut crate::arm::Arm7Processor) -> Result<bool, RunError> {
        let (result, carry, overflow) = add_with_carry(proc.sp(), self.imm32, false);
        proc.registers.set(self.rd, result);
        if self.set_flags {
            proc.registers
                .xpsr
                .set_nz(result)
                .set_c(carry)
                .set_v(overflow);
        }
        Ok(false)
    }

    fn name(&self) -> String {
        if self.set_flags { "adds" } else { "add" }.into()
    }

    fn args(&self, _pc: u32) -> String {
        format!(
            "{}, #{}",
            rdn_args_string(self.rd, RegisterIndex::Sp),
            self.imm32
        )
    }
}

/// ADD (SP plus register) instruction.
pub struct AddSpPlusReg {
    /// Destination register.
    rd: RegisterIndex,
    /// Second operand register.
    rm: RegisterIndex,
    /// Shift applied to Rm.
    shift: Shift,
    /// True if condition flags are updated.
    set_flags: bool,
}

impl Instruction for AddSpPlusReg {
    fn patterns() -> &'static [&'static str] {
        &[
            "01000100x1101xxx",
            "010001001xxxx101",
            "11101011000x11010xxxxxxxxxxxxxxx",
        ]
    }

    fn try_decode(tn: usize, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        Ok(match tn {
            1 => {
                let rdm = RegisterIndex::new_main(ins.imm1(7) << 3 | ins.imm3(0));
                Self {
                    rd: rdm,
                    rm: rdm,
                    shift: Shift::lsl(0),
                    set_flags: false,
                }
            }
            2 => {
                let rm = ins.reg4(3);
                other(rm.is_sp())?; // T1 encoding
                Self {
                    rd: RegisterIndex::Sp,
                    rm,
                    shift: Shift::lsl(0),
                    set_flags: false,
                }
            }
            3 => {
                let rd = ins.reg4(8);
                let rm = ins.reg4(0);
                let imm5 = ins.imm3(12) << 2 | ins.imm2(6);
                let shift = Shift::from_bits(ins.imm2(4), imm5);
                unpredictable(rd.is_sp() && (shift.t != ShiftType::Lsl || shift.n > 3))?;
                unpredictable(rd.is_pc() || rm.is_sp_or_pc())?;
                Self {
                    rd,
                    rm,
                    shift,
                    set_flags: ins.bit(20),
                }
            }
            _ => panic!(),
        })
    }

    fn execute(&self, proc: &mut Arm7Processor) -> Result<bool, RunError> {
        let carry_in = proc.registers.xpsr.c();
        let (shifted, _) = shift_c(proc.registers[self.rm], self.shift, carry_in);
        let (result, carry, overflow) = add_with_carry(proc.sp(), shifted, false);
        if self.rd.is_pc() {
            proc.alu_write_pc(result);
            Ok(true)
        } else {
            proc.registers.set(self.rd, result);
            if self.set_flags {
                proc.registers
                    .xpsr
                    .set_nz(result)
                    .set_c(carry)
                    .set_v(overflow);
            }
            Ok(false)
        }
    }

    fn name(&self) -> String {
        if self.set_flags { "adds" } else { "add" }.into()
    }

    fn args(&self, _pc: u32) -> String {
        format!(
            "{}, {}{}",
            rdn_args_string(self.rd, RegisterIndex::Sp),
            self.rm,
            self.shift.arg_string()
        )
    }
}

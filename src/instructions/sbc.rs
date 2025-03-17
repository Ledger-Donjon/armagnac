//! Implements SBC (Subtract with Carry) instruction.

use super::Instruction;
use crate::{
    arith::{add_with_carry, shift_c, thumb_expand_imm, Shift},
    arm::{ArmProcessor, RunError},
    decoder::DecodeError,
    helpers::BitAccess,
    instructions::{rdn_args_string, unpredictable, DecodeHelper},
    it_state::ItState,
    registers::RegisterIndex,
};
use core::panic;

/// SBC (immediate) instruction.
///
/// Subtract with Carry.
pub struct SbcImm {
    /// Destination register.
    rd: RegisterIndex,
    /// First operand register.
    rn: RegisterIndex,
    /// Second operand immediate value.
    imm32: u32,
    /// True if condition flags are updated.
    set_flags: bool,
}

impl Instruction for SbcImm {
    fn patterns() -> &'static [&'static str] {
        &["11110x01011xxxxx0xxxxxxxxxxxxxxx"]
    }

    fn try_decode(tn: usize, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        debug_assert_eq!(tn, 1);
        let rd = ins.reg4(8);
        let rn = ins.reg4(16);
        unpredictable(rd.is_sp_or_pc() | rn.is_sp_or_pc())?;
        let imm32 = thumb_expand_imm((ins.imm1(26) << 11) | (ins.imm3(12) << 8) | ins.imm8(0))?;
        Ok(Self {
            rd,
            rn,
            imm32,
            set_flags: ins.bit(20),
        })
    }

    fn execute(&self, proc: &mut ArmProcessor) -> Result<bool, RunError> {
        let carry_in = proc.registers.xpsr.c();
        let (result, carry, overflow) = add_with_carry(proc[self.rn], !self.imm32, carry_in);
        proc.set(self.rd, result);
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
        if self.set_flags { "sbcs" } else { "sbc" }.into()
    }

    fn args(&self, _pc: u32) -> String {
        format!("{}, {}", rdn_args_string(self.rd, self.rn), self.imm32)
    }
}

/// SBC (register) instruction.
///
/// Subtract with Carry.
pub struct SbcReg {
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

impl Instruction for SbcReg {
    fn patterns() -> &'static [&'static str] {
        &["0100000110xxxxxx", "11101011011xxxxx(0)xxxxxxxxxxxxxxx"]
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
                unpredictable(rd.is_sp_or_pc() | rn.is_sp_or_pc() | rm.is_sp_or_pc())?;
                Self {
                    rd,
                    rn,
                    rm,
                    shift: Shift::from_bits(ins.imm2(4), (ins.imm3(12) << 2) | ins.imm2(6)),
                    set_flags: ins.bit(20),
                }
            }
            _ => panic!(),
        })
    }

    fn execute(&self, proc: &mut ArmProcessor) -> Result<bool, RunError> {
        let carry_in = proc.registers.xpsr.c();
        let shifted = shift_c(proc[self.rm], self.shift, carry_in).0;
        let (result, carry, overflow) = add_with_carry(proc[self.rn], !shifted, carry_in);
        proc.set(self.rd, result);
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
        if self.set_flags { "sbcs" } else { "sbc" }.into()
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

#[cfg(test)]
mod tests {
    use crate::{
        arith::Shift,
        arm::ArmProcessor,
        instructions::{
            sbc::{SbcImm, SbcReg},
            Instruction,
        },
        registers::RegisterIndex,
    };

    #[test]
    fn test_sbc_imm() {
        let mut proc = ArmProcessor::new(crate::arm::ArmVersion::V8M, 0);
        proc.registers.r1 = 1000;
        SbcImm {
            rd: RegisterIndex::R0,
            rn: RegisterIndex::R1,
            imm32: 99,
            set_flags: true,
        }
        .execute(&mut proc)
        .unwrap();
        assert_eq!(proc.registers.r0, 900);
        assert_eq!(proc.registers.xpsr.z(), false);
        assert_eq!(proc.registers.xpsr.c(), true);
        assert_eq!(proc.registers.xpsr.v(), false);

        proc.registers.xpsr.set_c(true);
        SbcImm {
            rd: RegisterIndex::R0,
            rn: RegisterIndex::R1,
            imm32: 1000,
            set_flags: true,
        }
        .execute(&mut proc)
        .unwrap();
        assert_eq!(proc.registers.r0, 0);
        assert_eq!(proc.registers.xpsr.z(), true);
        assert_eq!(proc.registers.xpsr.c(), true);
        assert_eq!(proc.registers.xpsr.v(), false);
    }

    #[test]
    fn test_sbc_reg() {
        let mut proc = ArmProcessor::new(crate::arm::ArmVersion::V8M, 0);
        proc.registers.r1 = 1000;
        proc.registers.r2 = 99;
        SbcReg {
            rd: RegisterIndex::R0,
            rn: RegisterIndex::R1,
            rm: RegisterIndex::R2,
            shift: Shift::lsl(0),
            set_flags: true,
        }
        .execute(&mut proc)
        .unwrap();
        assert_eq!(proc.registers.r0, 900);
        assert_eq!(proc.registers.xpsr.z(), false);
        assert_eq!(proc.registers.xpsr.c(), true);
        assert_eq!(proc.registers.xpsr.v(), false);

        proc.registers.xpsr.set_c(true);
        proc.registers.r2 = 250;
        SbcReg {
            rd: RegisterIndex::R0,
            rn: RegisterIndex::R1,
            rm: RegisterIndex::R2,
            shift: Shift::lsl(2),
            set_flags: true,
        }
        .execute(&mut proc)
        .unwrap();
        assert_eq!(proc.registers.r0, 0);
        assert_eq!(proc.registers.xpsr.z(), true);
        assert_eq!(proc.registers.xpsr.c(), true);
        assert_eq!(proc.registers.xpsr.v(), false);
    }
}

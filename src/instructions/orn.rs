//! Implements ORN (Logical OR NOT) instruction.

use crate::{
    arith::{shift_c, thumb_expand_imm_optc, Shift},
    arm::{ArmProcessor, RunError},
    decoder::DecodeError,
    helpers::BitAccess,
    instructions::{other, rdn_args_string, unpredictable, DecodeHelper},
    it_state::ItState,
    registers::RegisterIndex,
};

use super::Instruction;

/// ORN (immediate) instruction.
///
/// Logical OR NOT.
pub struct OrnImm {
    /// Destination register.
    rd: RegisterIndex,
    /// Operand register.
    rn: RegisterIndex,
    /// Immediate value to be added to Rn.
    imm32: u32,
    /// Carry.
    carry: Option<bool>,
    /// True if condition flags are updated.
    set_flags: bool,
}

impl Instruction for OrnImm {
    fn patterns() -> &'static [&'static str] {
        &["11110x00011xxxxx0xxxxxxxxxxxxxxx"]
    }

    fn try_decode(tn: usize, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        debug_assert_eq!(tn, 1);
        let rd = ins.reg4(8);
        let rn = ins.reg4(16);
        other(rn.is_pc())?; // MVN (immediate)
        let (imm32, carry) =
            thumb_expand_imm_optc((ins.imm1(26) << 11) | (ins.imm3(12) << 8) | ins.imm8(0))?;
        Ok(Self {
            rd,
            rn,
            imm32,
            carry,
            set_flags: ins.bit(20),
        })
    }

    fn execute(&self, proc: &mut ArmProcessor) -> Result<bool, RunError> {
        let result = proc[self.rn] | !self.imm32;
        proc.set(self.rd, result);
        if self.set_flags {
            proc.registers.xpsr.set_nz(result).set_c_opt(self.carry);
        }
        Ok(false)
    }

    fn name(&self) -> String {
        if self.set_flags { "orns" } else { "orn" }.into()
    }

    fn args(&self, _pc: u32) -> String {
        format!("{}, #{}", rdn_args_string(self.rd, self.rn), self.imm32)
    }
}

/// ORN (register) instruction.
///
/// Logical OR NOT.
pub struct OrnReg {
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

impl Instruction for OrnReg {
    fn patterns() -> &'static [&'static str] {
        &["11101010011xxxxx(0)xxxxxxxxxxxxxxx"]
    }

    fn try_decode(tn: usize, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        debug_assert_eq!(tn, 1);
        let rd = ins.reg4(8);
        let rn = ins.reg4(16);
        let rm = ins.reg4(0);
        other(rn.is_pc())?; // MVN (register)
        unpredictable(rd.is_sp_or_pc() || rn.is_sp() || rm.is_sp_or_pc())?;
        Ok(Self {
            rd,
            rn,
            rm,
            shift: Shift::from_bits(ins.imm2(4), (ins.imm3(12) << 2) | ins.imm2(6)),
            set_flags: ins.bit(20),
        })
    }

    fn execute(&self, proc: &mut ArmProcessor) -> Result<bool, RunError> {
        let carry_in = proc.registers.xpsr.c();
        let (shifted, carry) = shift_c(proc[self.rm], self.shift, carry_in);
        let result = proc[self.rn] | !shifted;
        proc.set(self.rd, result);
        if self.set_flags {
            proc.registers.xpsr.set_nz(result).set_c(carry);
        }
        Ok(false)
    }

    fn name(&self) -> String {
        if self.set_flags { "orns" } else { "orn" }.into()
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
    use super::OrnImm;
    use crate::{
        arith::Shift,
        arm::ArmProcessor,
        instructions::{orn::OrnReg, Instruction},
        registers::RegisterIndex,
    };

    #[test]
    fn test_orn_imm() {
        let mut proc = ArmProcessor::new(crate::arm::ArmVersion::V8M, 0);
        proc.registers.r1 = 0x12345678;
        OrnImm {
            rd: RegisterIndex::R0,
            rn: RegisterIndex::R1,
            imm32: 0x00ff00ff,
            carry: None,
            set_flags: true,
        }
        .execute(&mut proc)
        .unwrap();
        assert_eq!(proc.registers.r0, 0xff34ff78);
        assert_eq!(proc.registers.xpsr.n(), true);
        assert_eq!(proc.registers.xpsr.z(), false);
        assert_eq!(proc.registers.xpsr.z(), false);
    }

    #[test]
    fn test_orn_reg() {
        let mut proc = ArmProcessor::new(crate::arm::ArmVersion::V8M, 0);
        proc.registers.r1 = 0x12345678;
        proc.registers.r2 = 0x00ff00ff;
        OrnReg {
            rd: RegisterIndex::R0,
            rn: RegisterIndex::R1,
            rm: RegisterIndex::R2,
            shift: Shift::lsl(0),
            set_flags: true,
        }
        .execute(&mut proc)
        .unwrap();
        assert_eq!(proc.registers.r0, 0xff34ff78);
        assert_eq!(proc.registers.xpsr.n(), true);
        assert_eq!(proc.registers.xpsr.z(), false);
        assert_eq!(proc.registers.xpsr.z(), false);

        OrnReg {
            rd: RegisterIndex::R0,
            rn: RegisterIndex::R1,
            rm: RegisterIndex::R2,
            shift: Shift::lsl(8),
            set_flags: false,
        }
        .execute(&mut proc)
        .unwrap();
        assert_eq!(proc.registers.r0, 0x12ff56ff);
        assert_eq!(proc.registers.xpsr.n(), true);
        assert_eq!(proc.registers.xpsr.z(), false);
        assert_eq!(proc.registers.xpsr.z(), false);
    }
}

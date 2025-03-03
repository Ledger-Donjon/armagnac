//! Implements ADC (Add with Carry) instruction.

use crate::{
    arith::{add_with_carry, shift_c, thumb_expand_imm, Shift},
    arm::{Arm7Processor, RunError},
    decoder::DecodeError,
    helpers::BitAccess,
    instructions::{rdn_args_string, unpredictable, DecodeHelper},
    it_state::ItState,
    registers::RegisterIndex,
};

use super::Instruction;

/// ADC (immediate) instruction.
///
/// Add with Carry.
pub struct AdcImm {
    /// Destination register.
    rd: RegisterIndex,
    /// First operand register.
    rn: RegisterIndex,
    /// Value to be added.
    imm32: u32,
    /// True if condition flags are updated.
    set_flags: bool,
}

impl Instruction for AdcImm {
    fn patterns() -> &'static [&'static str] {
        &["11110x01010xxxxx0xxxxxxxxxxxxxxx"]
    }

    fn try_decode(tn: usize, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        assert_eq!(tn, 1);
        let rd = ins.reg4(8);
        let rn = ins.reg4(16);
        unpredictable(rd.is_sp_or_pc() || rn.is_sp_or_pc())?;
        let imm32 = thumb_expand_imm(ins.imm1(26) << 11 | ins.imm3(12) << 8 | ins.imm8(0))?;
        Ok(Self {
            rd,
            rn,
            imm32,
            set_flags: ins.bit(20),
        })
    }

    fn execute(&self, proc: &mut Arm7Processor) -> Result<bool, RunError> {
        let carry_in = proc.registers.xpsr.c();
        let (result, carry, overflow) =
            add_with_carry(proc.registers[self.rn], self.imm32, carry_in);
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
        "adc".into()
    }

    fn args(&self, _pc: u32) -> String {
        format!("{}, #{}", rdn_args_string(self.rd, self.rn), self.imm32)
    }
}

/// ADC (register) instruction.
///
/// Add with Carry.
pub struct AdcReg {
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

impl Instruction for AdcReg {
    fn patterns() -> &'static [&'static str] {
        &["0100000101xxxxxx", "11101011010xxxxx(0)xxxxxxxxxxxxxxx"]
    }

    fn try_decode(tn: usize, ins: u32, state: ItState) -> Result<Self, DecodeError> {
        Ok(match tn {
            1 => Self {
                rd: ins.reg3(0),
                rn: ins.reg3(0),
                rm: ins.reg3(3),
                shift: Shift::lsl(0),
                set_flags: !state.in_it_block(),
            },
            2 => {
                let rd = ins.reg4(8);
                let rn = ins.reg4(16);
                let rm = ins.reg4(0);
                unpredictable(rd.is_sp_or_pc() || rn.is_sp_or_pc() || rm.is_sp_or_pc())?;
                Self {
                    rd,
                    rn,
                    rm,
                    shift: Shift::from_bits(ins.imm2(4), ins.imm3(12) << 2 | ins.imm2(6)),
                    set_flags: ins.bit(20),
                }
            }
            _ => panic!(),
        })
    }

    fn execute(&self, proc: &mut Arm7Processor) -> Result<bool, RunError> {
        let carry_in = proc.registers.xpsr.c();
        let shifted = shift_c(proc.registers[self.rm], self.shift, carry_in).0;
        let (result, carry, overflow) = add_with_carry(proc.registers[self.rn], shifted, carry_in);
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
        if self.set_flags { "adcs" } else { "adc" }.into()
    }

    fn args(&self, _pc: u32) -> String {
        format!(
            "{}, {}{}",
            rdn_args_string(self.rd, self.rn),
            self.rm,
            self.shift.arg_string(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::AdcImm;
    use crate::{
        arith::Shift,
        arm::Arm7Processor,
        instructions::{adc::AdcReg, Instruction},
        registers::RegisterIndex,
    };

    fn test_adc_imm_vec(
        proc: &mut Arm7Processor,
        carry_in: bool,
        imm: u32,
        set_flags: bool,
        expected_r0: u32,
        expected_carry: bool,
    ) {
        proc.registers.r0 = 0;
        proc.registers.r1 = 100;
        proc.registers.xpsr.set(0);
        proc.registers.xpsr.set_c(carry_in);
        AdcImm {
            rd: RegisterIndex::R0,
            rn: RegisterIndex::R1,
            imm32: imm,
            set_flags: set_flags,
        }
        .execute(proc)
        .unwrap();
        assert_eq!(proc.registers.r0, expected_r0);
        assert_eq!(proc.registers.xpsr.c(), expected_carry);
    }

    #[test]
    fn test_adc_imm() {
        let mut proc = Arm7Processor::new(crate::arm::ArmVersion::V8M, 0);
        test_adc_imm_vec(&mut proc, false, 10, true, 110, false);
        test_adc_imm_vec(&mut proc, true, 10, true, 111, false);
        test_adc_imm_vec(&mut proc, false, 0xffffffff - 99, true, 0, true);
        test_adc_imm_vec(&mut proc, true, 0xffffffff - 99, true, 1, true);
        test_adc_imm_vec(&mut proc, false, 0xffffffff - 99, false, 0, false);
    }

    fn test_adc_reg_vec(
        proc: &mut Arm7Processor,
        carry_in: bool,
        r2: u32,
        shift: Shift,
        expected_r0: u32,
        expected_carry: bool,
    ) {
        proc.registers.r0 = 0;
        proc.registers.r1 = 100;
        proc.registers.r2 = r2;
        proc.registers.xpsr.set(0);
        proc.registers.xpsr.set_c(carry_in);
        AdcReg {
            rd: RegisterIndex::R0,
            rn: RegisterIndex::R1,
            rm: RegisterIndex::R2,
            shift,
            set_flags: true,
        }
        .execute(proc)
        .unwrap();
        assert_eq!(proc.registers.r0, expected_r0);
        assert_eq!(proc.registers.xpsr.c(), expected_carry);
    }

    #[test]
    fn test_adc_reg() {
        let mut proc = Arm7Processor::new(crate::arm::ArmVersion::V8M, 0);
        test_adc_reg_vec(&mut proc, false, 10, Shift::lsl(0), 110, false);
        test_adc_reg_vec(&mut proc, true, 10, Shift::lsl(0), 111, false);
        test_adc_reg_vec(&mut proc, false, 0xffffffff - 99, Shift::lsl(0), 0, true);
        test_adc_reg_vec(&mut proc, true, 0xffffffff - 99, Shift::lsl(0), 1, true);
        test_adc_reg_vec(&mut proc, true, 10, Shift::lsl(2), 141, false);
    }
}

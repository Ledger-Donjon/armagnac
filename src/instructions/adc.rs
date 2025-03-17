//! Implements ADC (Add with Carry) instruction.

use crate::{
    arith::{add_with_carry, shift_c, thumb_expand_imm, Shift},
    arm::{ArmProcessor, RunError},
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
        let imm32 = thumb_expand_imm((ins.imm1(26) << 11) | (ins.imm3(12) << 8) | ins.imm8(0))?;
        Ok(Self {
            rd,
            rn,
            imm32,
            set_flags: ins.bit(20),
        })
    }

    fn execute(&self, proc: &mut ArmProcessor) -> Result<bool, RunError> {
        let carry_in = proc.registers.psr.c();
        let (result, carry, overflow) = add_with_carry(proc[self.rn], self.imm32, carry_in);
        proc.set(self.rd, result);
        if self.set_flags {
            proc.registers
                .psr
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
                    shift: Shift::from_bits(ins.imm2(4), (ins.imm3(12) << 2) | ins.imm2(6)),
                    set_flags: ins.bit(20),
                }
            }
            _ => panic!(),
        })
    }

    fn execute(&self, proc: &mut ArmProcessor) -> Result<bool, RunError> {
        let carry_in = proc.registers.psr.c();
        let shifted = shift_c(proc[self.rm], self.shift, carry_in).0;
        let (result, carry, overflow) = add_with_carry(proc[self.rn], shifted, carry_in);
        proc.set(self.rd, result);
        if self.set_flags {
            proc.registers
                .psr
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
        arm::{
            ArmProcessor,
            ArmVersion::{V7M, V8M},
        },
        instructions::{adc::AdcReg, Instruction},
        registers::RegisterIndex,
    };

    #[test]
    fn test_adc_imm() {
        struct Test {
            carry_in: bool,
            imm: u32,
            set_flags: bool,
            expected_r0: u32,
            expected_nzcv: (bool, bool, bool, bool),
        }

        let vectors = [
            // Test basic addition
            Test {
                carry_in: false,
                imm: 10,
                set_flags: true,
                expected_r0: 110,
                expected_nzcv: (false, false, false, false),
            },
            // Test input carry is added
            Test {
                carry_in: true,
                imm: 10,
                set_flags: true,
                expected_r0: 111,
                expected_nzcv: (false, false, false, false),
            },
            // Test Z and C are set
            Test {
                carry_in: false,
                imm: 0xffffffff - 99,
                set_flags: true,
                expected_r0: 0,
                expected_nzcv: (false, true, true, false),
            },
            // Test only C is set
            Test {
                carry_in: true,
                imm: 0xffffffff - 99,
                set_flags: true,
                expected_r0: 1,
                expected_nzcv: (false, false, true, false),
            },
            // Test flags are not updated
            Test {
                carry_in: false,
                imm: 0xffffffff - 99,
                set_flags: false,
                expected_r0: 0,
                expected_nzcv: (false, false, false, false),
            },
            // Test overflow bit
            Test {
                carry_in: true,
                imm: 0x7fffffff - 99,
                set_flags: true,
                expected_r0: 0x80000001,
                expected_nzcv: (true, false, false, true),
            },
        ];

        for v in vectors {
            let mut proc = ArmProcessor::new(V7M, 0);
            let rd = RegisterIndex::new_general_random();
            let rn = RegisterIndex::new_general_random();
            proc.registers.psr.set_c(v.carry_in);
            proc.set(rn, 100);
            let mut expected = proc.registers.clone();
            expected.set(rd, v.expected_r0);
            expected
                .psr
                .set_n(v.expected_nzcv.0)
                .set_z(v.expected_nzcv.1)
                .set_c(v.expected_nzcv.2)
                .set_v(v.expected_nzcv.3);
            AdcImm {
                rd,
                rn,
                imm32: v.imm,
                set_flags: v.set_flags,
            }
            .execute(&mut proc)
            .unwrap();
            assert_eq!(proc.registers, expected);
        }
    }

    #[test]
    fn test_adc_reg() {
        struct Test {
            carry_in: bool,
            initial_r2: u32,
            shift: Shift,
            set_flags: bool,
            expected_r0: u32,
            expected_nzcv: (bool, bool, bool, bool),
        }

        let vectors = [
            // Test basic addition
            Test {
                carry_in: false,
                initial_r2: 10,
                shift: Shift::lsl(0),
                set_flags: true,
                expected_r0: 110,
                expected_nzcv: (false, false, false, false),
            },
            // Test input carry is added
            Test {
                carry_in: true,
                initial_r2: 10,
                shift: Shift::lsl(0),
                set_flags: true,
                expected_r0: 111,
                expected_nzcv: (false, false, false, false),
            },
            // Test Z and C flags are set
            Test {
                carry_in: false,
                initial_r2: 0xffffffff - 99,
                shift: Shift::lsl(0),
                set_flags: true,
                expected_r0: 0,
                expected_nzcv: (false, true, true, false),
            },
            // Test shift is applied
            Test {
                carry_in: true,
                initial_r2: 10,
                shift: Shift::lsl(2),
                set_flags: true,
                expected_r0: 141,
                expected_nzcv: (false, false, false, false),
            },
            // Test overflow flag is set
            Test {
                carry_in: false,
                initial_r2: 0x7fffffff - 99,
                shift: Shift::lsl(0),
                set_flags: true,
                expected_r0: 0x80000000,
                expected_nzcv: (true, false, false, true),
            },
            // Test flags are NOT updated
            Test {
                carry_in: false,
                initial_r2: 0xffffffff - 99,
                shift: Shift::lsl(0),
                set_flags: false,
                expected_r0: 0,
                expected_nzcv: (false, false, false, false),
            },
        ];

        for v in vectors {
            let mut proc = ArmProcessor::new(V8M, 0);
            let rd = RegisterIndex::new_general_random();
            let (rn, rm) = RegisterIndex::pick_two_general_distinct();
            proc.set(rn, 100);
            proc.set(rm, v.initial_r2);
            proc.registers.psr.set_c(v.carry_in);
            let mut expected = proc.registers.clone();
            expected.set(rd, v.expected_r0);
            expected
                .psr
                .set_n(v.expected_nzcv.0)
                .set_z(v.expected_nzcv.1)
                .set_c(v.expected_nzcv.2)
                .set_v(v.expected_nzcv.3);
            AdcReg {
                rd,
                rn,
                rm,
                shift: v.shift,
                set_flags: v.set_flags,
            }
            .execute(&mut proc)
            .unwrap();
            assert_eq!(proc.registers, expected);
        }
    }
}

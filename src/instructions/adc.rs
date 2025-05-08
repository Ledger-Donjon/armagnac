//! Implements ADC (Add with Carry) instruction.

use super::ArmVersion::{V6M, V7EM, V7M, V8M};
use super::Encoding::{self, T1, T2};
use super::{Instruction, Pattern, Qualifier};
use crate::instructions::rdn_args_string;
use crate::qualifier_wide_match;
use crate::{
    arith::{add_with_carry, shift_c, thumb_expand_imm, Shift},
    arm::{ArmProcessor, RunError},
    decoder::DecodeError,
    helpers::BitAccess,
    instructions::{unpredictable, DecodeHelper},
    it_state::ItState,
    registers::RegisterIndex,
};

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
    fn patterns() -> &'static [Pattern] {
        &[Pattern {
            encoding: T1,
            versions: &[V7M, V7EM, V8M],
            expression: "11110x01010xxxxx0xxxxxxxxxxxxxxx",
        }]
    }

    fn try_decode(encoding: Encoding, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        assert_eq!(encoding, T1);
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

    fn sets_flags(&self) -> bool {
        self.set_flags
    }

    fn args(&self, _pc: u32) -> String {
        format!("{}, {}, #{}", self.rd, self.rn, self.imm32)
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
    /// Encoding.
    encoding: Encoding,
}

impl Instruction for AdcReg {
    fn patterns() -> &'static [Pattern] {
        &[
            Pattern {
                encoding: T1,
                versions: &[V6M, V7M, V7EM, V8M],
                expression: "0100000101xxxxxx",
            },
            Pattern {
                encoding: T2,
                versions: &[V7M, V7EM, V8M],
                expression: "11101011010xxxxx(0)xxxxxxxxxxxxxxx",
            },
        ]
    }

    fn try_decode(encoding: Encoding, ins: u32, state: ItState) -> Result<Self, DecodeError> {
        Ok(match encoding {
            T1 => Self {
                rd: ins.reg3(0),
                rn: ins.reg3(0),
                rm: ins.reg3(3),
                shift: Shift::lsl(0),
                set_flags: !state.in_it_block(),
                encoding,
            },
            T2 => {
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
                    encoding,
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
        "adc".into()
    }

    fn sets_flags(&self) -> bool {
        self.set_flags
    }

    fn qualifier(&self) -> Qualifier {
        qualifier_wide_match!(self.encoding, T2)
    }

    fn args(&self, _pc: u32) -> String {
        format!(
            "{}, {}{}",
            rdn_args_string(self.rd, self.rn, self.encoding == T1),
            self.rm,
            self.shift.arg_string()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::AdcImm;
    use crate::{
        arith::Shift,
        arm::{ArmProcessor, Config},
        instructions::{adc::AdcReg, Encoding, Instruction},
        registers::RegisterIndex,
    };

    #[test]
    fn test_adc_imm() {
        struct Test {
            carry_in: bool,
            imm: u32,
            set_flags: bool,
            expected_r0: u32,
            expected_flags: u8,
        }

        let vectors = [
            // Test basic addition
            Test {
                carry_in: false,
                imm: 10,
                set_flags: true,
                expected_r0: 110,
                expected_flags: 0,
            },
            // Test input carry is added
            Test {
                carry_in: true,
                imm: 10,
                set_flags: true,
                expected_r0: 111,
                expected_flags: 0,
            },
            // Test Z and C are set
            Test {
                carry_in: false,
                imm: 0xffffffff - 99,
                set_flags: true,
                expected_r0: 0,
                expected_flags: 0b01100,
            },
            // Test only C is set
            Test {
                carry_in: true,
                imm: 0xffffffff - 99,
                set_flags: true,
                expected_r0: 1,
                expected_flags: 0b00100,
            },
            // Test flags are not updated
            Test {
                carry_in: false,
                imm: 0xffffffff - 99,
                set_flags: false,
                expected_r0: 0,
                expected_flags: 0,
            },
            // Test overflow bit
            Test {
                carry_in: true,
                imm: 0x7fffffff - 99,
                set_flags: true,
                expected_r0: 0x80000001,
                expected_flags: 0b10010,
            },
        ];

        for v in vectors {
            let mut proc = ArmProcessor::new(Config::v7m());
            let rd = RegisterIndex::new_general_random();
            let rn = RegisterIndex::new_general_random();
            proc.registers.psr.set_c(v.carry_in);
            proc.set(rn, 100);
            let mut expected = proc.registers.clone();
            expected.set(rd, v.expected_r0);
            expected.psr.set_flags(v.expected_flags);
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
            expected_flags: u8,
        }

        let vectors = [
            // Test basic addition
            Test {
                carry_in: false,
                initial_r2: 10,
                shift: Shift::lsl(0),
                set_flags: true,
                expected_r0: 110,
                expected_flags: 0,
            },
            // Test input carry is added
            Test {
                carry_in: true,
                initial_r2: 10,
                shift: Shift::lsl(0),
                set_flags: true,
                expected_r0: 111,
                expected_flags: 0,
            },
            // Test Z and C flags are set
            Test {
                carry_in: false,
                initial_r2: 0xffffffff - 99,
                shift: Shift::lsl(0),
                set_flags: true,
                expected_r0: 0,
                expected_flags: 0b01100,
            },
            // Test shift is applied
            Test {
                carry_in: true,
                initial_r2: 10,
                shift: Shift::lsl(2),
                set_flags: true,
                expected_r0: 141,
                expected_flags: 0,
            },
            // Test overflow flag is set
            Test {
                carry_in: false,
                initial_r2: 0x7fffffff - 99,
                shift: Shift::lsl(0),
                set_flags: true,
                expected_r0: 0x80000000,
                expected_flags: 0b10010,
            },
            // Test flags are NOT updated
            Test {
                carry_in: false,
                initial_r2: 0xffffffff - 99,
                shift: Shift::lsl(0),
                set_flags: false,
                expected_r0: 0,
                expected_flags: 0,
            },
        ];

        for v in vectors {
            let mut proc = ArmProcessor::new(Config::v8m());
            let rd = RegisterIndex::new_general_random();
            let (rn, rm) = RegisterIndex::pick_two_general_distinct();
            proc.set(rn, 100);
            proc.set(rm, v.initial_r2);
            proc.registers.psr.set_c(v.carry_in);
            let mut expected = proc.registers.clone();
            expected.set(rd, v.expected_r0);
            expected.psr.set_flags(v.expected_flags);
            AdcReg {
                rd,
                rn,
                rm,
                shift: v.shift,
                set_flags: v.set_flags,
                encoding: Encoding::DontCare,
            }
            .execute(&mut proc)
            .unwrap();
            assert_eq!(proc.registers, expected);
        }
    }
}

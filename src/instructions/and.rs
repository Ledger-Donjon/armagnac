//! Implements AND instruction.

use super::ArmVersion::{V6M, V7EM, V7M, V8M};
use super::{other, unpredictable, DecodeHelper, Instruction, Pattern, Qualifier};
use crate::instructions::rdn_args_string;
use crate::qualifier_wide_match;
use crate::{
    arith::{shift_c, thumb_expand_imm_optc, Shift},
    arm::{ArmProcessor, RunError},
    decoder::DecodeError,
    helpers::BitAccess,
    it_state::ItState,
    registers::RegisterIndex,
};

/// AND immediate instruction.
pub struct AndImm {
    /// Destination register.
    rd: RegisterIndex,
    /// First operand register.
    rn: RegisterIndex,
    /// Second operand.
    imm32: u32,
    /// True if condition flags are updated.
    set_flags: bool,
    /// Carry.
    carry: Option<bool>,
}

impl Instruction for AndImm {
    fn patterns() -> &'static [Pattern] {
        &[Pattern {
            tn: 1,
            versions: &[V7M, V7EM, V8M],
            expression: "11110x00000xxxxx0xxxxxxxxxxxxxxx",
        }]
    }

    fn try_decode(tn: usize, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        Ok(match tn {
            1 => {
                let rd = ins.reg4(8);
                let rn = ins.reg4(16);
                let imm12 = (ins.imm1(26) << 11) | (ins.imm3(12) << 8) | (ins & 0xff);
                let (imm32, carry) = thumb_expand_imm_optc(imm12)?;
                let set_flags = ins.bit(20);
                other(rd.is_pc() && set_flags)?; // TST (immediate)
                unpredictable(rd.is_sp_or_pc() || rn.is_sp_or_pc())?;
                Self {
                    rd,
                    rn,
                    imm32,
                    set_flags,
                    carry,
                }
            }
            _ => panic!(),
        })
    }

    fn execute(&self, proc: &mut ArmProcessor) -> Result<bool, RunError> {
        let result = proc[self.rn] & self.imm32;
        proc.set(self.rd, result);
        if self.set_flags {
            proc.registers.psr.set_nz(result).set_c_opt(self.carry);
        }
        Ok(false)
    }

    fn name(&self) -> String {
        "and".into()
    }

    fn sets_flags(&self) -> bool {
        self.set_flags
    }

    fn args(&self, _pc: u32) -> String {
        format!("{}, {}, #{}", self.rd, self.rn, self.imm32)
    }
}

pub struct AndReg {
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
    tn: usize,
}

impl Instruction for AndReg {
    fn patterns() -> &'static [Pattern] {
        &[
            Pattern {
                tn: 1,
                versions: &[V6M, V7M, V7EM, V8M],
                expression: "0100000000xxxxxx",
            },
            Pattern {
                tn: 2,
                versions: &[V7M, V7EM, V8M],
                expression: "11101010000xxxxx(0)xxxxxxxxxxxxxxx",
            },
        ]
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
                    tn,
                }
            }
            2 => {
                let rd = ins.reg4(8);
                let rn = ins.reg4(16);
                let rm = ins.reg4(0);
                let set_flags = ins.bit(20);
                other(rd.is_pc() && set_flags)?; // TST (register)
                unpredictable(rd.is_sp_or_pc() || rn.is_sp_or_pc() || rm.is_sp_or_pc())?;
                Self {
                    rd,
                    rn,
                    rm,
                    shift: Shift::from_bits(ins.imm2(4), (ins.imm3(12) << 2) | ins.imm2(6)),
                    set_flags,
                    tn,
                }
            }
            _ => panic!(),
        })
    }

    fn execute(&self, proc: &mut ArmProcessor) -> Result<bool, RunError> {
        let carry_in = proc.registers.psr.c();
        let (shifted, carry) = shift_c(proc[self.rm], self.shift, carry_in);
        let result = proc[self.rn] & shifted;
        proc.set(self.rd, result);
        if self.set_flags {
            proc.registers.psr.set_nz(result).set_c(carry);
        }
        Ok(false)
    }

    fn name(&self) -> String {
        "and".into()
    }

    fn sets_flags(&self) -> bool {
        self.set_flags
    }

    fn qualifier(&self) -> Qualifier {
        qualifier_wide_match!(self.tn, 2)
    }

    fn args(&self, _pc: u32) -> String {
        format!(
            "{}, {}{}",
            rdn_args_string(self.rd, self.rn, self.tn == 1),
            self.rm,
            self.shift.arg_string()
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        arith::Shift,
        arm::{ArmProcessor, ArmVersion::V7M},
        instructions::{
            and::{AndImm, AndReg},
            Instruction,
        },
        registers::RegisterIndex,
    };

    #[test]
    fn test_and_imm() {
        struct Test {
            initial_rn: u32,
            imm32: u32,
            set_flags: bool,
            carry: Option<bool>,
            expected_rd: u32,
            expected_flags: u8,
        }

        let vectors = [
            Test {
                initial_rn: 0x12345678,
                imm32: 0,
                set_flags: true,
                carry: None,
                expected_rd: 0,
                expected_flags: 0b01000,
            },
            Test {
                initial_rn: 0x55aa55aa,
                imm32: 0xaa55aa00,
                set_flags: false,
                carry: Some(true),
                expected_rd: 0,
                expected_flags: 0b00000,
            },
            Test {
                initial_rn: 0x12345678,
                imm32: 0x87654321,
                set_flags: true,
                carry: None,
                expected_rd: 0x02244220,
                expected_flags: 0b00000,
            },
            Test {
                initial_rn: 0x92345678,
                imm32: 0x87654321,
                set_flags: true,
                carry: Some(true),
                expected_rd: 0x82244220,
                expected_flags: 0b10100,
            },
        ];

        for v in vectors {
            let mut proc = ArmProcessor::new(V7M, 0);
            let rd = RegisterIndex::new_general_random();
            let rn = RegisterIndex::new_general_random();
            proc.set(rn, v.initial_rn);
            let mut expected = proc.registers.clone();
            expected.set(rd, v.expected_rd);
            expected.psr.set_flags(v.expected_flags);
            AndImm {
                rd,
                rn,
                imm32: v.imm32,
                set_flags: v.set_flags,
                carry: v.carry,
            }
            .execute(&mut proc)
            .unwrap();
            assert_eq!(proc.registers, expected);
        }
    }

    #[test]
    fn test_and_reg() {
        struct Test {
            initial_rn: u32,
            initial_rm: u32,
            shift: Shift,
            set_flags: bool,
            expected_rd: u32,
            expected_flags: u8,
        }

        let vectors = [
            Test {
                initial_rn: 0x12345678,
                initial_rm: 0,
                shift: Shift::lsl(0),
                set_flags: true,
                expected_rd: 0,
                expected_flags: 0b01000,
            },
            Test {
                initial_rn: 0x92345678,
                initial_rm: 0x87654321,
                shift: Shift::lsl(0),
                set_flags: false,
                expected_rd: 0x82244220,
                expected_flags: 0b00000,
            },
            Test {
                initial_rn: 0x12345678,
                initial_rm: 0x87654321,
                shift: Shift::lsl(0),
                set_flags: true,
                expected_rd: 0x02244220,
                expected_flags: 0b00000,
            },
            Test {
                initial_rn: 0xaaaaaaaa,
                initial_rm: 0xaaaa5555,
                shift: Shift::lsl(1),
                set_flags: true,
                expected_rd: 0xaaaa,
                expected_flags: 0b00100,
            },
        ];

        for v in vectors {
            let mut proc = ArmProcessor::new(V7M, 0);
            let rd = RegisterIndex::new_general_random();
            let (rn, rm) = RegisterIndex::pick_two_general_distinct();
            proc.set(rn, v.initial_rn);
            proc.set(rm, v.initial_rm);
            let mut expected = proc.registers.clone();
            expected.set(rd, v.expected_rd);
            expected.psr.set_flags(v.expected_flags);
            AndReg {
                rd,
                rn,
                rm,
                shift: v.shift,
                set_flags: v.set_flags,
                tn: 0,
            }
            .execute(&mut proc)
            .unwrap();
            assert_eq!(proc.registers, expected);
        }
    }
}

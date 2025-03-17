//! Implements TEQ (Test Equivalence) instruction.

use super::Instruction;
use crate::{
    arith::{shift_c, thumb_expand_imm_optc, Shift},
    arm::{ArmProcessor, RunError},
    decoder::DecodeError,
    instructions::{unpredictable, DecodeHelper},
    it_state::ItState,
    registers::RegisterIndex,
};

/// TEQ (immediate) instruction.
///
/// Test Equivalence.
pub struct TeqImm {
    /// Operand register.
    rn: RegisterIndex,
    /// Immediate value to be tested against Rn.
    imm32: u32,
    /// Carry.
    carry: Option<bool>,
}

impl Instruction for TeqImm {
    fn patterns() -> &'static [&'static str] {
        &["11110x001001xxxx0xxx1111xxxxxxxx"]
    }

    fn try_decode(tn: usize, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        debug_assert_eq!(tn, 1);
        let rn = ins.reg4(16);
        let (imm32, carry) =
            thumb_expand_imm_optc((ins.imm1(26) << 11) | (ins.imm3(12) << 8) | ins.imm8(0))?;
        unpredictable(rn.is_sp_or_pc())?;
        Ok(Self { rn, imm32, carry })
    }

    fn execute(&self, proc: &mut ArmProcessor) -> Result<bool, RunError> {
        let result = proc[self.rn] ^ self.imm32;
        proc.registers.xpsr.set_nz(result).set_c_opt(self.carry);
        Ok(false)
    }

    fn name(&self) -> String {
        "teq".into()
    }

    fn args(&self, _pc: u32) -> String {
        format!("{}, #{}", self.rn, self.imm32)
    }
}

/// TEQ (register) instruction.
///
/// Test Equivalence.
pub struct TeqReg {
    /// First operand register.
    rn: RegisterIndex,
    /// Second operand register.
    rm: RegisterIndex,
    /// Shift to be applied to Rm.
    shift: Shift,
}

impl Instruction for TeqReg {
    fn patterns() -> &'static [&'static str] {
        &["111010101001xxxx(0)xxx1111xxxxxxxx"]
    }

    fn try_decode(tn: usize, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        debug_assert_eq!(tn, 1);
        let rn = ins.reg4(16);
        let rm = ins.reg4(0);
        unpredictable(rn.is_sp_or_pc() || rm.is_sp_or_pc())?;
        Ok(Self {
            rn,
            rm,
            shift: Shift::from_bits(ins.imm2(4), (ins.imm3(12) << 2) | ins.imm2(6)),
        })
    }

    fn execute(&self, proc: &mut ArmProcessor) -> Result<bool, RunError> {
        let carry_in = proc.registers.xpsr.c();
        let (shifted, carry) = shift_c(proc[self.rm], self.shift, carry_in);
        let result = proc[self.rn] ^ shifted;
        proc.registers.xpsr.set_nz(result).set_c(carry);
        Ok(false)
    }

    fn name(&self) -> String {
        "teq".into()
    }

    fn args(&self, _pc: u32) -> String {
        format!("{}, {}{}", self.rn, self.rm, self.shift.arg_string())
    }
}

#[cfg(test)]
mod tests {
    use super::TeqImm;
    use crate::{
        arith::Shift,
        arm::{ArmProcessor, ArmVersion},
        instructions::{teq::TeqReg, Instruction},
        registers::RegisterIndex,
    };

    #[test]
    fn test_teq_imm() {
        struct Test {
            imm32: u32,
            carry: Option<bool>,
            initial_c: bool,
            expected_nzcv: (bool, bool, bool, bool),
        }

        let vectors = [
            Test {
                imm32: 0x12345678,
                carry: None,
                initial_c: false,
                expected_nzcv: (false, true, false, false),
            },
            Test {
                imm32: 0x12345678,
                carry: None,
                initial_c: true,
                expected_nzcv: (false, true, true, false),
            },
            Test {
                imm32: 0x80000000,
                carry: Some(true),
                initial_c: false,
                expected_nzcv: (true, false, true, false),
            },
            Test {
                imm32: 0x00000000,
                carry: Some(false),
                initial_c: false,
                expected_nzcv: (false, false, false, false),
            },
        ];

        for v in vectors {
            let mut proc = ArmProcessor::new(ArmVersion::V7M, 0);
            let rn = RegisterIndex::new_general_random();
            proc.set(rn, 0x12345678);
            proc.registers.xpsr.set_c(v.initial_c);
            let mut expected = proc.registers.clone();
            expected.xpsr.set_n(v.expected_nzcv.0);
            expected.xpsr.set_z(v.expected_nzcv.1);
            expected.xpsr.set_c(v.expected_nzcv.2);
            expected.xpsr.set_v(v.expected_nzcv.3);
            TeqImm {
                rn,
                imm32: v.imm32,
                carry: v.carry,
            }
            .execute(&mut proc)
            .unwrap();
            assert_eq!(proc.registers, expected);
        }
    }

    #[test]
    fn test_teq_reg() {
        struct Test {
            shift: Shift,
            initial_rm: u32,
            expected_nzcv: (bool, bool, bool, bool),
        }

        let vectors = [
            Test {
                shift: Shift::lsl(0),
                initial_rm: 0x12345678,
                expected_nzcv: (false, true, false, false),
            },
            Test {
                shift: Shift::lsl(1),
                initial_rm: 0x91a2b3c,
                expected_nzcv: (false, true, false, false),
            },
            Test {
                shift: Shift::lsl(2),
                initial_rm: 0x20000000,
                expected_nzcv: (true, false, false, false),
            },
            Test {
                shift: Shift::ror(2),
                initial_rm: 2,
                expected_nzcv: (true, false, true, false),
            },
        ];

        for v in vectors {
            let mut proc = ArmProcessor::new(ArmVersion::V7M, 0);
            let (rn, rm) = RegisterIndex::pick_two_general_distinct();
            proc.set(rn, 0x12345678);
            proc.set(rm, v.initial_rm);
            let mut expected = proc.registers.clone();
            expected.xpsr.set_n(v.expected_nzcv.0);
            expected.xpsr.set_z(v.expected_nzcv.1);
            expected.xpsr.set_c(v.expected_nzcv.2);
            expected.xpsr.set_v(v.expected_nzcv.3);
            TeqReg {
                rn,
                rm,
                shift: v.shift,
            }
            .execute(&mut proc)
            .unwrap();
            assert_eq!(proc.registers, expected);
        }
    }
}

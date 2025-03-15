//! Implements QADD (Saturating Add) instruction.

use super::Instruction;
use crate::{
    arm::{ArmProcessor, RunError},
    decoder::DecodeError,
    instructions::{rdn_args_string, unpredictable, DecodeHelper},
    it_state::ItState,
    registers::RegisterIndex,
};

/// QADD instruction.
///
/// Saturating Add.
pub struct Qadd {
    /// Destination register.
    rd: RegisterIndex,
    /// First operand register.
    rm: RegisterIndex,
    /// Second operand register.
    rn: RegisterIndex,
}

impl Instruction for Qadd {
    fn patterns() -> &'static [&'static str] {
        &["111110101000xxxx1111xxxx1000xxxx"]
    }

    fn try_decode(tn: usize, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        debug_assert_eq!(tn, 1);
        let rd = ins.reg4(8);
        let rm = ins.reg4(0);
        let rn = ins.reg4(16);
        unpredictable(rd.is_sp_or_pc() || rm.is_sp_or_pc() || rn.is_sp_or_pc())?;
        Ok(Self { rd, rm, rn })
    }

    fn execute(&self, proc: &mut ArmProcessor) -> Result<bool, RunError> {
        let rm = proc.registers[self.rm] as i32;
        let rn = proc.registers[self.rn] as i32;
        let non_saturated = rm.wrapping_add(rn);
        let result = rm.saturating_add(rn);
        proc.registers.set(self.rd, result as u32);
        if result != non_saturated {
            proc.registers.xpsr.set_q(true);
        }
        Ok(false)
    }

    fn name(&self) -> String {
        "qadd".into()
    }

    fn args(&self, _pc: u32) -> String {
        format!("{}, {}", rdn_args_string(self.rd, self.rm), self.rn)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        arm::{ArmProcessor, ArmVersion::V7M},
        instructions::{qadd::Qadd, Instruction},
        registers::RegisterIndex,
    };

    #[test]
    fn test_qadd() {
        struct Test {
            rd: RegisterIndex,
            rm: RegisterIndex,
            rn: RegisterIndex,
            initial_rm: u32,
            initial_rn: u32,
            initial_q: bool,
            expected_rd: u32,
            expected_q: bool,
        }

        let vectors = [
            Test {
                rd: RegisterIndex::R0,
                rm: RegisterIndex::R1,
                rn: RegisterIndex::R2,
                initial_rm: 0x7ffffffe,
                initial_rn: 1,
                initial_q: false,
                expected_rd: 0x7fffffff,
                expected_q: false,
            },
            Test {
                rd: RegisterIndex::R0,
                rm: RegisterIndex::R1,
                rn: RegisterIndex::R2,
                initial_rm: 0x7ffffffe,
                initial_rn: 2,
                initial_q: false,
                expected_rd: 0x7fffffff,
                expected_q: true,
            },
            Test {
                rd: RegisterIndex::R1,
                rm: RegisterIndex::R2,
                rn: RegisterIndex::R3,
                initial_rm: 0x80000001,
                initial_rn: 0xffffffff,
                initial_q: false,
                expected_rd: 0x80000000,
                expected_q: false,
            },
            Test {
                rd: RegisterIndex::R2,
                rm: RegisterIndex::R3,
                rn: RegisterIndex::R4,
                initial_rm: 0x80000001,
                initial_rn: 0xfffffffe,
                initial_q: false,
                expected_rd: 0x80000000,
                expected_q: true,
            },
            Test {
                rd: RegisterIndex::R3,
                rm: RegisterIndex::R4,
                rn: RegisterIndex::R5,
                initial_rm: 0x80000001,
                initial_rn: 0xffffffff,
                initial_q: true,
                expected_rd: 0x80000000,
                expected_q: true,
            },
            Test {
                rd: RegisterIndex::R4,
                rm: RegisterIndex::R5,
                rn: RegisterIndex::R6,
                initial_rm: 0x7fffffff,
                initial_rn: 0xffffffff,
                initial_q: false,
                expected_rd: 0x7ffffffe,
                expected_q: false,
            },
        ];

        for v in vectors {
            let mut proc = ArmProcessor::new(V7M, 0);
            proc.registers.set(v.rm, v.initial_rm);
            proc.registers.set(v.rn, v.initial_rn);
            proc.registers.xpsr.set_q(v.initial_q);
            let mut expected = proc.registers.clone();
            expected.xpsr.set_q(v.expected_q);
            expected.set(v.rd, v.expected_rd);
            Qadd {
                rd: v.rd,
                rm: v.rm,
                rn: v.rn,
            }
            .execute(&mut proc)
            .unwrap();
            assert_eq!(proc.registers, expected);
        }
    }
}

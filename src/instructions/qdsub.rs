//! Implements QDSUB (Saturating Double and Subtract) instruction.

use crate::{
    arith::signed_sat_q,
    arm::{ArmProcessor, RunError},
    decoder::DecodeError,
    instructions::{rdn_args_string, unpredictable, DecodeHelper},
    it_state::ItState,
    registers::RegisterIndex,
};

use super::Instruction;

/// QDSUB instruction.
///
/// Saturating Double and Subtract.
pub struct Qdsub {
    /// Destination register.
    rd: RegisterIndex,
    /// First operand register.
    rm: RegisterIndex,
    /// Second operand register.
    rn: RegisterIndex,
}

impl Instruction for Qdsub {
    fn patterns() -> &'static [&'static str] {
        &["111110101000xxxx1111xxxx1011xxxx"]
    }

    fn try_decode(tn: usize, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        debug_assert_eq!(tn, 1);
        let rd = ins.reg4(8);
        let rm = ins.reg4(0);
        let rn = ins.reg4(16);
        unpredictable(rd.is_sp_or_pc() || rn.is_sp_or_pc() || rm.is_sp_or_pc())?;
        Ok(Self { rd, rm, rn })
    }

    fn execute(&self, proc: &mut ArmProcessor) -> Result<bool, RunError> {
        let (doubled, sat1) = signed_sat_q(2 * (proc[self.rn] as i32 as i64), 32);
        let (result, sat2) = signed_sat_q(proc[self.rm] as i32 as i64 - doubled, 32);
        proc.set(self.rd, result as u32);
        if sat1 || sat2 {
            proc.registers.psr.set_q(true);
        }
        Ok(false)
    }

    fn name(&self) -> String {
        "qdsub".into()
    }

    fn args(&self, _pc: u32) -> String {
        format!("{}, {}", rdn_args_string(self.rd, self.rm), self.rn)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        arm::{ArmProcessor, ArmVersion::V7M},
        instructions::{qdsub::Qdsub, Instruction},
        registers::RegisterIndex,
    };

    #[test]
    fn test_qdsub() {
        struct Test {
            initial_rm: u32,
            initial_rn: u32,
            initial_q: bool,
            expected_rd: u32,
            expected_q: bool,
        }

        let vectors = [
            Test {
                initial_rm: 5,
                initial_rn: 4,
                initial_q: false,
                expected_rd: 0xfffffffd,
                expected_q: false,
            },
            Test {
                initial_rm: 0xffffffff,
                initial_rn: 0x3fffffff,
                initial_q: false,
                expected_rd: 0x80000001,
                expected_q: false,
            },
            Test {
                initial_rm: 0xfffffffd,
                initial_rn: 0x3fffffff,
                initial_q: false,
                expected_rd: 0x80000000,
                expected_q: true,
            },
            Test {
                initial_rm: 1,
                initial_rn: 0xc0000001,
                initial_q: false,
                expected_rd: 0x7fffffff,
                expected_q: false,
            },
            Test {
                initial_rm: 2,
                initial_rn: 0xc0000001,
                initial_q: false,
                expected_rd: 0x7fffffff,
                expected_q: true,
            },
            Test {
                initial_rm: 0x80000000,
                initial_rn: 0x80000000,
                initial_q: false,
                expected_rd: 0x0,
                expected_q: true,
            },
            //// Check Q remains untouched when no saturation occurs.
            Test {
                initial_rm: 5,
                initial_rn: 4,
                initial_q: true,
                expected_rd: 0xfffffffd,
                expected_q: true,
            },
        ];

        for v in vectors {
            let mut proc = ArmProcessor::new(V7M, 0);
            let rd = RegisterIndex::new_general_random();
            let (rm, rn) = RegisterIndex::pick_two_general_distinct();
            proc.set(rm, v.initial_rm);
            proc.set(rn, v.initial_rn);
            proc.registers.psr.set_q(v.initial_q);
            let mut expected = proc.registers.clone();
            expected.set(rd, v.expected_rd);
            expected.psr.set_q(v.expected_q);
            Qdsub { rd, rm, rn }.execute(&mut proc).unwrap();
            assert_eq!(proc.registers, expected);
        }
    }
}

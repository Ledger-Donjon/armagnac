//! Implements QDADD (Saturating Double and Add) instruction.

use super::Encoding::{self, T1};
use super::Instruction;
use super::{
    ArmVersion::{V7EM, V8M},
    Pattern,
};
use crate::{
    arith::signed_sat_q,
    core::ItState,
    core::{Processor, Effect, RunError},
    decoder::DecodeError,
    instructions::{unpredictable, DecodeHelper},
    registers::RegisterIndex,
};

/// QDADD instruction.
///
/// Saturating Double and Add.
pub struct Qdadd {
    /// Destination register.
    rd: RegisterIndex,
    /// First operand register.
    rm: RegisterIndex,
    /// Second operand register.
    rn: RegisterIndex,
}

impl Instruction for Qdadd {
    fn patterns() -> &'static [Pattern] {
        &[Pattern {
            encoding: T1,
            versions: &[V7EM, V8M],
            expression: "111110101000xxxx1111xxxx1001xxxx",
        }]
    }

    fn try_decode(encoding: Encoding, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        debug_assert_eq!(encoding, T1);
        let rd = ins.reg4(8);
        let rm = ins.reg4(0);
        let rn = ins.reg4(16);
        unpredictable(rd.is_sp_or_pc() || rn.is_sp_or_pc() || rm.is_sp_or_pc())?;
        Ok(Self { rd, rm, rn })
    }

    fn execute(&self, proc: &mut Processor) -> Result<Effect, RunError> {
        let (doubled, sat1) = signed_sat_q(2 * (proc[self.rn] as i32 as i64), 32);
        let (result, sat2) = signed_sat_q(proc[self.rm] as i32 as i64 + doubled, 32);
        proc.set(self.rd, result as u32);
        if sat1 || sat2 {
            proc.registers.psr.set_q(true);
        }
        Ok(Effect::None)
    }

    fn name(&self) -> String {
        "qdadd".into()
    }

    fn args(&self, _pc: u32) -> String {
        format!("{}, {}, {}", self.rd, self.rm, self.rn)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        core::{Processor, Config},
        instructions::{qdadd::Qdadd, Instruction},
        registers::RegisterIndex,
    };

    #[test]
    fn test_qdadd() {
        struct Test {
            initial_rm: u32,
            initial_rn: u32,
            initial_q: bool,
            expected_rd: u32,
            expected_q: bool,
        }

        let vectors = [
            Test {
                initial_rm: 3,
                initial_rn: 2,
                initial_q: false,
                expected_rd: 7,
                expected_q: false,
            },
            Test {
                initial_rm: 0,
                initial_rn: 0x3fffffff,
                initial_q: false,
                expected_rd: 0x7ffffffe,
                expected_q: false,
            },
            Test {
                initial_rm: 2,
                initial_rn: 0x3fffffff,
                initial_q: false,
                expected_rd: 0x7fffffff,
                expected_q: true,
            },
            Test {
                initial_rm: 0,
                initial_rn: 0x40000000,
                initial_q: false,
                expected_rd: 0x7fffffff,
                expected_q: true,
            },
            Test {
                initial_rm: 0xfffffffe,
                initial_rn: 0xc0000001,
                initial_q: false,
                expected_rd: 0x80000000,
                expected_q: false,
            },
            Test {
                initial_rm: 0xfffffffd,
                initial_rn: 0xc0000001,
                initial_q: false,
                expected_rd: 0x80000000,
                expected_q: true,
            },
            // Check Q remains untouched when no saturation occurs.
            Test {
                initial_rm: 3,
                initial_rn: 2,
                initial_q: true,
                expected_rd: 7,
                expected_q: true,
            },
        ];

        for v in vectors {
            let mut proc = Processor::new(Config::v7em());
            let rd = RegisterIndex::new_general_random();
            let (rm, rn) = RegisterIndex::pick_two_general_distinct();
            proc.set(rm, v.initial_rm);
            proc.set(rn, v.initial_rn);
            proc.registers.psr.set_q(v.initial_q);
            let mut expected = proc.registers.clone();
            expected.set(rd, v.expected_rd);
            expected.psr.set_q(v.expected_q);
            Qdadd { rd, rm, rn }.execute(&mut proc).unwrap();
            assert_eq!(proc.registers, expected);
        }
    }
}

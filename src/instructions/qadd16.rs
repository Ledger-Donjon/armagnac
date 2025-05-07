//! Implements QADD16 (Saturating Add 16) instruction.

use super::Instruction;
use super::{
    ArmVersion::{V7EM, V8M},
    Pattern,
};
use crate::{
    arm::{ArmProcessor, RunError},
    decoder::DecodeError,
    instructions::{unpredictable, DecodeHelper},
    it_state::ItState,
    registers::RegisterIndex,
};

/// QADD16 instruction.
///
/// Saturating Add 16.
pub struct Qadd16 {
    /// Destination register.
    rd: RegisterIndex,
    /// First operand register.
    rn: RegisterIndex,
    /// Second operand register.
    rm: RegisterIndex,
}

impl Instruction for Qadd16 {
    fn patterns() -> &'static [Pattern] {
        &[Pattern {
            tn: 1,
            versions: &[V7EM, V8M],
            expression: "111110101001xxxx1111xxxx0001xxxx",
        }]
    }

    fn try_decode(tn: usize, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        debug_assert_eq!(tn, 1);
        let rd = ins.reg4(8);
        let rn = ins.reg4(16);
        let rm = ins.reg4(0);
        unpredictable(rd.is_sp_or_pc() || rn.is_sp_or_pc() || rm.is_sp_or_pc())?;
        Ok(Self { rd, rn, rm })
    }

    fn execute(&self, proc: &mut ArmProcessor) -> Result<bool, RunError> {
        let rn = proc[self.rn];
        let rm = proc[self.rm];
        let sum1 = (rn as i16).saturating_add(rm as i16);
        let sum2 = ((rn >> 16) as i16).saturating_add((rm >> 16) as i16);
        let result = ((sum2 as u32) << 16) | (sum1 as u16) as u32;
        proc.set(self.rd, result);
        Ok(false)
    }

    fn name(&self) -> String {
        "qadd16".into()
    }

    fn args(&self, _pc: u32) -> String {
        format!("{}, {}, {}", self.rd, self.rn, self.rm)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        arm::{ArmProcessor, Config},
        instructions::{qadd16::Qadd16, Instruction},
        registers::RegisterIndex,
    };

    #[test]
    fn test_qadd16() {
        struct Test {
            initial_rm: u32,
            initial_rn: u32,
            expected_rd: u32,
        }

        let vectors = [
            Test {
                initial_rm: 0x7ffe7ffe,
                initial_rn: 0x00010001,
                expected_rd: 0x7fff7fff,
            },
            Test {
                initial_rm: 0x7ffe7ffe,
                initial_rn: 0x00020002,
                expected_rd: 0x7fff7fff,
            },
            Test {
                initial_rm: 0x80018001,
                initial_rn: 0xffffffff,
                expected_rd: 0x80008000,
            },
            Test {
                initial_rm: 0x80018001,
                initial_rn: 0xfff0fff0,
                expected_rd: 0x80008000,
            },
            Test {
                initial_rm: 0x11112222,
                initial_rn: 0x33334444,
                expected_rd: 0x44446666,
            },
        ];

        for v in vectors {
            let mut proc = ArmProcessor::new(Config::v7em());
            let rd = RegisterIndex::new_general_random();
            let (rm, rn) = RegisterIndex::pick_two_general_distinct();
            proc.set(rm, v.initial_rm);
            proc.set(rn, v.initial_rn);
            let mut expected = proc.registers.clone();
            expected.set(rd, v.expected_rd);
            Qadd16 { rd, rm, rn }.execute(&mut proc).unwrap();
            assert_eq!(proc.registers, expected);
        }
    }
}

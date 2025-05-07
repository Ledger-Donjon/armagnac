//! Implements QSUB16 (Saturating Subtract 16) instruction.

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

/// QSUB16 instruction.
///
/// Saturating Subtract 16.
pub struct Qsub16 {
    /// Destination register.
    rd: RegisterIndex,
    /// First operand register.
    rn: RegisterIndex,
    /// Second operand register.
    rm: RegisterIndex,
}

impl Instruction for Qsub16 {
    fn patterns() -> &'static [Pattern] {
        &[Pattern {
            tn: 1,
            versions: &[V7EM, V8M],
            expression: "111110101101xxxx1111xxxx0001xxxx",
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
        let diff1 = (rn as i16).saturating_sub(rm as i16);
        let diff2 = ((rn >> 16) as i16).saturating_sub((rm >> 16) as i16);
        let result = ((diff2 as u32) << 16) | diff1 as u16 as u32;
        proc.set(self.rd, result);
        Ok(false)
    }

    fn name(&self) -> String {
        "qsub16".into()
    }

    fn args(&self, _pc: u32) -> String {
        format!("{}, {}, {}", self.rd, self.rn, self.rm)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        arm::{ArmProcessor, Config},
        instructions::{qsub16::Qsub16, Instruction},
        registers::RegisterIndex,
    };

    #[test]
    fn test_qsub16() {
        struct Test {
            initial_rn: u32,
            initial_rm: u32,
            expected_rd: u32,
        }

        let vectors = [
            Test {
                initial_rn: 0x80018001,
                initial_rm: 0x00010001,
                expected_rd: 0x80008000,
            },
            Test {
                initial_rn: 0x80018001,
                initial_rm: 0x00020002,
                expected_rd: 0x80008000,
            },
            Test {
                initial_rn: 0x7ffe7ffe,
                initial_rm: 0xffffffff,
                expected_rd: 0x7fff7fff,
            },
            Test {
                initial_rn: 0x7ffe7ffe,
                initial_rm: 0xfffefffe,
                expected_rd: 0x7fff7fff,
            },
            Test {
                initial_rn: 0x33335555,
                initial_rm: 0x11112222,
                expected_rd: 0x22223333,
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
            Qsub16 { rd, rm, rn }.execute(&mut proc).unwrap();
            assert_eq!(proc.registers, expected);
        }
    }
}

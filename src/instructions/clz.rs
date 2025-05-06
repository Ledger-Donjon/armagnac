//! Implements CLZ (Count Leading Zeros) instruction.

use super::Instruction;
use super::{
    ArmVersion::{V7EM, V7M, V8M},
    Pattern,
};
use crate::{
    arm::{ArmProcessor, RunError},
    decoder::DecodeError,
    instructions::{unpredictable, DecodeHelper},
    it_state::ItState,
    registers::RegisterIndex,
};

/// CLZ instruction.
///
/// Count Leading Zeros.
pub struct Clz {
    /// Destination register
    rd: RegisterIndex,
    /// Operand register
    rm: RegisterIndex,
}

impl Instruction for Clz {
    fn patterns() -> &'static [Pattern] {
        &[Pattern {
            tn: 1,
            versions: &[V7M, V7EM, V8M],
            expression: "111110101011xxxx1111xxxx1000xxxx",
        }]
    }

    fn try_decode(tn: usize, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        debug_assert_eq!(tn, 1);
        let rm1 = ins.reg4(16);
        let rm2 = ins.reg4(0);
        let rd = ins.reg4(8);
        unpredictable(rm1 != rm2)?;
        unpredictable(rm1.is_sp_or_pc() || rd.is_sp_or_pc())?;
        Ok(Self { rd, rm: rm1 })
    }

    fn execute(&self, proc: &mut ArmProcessor) -> Result<bool, RunError> {
        proc.set(self.rd, proc[self.rm].leading_zeros());
        Ok(false)
    }

    fn name(&self) -> String {
        "clz".into()
    }

    fn args(&self, _pc: u32) -> String {
        format!("{}, {}", self.rd, self.rm)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        arm::{ArmProcessor, ArmVersion},
        instructions::Instruction,
        registers::RegisterIndex,
    };

    use super::Clz;

    #[test]
    fn test_clz() {
        let vectors = [
            (0xffffffff, 0),
            (0x40000000, 1),
            (0x20000000, 2),
            (0x008aaaaa, 8),
            (0x00008aaa, 16),
            (0x0000008a, 24),
            (0x00000002, 30),
            (0x00000001, 31),
            (0x00000000, 32),
        ];
        for v in vectors {
            let mut proc = ArmProcessor::new(ArmVersion::V7M, 0);
            let rm = RegisterIndex::new_general_random();
            let rd = RegisterIndex::new_general_random();
            proc.set(rm, v.0);
            let mut expected = proc.registers.clone();
            expected.set(rd, v.1);
            Clz { rd, rm }.execute(&mut proc).unwrap();
            assert_eq!(proc.registers, expected);
        }
    }
}

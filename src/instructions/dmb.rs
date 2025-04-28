//! Implements DMB (Data Memory Barrier) instruction.

use super::Instruction;
use super::{
    ArmVersion::{V6M, V7M, V8M},
    Pattern,
};
use crate::instructions::DecodeHelper;

/// DMB instruction.
///
/// Data Memory Barrier.
pub struct Dmb {
    /// Option field.
    /// 4-bit.
    /// The only valid value defined by ARM is 0xf. All other values are reserved.
    option: u8,
}

impl Instruction for Dmb {
    fn patterns() -> &'static [Pattern] {
        &[Pattern {
            tn: 1,
            versions: &[V6M, V7M, V8M],
            expression: "111100111011(1)(1)(1)(1)10(0)0(1)(1)(1)(1)0101xxxx",
        }]
    }

    fn try_decode(
        tn: usize,
        ins: u32,
        _state: crate::it_state::ItState,
    ) -> Result<Self, crate::decoder::DecodeError> {
        debug_assert_eq!(tn, 1);
        Ok(Self {
            option: ins.imm4(0) as u8,
        })
    }

    fn execute(&self, _proc: &mut crate::arm::ArmProcessor) -> Result<bool, crate::arm::RunError> {
        Ok(false)
    }

    fn name(&self) -> String {
        "dmb".into()
    }

    fn args(&self, _pc: u32) -> String {
        if self.option == 0xf {
            "sy".into()
        } else {
            // Other values are reserved, show them as an integer.
            format!("#0x{:x}", self.option)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Dmb;
    use crate::{
        arm::{ArmProcessor, ArmVersion::V7M},
        instructions::Instruction,
    };

    #[test]
    fn test_dmb() {
        // Check that the instruction does nothing.
        let mut proc = ArmProcessor::new(V7M, 0);
        let expected = proc.registers.clone();
        for option in 0..=0xf {
            Dmb { option }.execute(&mut proc).unwrap();
            assert_eq!(proc.registers, expected)
        }
    }
}

//! Implements DMB (Data Memory Barrier) instruction.

use super::Encoding::{self, T1};
use super::Instruction;
use super::{
    ArmVersion::{V6M, V7EM, V7M, V8M},
    Pattern,
};
use crate::core::{ArmProcessor, Effect, RunError};
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
            encoding: T1,
            versions: &[V6M, V7M, V7EM, V8M],
            expression: "111100111011(1)(1)(1)(1)10(0)0(1)(1)(1)(1)0101xxxx",
        }]
    }

    fn try_decode(
        encoding: Encoding,
        ins: u32,
        _state: crate::core::ItState,
    ) -> Result<Self, crate::decoder::DecodeError> {
        debug_assert_eq!(encoding, T1);
        Ok(Self {
            option: ins.imm4(0) as u8,
        })
    }

    fn execute(&self, _proc: &mut ArmProcessor) -> Result<Effect, RunError> {
        Ok(Effect::None)
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
        core::{ArmProcessor, Config},
        instructions::Instruction,
    };

    #[test]
    fn test_dmb() {
        // Check that the instruction does nothing.
        let mut proc = ArmProcessor::new(Config::v7m());
        let expected = proc.registers.clone();
        for option in 0..=0xf {
            Dmb { option }.execute(&mut proc).unwrap();
            assert_eq!(proc.registers, expected)
        }
    }
}

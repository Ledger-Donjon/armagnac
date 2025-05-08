//! Implements YIELD instruction.

use super::{
    Encoding::{self, T1, T2},
    Instruction, Pattern, Qualifier,
};
use crate::{
    arm::{
        ArmProcessor,
        ArmVersion::{V6M, V7EM, V7M, V8M},
        RunError,
    },
    decoder::DecodeError,
    it_state::ItState,
    qualifier_wide_match,
};

/// Yield instruction.
///
/// Yield instruction is treated as NOP in Armagnac.
pub struct Yield {
    /// Encoding.
    encoding: Encoding,
}

impl Instruction for Yield {
    fn patterns() -> &'static [super::Pattern] {
        &[
            Pattern {
                encoding: T1,
                versions: &[V6M, V7M, V7EM, V8M],
                expression: "1011111100010000",
            },
            Pattern {
                encoding: T2,
                versions: &[V7M, V7EM, V8M],
                expression: "111100111010(1)(1)(1)(1)10(0)0(0)00000000001",
            },
        ]
    }

    fn try_decode(encoding: Encoding, _ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        debug_assert!(encoding == T1 || encoding == T2);
        Ok(Self { encoding })
    }

    fn execute(&self, _proc: &mut ArmProcessor) -> Result<bool, RunError> {
        Ok(false)
    }

    fn name(&self) -> String {
        "yield".into()
    }

    fn qualifier(&self) -> Qualifier {
        qualifier_wide_match!(self.encoding, T2)
    }

    fn args(&self, _pc: u32) -> String {
        "".into()
    }
}

#[cfg(test)]
mod tests {
    use super::Yield;
    use crate::{
        arm::{ArmProcessor, Config},
        instructions::{Encoding::DontCare, Instruction},
    };

    #[test]
    fn test_yield() {
        let mut proc = ArmProcessor::new(Config::v7m());
        let expected = proc.registers.clone();
        Yield { encoding: DontCare }.execute(&mut proc).unwrap();
        assert_eq!(proc.registers, expected);
    }
}

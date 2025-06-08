//! Implements WFE (Wait For Event) instruction.

use super::{
    Encoding::{self, T1, T2},
    Instruction, Pattern,
};
use crate::{
    core::ItState,
    core::{
        ArmProcessor,
        ArmVersion::{V6M, V7EM, V7M, V8M},
        Effect, RunError,
    },
    decoder::DecodeError,
    qualifier_wide_match,
};

/// WFE instruction.
///
/// Wait For Event.
pub struct Wfe {
    /// Encoding.
    encoding: Encoding,
}

impl Instruction for Wfe {
    fn patterns() -> &'static [Pattern] {
        &[
            Pattern {
                encoding: T1,
                versions: &[V6M, V7M, V7EM, V8M],
                expression: "1011111100100000",
            },
            Pattern {
                encoding: T2,
                versions: &[V7M, V7EM, V8M],
                expression: "111100111010(1)(1)(1)(1)10(0)0(0)00000000010",
            },
        ]
    }

    fn try_decode(encoding: Encoding, _ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        debug_assert!((encoding == T1) || (encoding == T2));
        Ok(Self { encoding })
    }

    fn execute(&self, proc: &mut ArmProcessor) -> Result<Effect, RunError> {
        if proc.registers.event {
            proc.registers.event = false;
            Ok(Effect::None)
        } else {
            Ok(Effect::WaitForEvent)
        }
    }

    fn name(&self) -> String {
        "wfe".into()
    }

    fn qualifier(&self) -> super::Qualifier {
        qualifier_wide_match!(self.encoding, T2)
    }

    fn args(&self, _pc: u32) -> String {
        "".into()
    }
}

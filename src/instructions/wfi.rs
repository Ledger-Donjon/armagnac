//! Implements WFI (Wait For Interrupt) instruction.

use super::{
    Encoding::{self, T1, T2},
    Instruction, Pattern,
};
use crate::{
    core::ItState,
    core::{
        Processor,
        ArmVersion::{V6M, V7EM, V7M, V8M},
        Effect, RunError,
    },
    decoder::DecodeError,
    qualifier_wide_match,
};

/// WFI instruction.
///
/// Wait For Interrupt.
pub struct Wfi {
    /// Encoding.
    encoding: Encoding,
}

impl Instruction for Wfi {
    fn patterns() -> &'static [Pattern] {
        &[
            Pattern {
                encoding: T1,
                versions: &[V6M, V7M, V7EM, V8M],
                expression: "1011111100110000",
            },
            Pattern {
                encoding: T2,
                versions: &[V7M, V7EM, V8M],
                expression: "111100111010(1)(1)(1)(1)10(0)0(0)00000000011",
            },
        ]
    }

    fn try_decode(encoding: Encoding, _ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        debug_assert!((encoding == T1) || (encoding == T2));
        Ok(Self { encoding })
    }

    fn execute(&self, _proc: &mut Processor) -> Result<Effect, RunError> {
        Ok(Effect::WaitForInterrupt)
    }

    fn name(&self) -> String {
        "wfi".into()
    }

    fn qualifier(&self) -> super::Qualifier {
        qualifier_wide_match!(self.encoding, T2)
    }

    fn args(&self, _pc: u32) -> String {
        "".into()
    }
}

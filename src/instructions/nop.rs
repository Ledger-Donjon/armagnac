//! Implements NOP (No Operation) instruction.

use super::Encoding::{self, T1, T2};
use super::Instruction;
use super::{
    ArmVersion::{V6M, V7EM, V7M, V8M},
    Pattern,
};
use crate::arm::Effect;
use crate::qualifier_wide_match;
use crate::{
    arm::{ArmProcessor, RunError},
    decoder::DecodeError,
    instructions::ItState,
};

/// NOP instruction.
pub struct Nop {
    /// Encoding.
    encoding: Encoding,
}

impl Instruction for Nop {
    fn patterns() -> &'static [Pattern] {
        // TODO: encoding T2 for ArmV7-M and ArmV8-M.
        &[
            Pattern {
                encoding: T1,
                versions: &[V6M, V7M, V7EM, V8M],
                expression: "1011111100000000",
            },
            Pattern {
                encoding: T2,
                versions: &[V7M, V7EM, V8M],
                expression: "111100111010(1)(1)(1)(1)10(0)0(0)00000000000",
            },
        ]
    }

    fn try_decode(encoding: Encoding, _ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        debug_assert!((encoding == T1) || (encoding == T2));
        Ok(Self { encoding })
    }

    fn execute(&self, _proc: &mut ArmProcessor) -> Result<Effect, RunError> {
        Ok(Effect::None)
    }

    fn name(&self) -> String {
        "nop".into()
    }

    fn qualifier(&self) -> super::Qualifier {
        qualifier_wide_match!(self.encoding, T2)
    }

    fn args(&self, _pc: u32) -> String {
        "".into()
    }
}

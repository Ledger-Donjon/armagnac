//! Implements SEV (Send Event) instruction.

use super::Encoding::{self, T1, T2};
use super::Instruction;
use super::{
    ArmVersion::{V6M, V7EM, V7M, V8M},
    Pattern,
};
use crate::arm::Effect;
use crate::{
    arm::{ArmProcessor, RunError},
    decoder::DecodeError,
    instructions::ItState,
};

/// SEV instruction.
pub struct Sev {}

impl Instruction for Sev {
    fn patterns() -> &'static [Pattern] {
        &[
            Pattern {
                encoding: T1,
                versions: &[V6M, V7M, V7EM, V8M],
                expression: "1011111101000000",
            },
            Pattern {
                encoding: T2,
                versions: &[V7M, V7EM, V8M],
                expression: "111100111010(1)(1)(1)(1)10(0)0(0)00000000100",
            },
        ]
    }

    fn try_decode(encoding: Encoding, _ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        debug_assert!(encoding == T1 || encoding == T2);
        Ok(Self {})
    }

    fn execute(&self, _proc: &mut ArmProcessor) -> Result<Effect, RunError> {
        todo!()
    }

    fn name(&self) -> String {
        "sev".into()
    }

    fn args(&self, _pc: u32) -> String {
        "".into()
    }
}

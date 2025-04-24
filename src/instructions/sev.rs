//! Implements SEV (Send Event) instruction.

use super::Instruction;
use super::{
    ArmVersion::{V6M, V7M, V8M},
    Pattern,
};
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
                tn: 1,
                versions: &[V6M, V7M, V8M],
                expression: "1011111101000000",
            },
            Pattern {
                tn: 2,
                versions: &[V7M, V8M],
                expression: "111100111010(1)(1)(1)(1)10(0)0(0)00000000100",
            },
        ]
    }

    fn try_decode(tn: usize, _ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        debug_assert!(tn == 1 || tn == 2);
        Ok(Self {})
    }

    fn execute(&self, _proc: &mut ArmProcessor) -> Result<bool, RunError> {
        todo!()
    }

    fn name(&self) -> String {
        "sev".into()
    }

    fn args(&self, _pc: u32) -> String {
        "".into()
    }
}

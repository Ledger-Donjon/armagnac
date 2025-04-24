//! Implements NOP (No Operation) instruction.

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

/// NOP instruction.
pub struct Nop {}

impl Instruction for Nop {
    fn patterns() -> &'static [Pattern] {
        // TODO: encoding T2 for ArmV7-M and ArmV8-M.
        &[
            Pattern {
                tn: 1,
                versions: &[V6M, V7M, V8M],
                expression: "1011111100000000",
            },
            Pattern {
                tn: 2,
                versions: &[V7M, V8M],
                expression: "111100111010(1)(1)(1)(1)10(0)0(0)00000000000",
            },
        ]
    }

    fn try_decode(_tn: usize, _ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        Ok(Self {})
    }

    fn execute(&self, _proc: &mut ArmProcessor) -> Result<bool, RunError> {
        Ok(false)
    }

    fn name(&self) -> String {
        "nop".into()
    }

    fn args(&self, _pc: u32) -> String {
        "".into()
    }
}

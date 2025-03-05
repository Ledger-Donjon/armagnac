//! Implements NOP (No Operation) instruction.

use crate::{
    arm::{ArmProcessor, RunError},
    decoder::DecodeError,
    instructions::ItState,
};

use super::Instruction;

/// NOP instruction.
pub struct Nop {}

impl Instruction for Nop {
    fn patterns() -> &'static [&'static str] {
        &["1011111100000000"]
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

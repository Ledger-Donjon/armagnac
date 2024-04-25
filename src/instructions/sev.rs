//! Implements SEV instruction.

use crate::{
    arm::{Arm7Processor, RunError},
    decoder::DecodeError,
    instructions::ItState,
};

use super::Instruction;

/// SEV instruction.
pub struct Sev {}

impl Instruction for Sev {
    fn patterns() -> &'static [&'static str] {
        &[
            "1011111101000000",
            "111100111010(1)(1)(1)(1)10(0)0(0)00000000000",
        ]
    }

    fn try_decode(tn: usize, _ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        debug_assert!(tn == 1 || tn == 2);
        Ok(Self {})
    }

    fn execute(&self, _proc: &mut Arm7Processor) -> Result<bool, RunError> {
        todo!()
    }

    fn name(&self) -> String {
        "sev".into()
    }

    fn args(&self, _pc: u32) -> String {
        "".into()
    }
}

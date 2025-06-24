//! Implements CLREX (Clear Exclusive) instruction.

use super::Encoding::T1;
use crate::{
    core::{
        ArmProcessor,
        ArmVersion::{V7EM, V7M, V8M},
        Effect, ItState, MonitorState, RunError,
    },
    decoder::DecodeError,
    instructions::{Encoding, Instruction, Pattern},
};

/// CLREX instruction.
///
/// Clear Exclusive.
pub struct Clrex {}

impl Instruction for Clrex {
    fn patterns() -> &'static [Pattern] {
        &[Pattern {
            encoding: T1,
            versions: &[V7M, V7EM, V8M],
            expression: "111100111011(1)(1)(1)(1)10(0)0(1)(1)(1)(1)0010(1)(1)(1)(1)",
        }]
    }

    fn try_decode(encoding: Encoding, _ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        debug_assert_eq!(encoding, T1);
        Ok(Self {})
    }

    fn execute(&self, proc: &mut ArmProcessor) -> Result<Effect, RunError> {
        proc.local_monitor.state = MonitorState::OpenAccess;
        Ok(Effect::None)
    }

    fn name(&self) -> String {
        "clrex".into()
    }

    fn args(&self, _pc: u32) -> String {
        "".into()
    }
}

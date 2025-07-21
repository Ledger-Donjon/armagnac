//! Implements SSBB (Speculative Store Bypass Barrier) instruction.

use crate::{
    core::{
        ArmVersion::{V7EM, V7M, V8M},
        Effect, ItState, Processor, RunError,
    },
    decoder::DecodeError,
    instructions::{
        Encoding::{self, T1},
        Instruction, Pattern,
    },
};

/// SSBB instruction.
///
/// Speculative Store Bypass Barrier
pub struct Ssbb {}

impl Instruction for Ssbb {
    fn patterns() -> &'static [super::Pattern] {
        &[Pattern {
            encoding: T1,
            versions: &[V7M, V7EM, V8M],
            expression: "111100111011(1)(1)(1)(1)10(0)0(1)(1)(1)(1)01000000",
        }]
    }

    fn try_decode(encoding: Encoding, _ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        debug_assert_eq!(encoding, T1);
        Ok(Self {})
    }

    fn execute(&self, _proc: &mut Processor) -> Result<Effect, RunError> {
        Ok(Effect::None)
    }

    fn name(&self) -> String {
        "ssbb".into()
    }

    fn args(&self, _pc: u32) -> String {
        "".into()
    }
}

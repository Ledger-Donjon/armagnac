//! Implements CSDB (Consumption of Speculative Data Barrier) instruction.

/// CSDB instruction.
///
/// Consumption of Speculative Data Barrier.
pub struct Csdb {}

use crate::{
    core::{
        ArmVersion::{V7EM, V7M, V8M},
        Effect, ItState, Processor, RunError,
    },
    decoder::DecodeError,
    instructions::{
        unpredictable,
        Encoding::{self, T1},
        Instruction, Pattern,
    },
};

impl Instruction for Csdb {
    fn patterns() -> &'static [Pattern] {
        &[Pattern {
            encoding: T1,
            versions: &[V7M, V7EM, V8M],
            expression: "111100111010(1)(1)(1)(1)10(0)0(0)00000010100",
        }]
    }

    fn try_decode(encoding: Encoding, _ins: u32, state: ItState) -> Result<Self, DecodeError> {
        debug_assert_eq!(encoding, T1);
        unpredictable(state.in_it_block())?;
        Ok(Self {})
    }

    fn execute(&self, _proc: &mut Processor) -> Result<Effect, RunError> {
        Ok(Effect::None)
    }

    fn name(&self) -> String {
        "csdb".into()
    }

    fn args(&self, _pc: u32) -> String {
        "".into()
    }
}

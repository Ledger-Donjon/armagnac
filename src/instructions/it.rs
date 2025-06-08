//! Implements IT (If Then) instruction.

use super::Encoding::{self, T1};
use super::Instruction;
use super::{
    ArmVersion::{V7EM, V7M, V8M},
    Pattern,
};
use crate::{
    core::Condition,
    core::{ArmProcessor, Effect, RunError},
    core::{ItState, ItThenElse},
    decoder::DecodeError,
    instructions::{other, unpredictable},
};

// IT instruction.
//
// If Then.
pub struct It {
    /// IT state to be set.
    state: ItState,
}

impl Instruction for It {
    fn patterns() -> &'static [Pattern] {
        &[Pattern {
            encoding: T1,
            versions: &[V7M, V7EM, V8M],
            expression: "10111111xxxxxxxx",
        }]
    }

    fn try_decode(encoding: Encoding, ins: u32, state: ItState) -> Result<Self, DecodeError> {
        debug_assert_eq!(encoding, T1);
        let mask = ins & 0xf;
        other(mask == 0)?;
        let new_state = ItState::try_new((ins & 0xff) as u8);
        unpredictable(new_state.is_err())?;
        unpredictable(state.in_it_block())?;
        Ok(Self {
            state: new_state.unwrap(),
        })
    }

    fn condition(&self) -> Option<Condition> {
        Some(Condition::Always)
    }

    fn execute(&self, proc: &mut ArmProcessor) -> Result<Effect, RunError> {
        proc.registers.psr.set_it_state(self.state);
        Ok(Effect::None)
    }

    fn name(&self) -> String {
        let mut name: String = "it".into();
        for x in &self.state.as_then_else() {
            name.push(match x {
                ItThenElse::Then => 't',
                ItThenElse::Else => 'e',
            });
        }
        name
    }

    fn args(&self, _pc: u32) -> String {
        self.state.current_condition().unwrap().to_string()
    }
}

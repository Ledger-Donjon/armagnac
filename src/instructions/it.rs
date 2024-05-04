//! Implements IT instruction.

use crate::{
    arm::{Arm7Processor, RunError},
    condition::Condition,
    decoder::DecodeError,
    instructions::{other, unpredictable},
    it_state::{ItState, ItThenElse},
};

use super::Instruction;

pub struct It {
    /// IT state to be set.
    state: ItState,
}

impl Instruction for It {
    fn patterns() -> &'static [&'static str] {
        &["10111111xxxxxxxx"]
    }

    fn try_decode(tn: usize, ins: u32, state: ItState) -> Result<Self, DecodeError> {
        debug_assert_eq!(tn, 1);
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

    fn execute(&self, proc: &mut Arm7Processor) -> Result<bool, RunError> {
        proc.it_state = self.state;
        Ok(false)
    }

    fn name(&self) -> String {
        let mut name: String = "it".into();
        for x in &self.state.to_then_else() {
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

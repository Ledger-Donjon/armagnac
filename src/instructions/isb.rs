//! Implements ISB instruction.

use crate::{
    arm::{Arm7Processor, RunError},
    decoder::DecodeError,
    it_state::ItState,
};

use super::Instruction;

pub struct Isb {
    /// Option, 4-bits wide.
    option: u8,
}

impl Instruction for Isb {
    fn patterns() -> &'static [&'static str] {
        &["111100111011(1)(1)(1)(1)10(0)0(1)(1)(1)(1)0110xxxx"]
    }

    fn try_decode(tn: usize, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        debug_assert_eq!(tn, 1);
        Ok(Self {
            option: (ins & 0xf) as u8,
        })
    }

    fn execute(&self, _proc: &mut Arm7Processor) -> Result<bool, RunError> {
        Ok(false)
    }

    fn name(&self) -> String {
        "isb".into()
    }

    fn args(&self, _pc: u32) -> String {
        match self.option {
            0xf => "sy".into(),
            _ => self.option.to_string(),
        }
    }
}

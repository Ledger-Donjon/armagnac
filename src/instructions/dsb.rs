//! Implements DSB (Data Synchronization Barrier) instruction.

use super::Instruction;
use super::{
    ArmVersion::{V6M, V7M, V8M},
    Pattern,
};
use crate::{
    arm::{ArmProcessor, RunError},
    decoder::DecodeError,
    it_state::ItState,
};

pub struct Dsb {
    /// Option, 4-bits wide.
    option: u8,
}

impl Instruction for Dsb {
    fn patterns() -> &'static [Pattern] {
        &[Pattern {
            tn: 1,
            versions: &[V6M, V7M, V8M],
            expression: "111100111011(1)(1)(1)(1)10(0)0(1)(1)(1)(1)0100xxxx",
        }]
    }

    fn try_decode(tn: usize, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        debug_assert_eq!(tn, 1);
        Ok(Self {
            option: (ins & 0xf) as u8,
        })
    }

    fn execute(&self, _proc: &mut ArmProcessor) -> Result<bool, RunError> {
        Ok(false)
    }

    fn name(&self) -> String {
        "dsb".into()
    }

    fn args(&self, _pc: u32) -> String {
        match self.option {
            0xf => "sy".into(),
            _ => self.option.to_string(),
        }
    }
}

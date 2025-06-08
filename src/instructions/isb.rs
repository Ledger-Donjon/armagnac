//! Implements ISB (Instruction Synchronization Barrier) instruction.

use super::Encoding::{self, T1};
use super::Instruction;
use super::{
    ArmVersion::{V6M, V7EM, V7M, V8M},
    Pattern,
};
use crate::{
    core::ItState,
    core::{ArmProcessor, Effect, RunError},
    decoder::DecodeError,
};

pub struct Isb {
    /// Option, 4-bits wide.
    option: u8,
}

impl Instruction for Isb {
    fn patterns() -> &'static [Pattern] {
        &[Pattern {
            encoding: T1,
            versions: &[V6M, V7M, V7EM, V8M],
            expression: "111100111011(1)(1)(1)(1)10(0)0(1)(1)(1)(1)0110xxxx",
        }]
    }

    fn try_decode(encoding: Encoding, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        debug_assert_eq!(encoding, T1);
        Ok(Self {
            option: (ins & 0xf) as u8,
        })
    }

    fn execute(&self, _proc: &mut ArmProcessor) -> Result<Effect, RunError> {
        Ok(Effect::None)
    }

    fn name(&self) -> String {
        "isb".into()
    }

    fn args(&self, _pc: u32) -> String {
        match self.option {
            0xf => "sy".into(),
            _ => format!("#0x{:x}", self.option),
        }
    }
}

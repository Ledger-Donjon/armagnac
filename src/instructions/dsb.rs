//! Implements DSB (Data Synchronization Barrier) instruction.

use super::Encoding::{self, T1};
use super::Instruction;
use super::{
    ArmVersion::{V6M, V7EM, V7M, V8M},
    Pattern,
};
use crate::instructions::other;
use crate::{
    core::ItState,
    core::{Effect, Processor, RunError},
    decoder::DecodeError,
};

pub struct Dsb {
    /// Option, 4-bits wide.
    option: u8,
}

impl Instruction for Dsb {
    fn patterns() -> &'static [Pattern] {
        &[Pattern {
            encoding: T1,
            versions: &[V6M, V7M, V7EM, V8M],
            expression: "111100111011(1)(1)(1)(1)10(0)0(1)(1)(1)(1)0100xxxx",
        }]
    }

    fn try_decode(encoding: Encoding, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        debug_assert_eq!(encoding, T1);
        let option = (ins & 0xf) as u8;
        // For Arm v6-M, only 0b1111 is specified for the option value. All other encodings are
        // said as reserved.
        // Some encodings matches others instructions like SSBB since Arm v7-M, so we mark those
        // values as 'others'.
        other((option == 0) || (option == 4))?; // SSBB or PSSBB
        Ok(Self { option })
    }

    fn execute(&self, _proc: &mut Processor) -> Result<Effect, RunError> {
        Ok(Effect::None)
    }

    fn name(&self) -> String {
        "dsb".into()
    }

    fn args(&self, _pc: u32) -> String {
        match self.option {
            0xf => "sy".into(),
            _ => format!("#0x{:x}", self.option),
        }
    }
}

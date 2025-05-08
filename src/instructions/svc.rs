//! Implements SVC (Supervisor Call) instruction.

use super::Encoding::{self, T1};
use super::Instruction;
use super::{
    ArmVersion::{V6M, V7EM, V7M, V8M},
    Pattern,
};
use crate::arm::Effect;
use crate::{
    arm::{ArmProcessor, RunError},
    decoder::DecodeError,
    instructions::DecodeHelper,
    irq::Irq,
    it_state::ItState,
};

/// Supervisor Call instruction.
pub struct Svc {
    /// Immediate constant
    imm8: u8,
}

impl Instruction for Svc {
    fn patterns() -> &'static [Pattern] {
        &[Pattern {
            encoding: T1,
            versions: &[V6M, V7M, V7EM, V8M],
            expression: "11011111xxxxxxxx",
        }]
    }

    fn try_decode(encoding: Encoding, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        debug_assert_eq!(encoding, T1);
        Ok(Self {
            imm8: ins.imm8(0) as u8,
        })
    }

    fn execute(&self, proc: &mut ArmProcessor) -> Result<Effect, RunError> {
        proc.request_interrupt(Irq::SVCall);
        Ok(Effect::None)
    }

    fn name(&self) -> String {
        "svc".into()
    }

    fn args(&self, _pc: u32) -> String {
        format!("#{}", self.imm8)
    }
}

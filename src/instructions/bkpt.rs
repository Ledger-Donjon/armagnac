//! Implements BKPT (Breakpoint) instruction.

use super::Encoding::{self, T1};
use super::{
    ArmVersion::{V6M, V7EM, V7M, V8M},
    Instruction, Pattern,
};
use crate::{
    core::Condition,
    core::ItState,
    core::{ArmProcessor, Effect, RunError},
    decoder::DecodeError,
    instructions::DecodeHelper,
};

/// BKPT instruction.
///
/// Breakpoint.
///
/// When executed, Armagnac will returns [crate::arm::Event::Break] event allowing the user to
/// catch such instruction.
pub struct Bkpt {
    /// 8-bit value stored by the instruction.
    imm8: u8,
}

impl Instruction for Bkpt {
    fn patterns() -> &'static [Pattern] {
        &[Pattern {
            encoding: T1,
            versions: &[V6M, V7M, V7EM, V8M],
            expression: "10111110xxxxxxxx",
        }]
    }

    fn try_decode(encoding: Encoding, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        debug_assert_eq!(encoding, T1);
        Ok(Self {
            imm8: ins.imm8(0) as u8,
        })
    }

    fn execute(&self, _proc: &mut ArmProcessor) -> Result<Effect, RunError> {
        Ok(Effect::Break(self.imm8))
    }

    fn condition(&self) -> Option<Condition> {
        // BKPT is unconditional, it always executes even in IT blocks.
        Some(Condition::Always)
    }

    fn name(&self) -> String {
        "bkpt".into()
    }

    fn args(&self, _pc: u32) -> String {
        format!("#{}", self.imm8)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        core::{ArmProcessor, Config, Effect},
        instructions::{bkpt::Bkpt, Instruction},
    };

    #[test]
    fn test_bkpt() {
        let mut proc = ArmProcessor::new(Config::v7m());
        let result = Bkpt { imm8: 0xa5 }.execute(&mut proc).unwrap();
        assert_eq!(result, Effect::Break(0xa5));
    }
}

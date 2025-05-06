//! Implements BKPT (Breakpoint) instruction.

use super::{
    ArmVersion::{V6M, V7EM, V7M, V8M},
    Instruction, Pattern,
};
use crate::{
    arm::{ArmProcessor, RunError},
    condition::Condition,
    decoder::DecodeError,
    instructions::DecodeHelper,
    it_state::ItState,
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
            tn: 1,
            versions: &[V6M, V7M, V7EM, V8M],
            expression: "10111110xxxxxxxx",
        }]
    }

    fn try_decode(tn: usize, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        debug_assert_eq!(tn, 1);
        Ok(Self {
            imm8: ins.imm8(0) as u8,
        })
    }

    fn execute(&self, proc: &mut ArmProcessor) -> Result<bool, RunError> {
        proc.break_request = Some(self.imm8);
        Ok(false)
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
        arm::{ArmProcessor, ArmVersion::V7M},
        instructions::{bkpt::Bkpt, Instruction},
    };

    #[test]
    fn test_bkpt() {
        let mut proc = ArmProcessor::new(V7M, 0);
        assert_eq!(proc.break_request, None);
        Bkpt { imm8: 0xa5 }.execute(&mut proc).unwrap();
        assert_eq!(proc.break_request, Some(0xa5));
    }
}

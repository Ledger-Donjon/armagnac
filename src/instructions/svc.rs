use crate::{
    arm::{Arm7Processor, RunError},
    decoder::DecodeError,
    instructions::DecodeHelper,
    irq::Irq,
    it_state::ItState,
};

use super::Instruction;

/// Supervisor Call instruction.
pub struct Svc {
    /// Immediate constant
    imm8: u8,
}

impl Instruction for Svc {
    fn patterns() -> &'static [&'static str] {
        &["11011111xxxxxxxx"]
    }

    fn try_decode(tn: usize, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        assert_eq!(tn, 1);
        Ok(Self {
            imm8: ins.imm8(0) as u8,
        })
    }

    fn execute(&self, proc: &mut Arm7Processor) -> Result<bool, RunError> {
        proc.request_interrupt(Irq::SVCall);
        Ok(false)
    }

    fn name(&self) -> String {
        "svc".into()
    }

    fn args(&self, _pc: u32) -> String {
        format!("#{}", self.imm8)
    }
}

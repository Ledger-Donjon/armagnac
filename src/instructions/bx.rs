//! Implements BX (Branch and Exchange) instruction.

use crate::{
    arm::{ArmProcessor, RunError},
    decoder::DecodeError,
    instructions::{unpredictable, DecodeHelper},
    it_state::ItState,
    registers::RegisterIndex,
};

use super::Instruction;

/// BX instruction.
pub struct Bx {
    /// Branch target register.
    rm: RegisterIndex,
}

impl Instruction for Bx {
    fn patterns() -> &'static [&'static str] {
        &["010001110xxxx(0)(0)(0)"]
    }

    fn try_decode(tn: usize, ins: u32, state: ItState) -> Result<Self, DecodeError> {
        debug_assert_eq!(tn, 1);
        unpredictable(state.in_it_block_not_last())?;
        Ok(Self { rm: ins.reg4(3) })
    }

    fn execute(&self, proc: &mut ArmProcessor) -> Result<bool, RunError> {
        let target = proc[self.rm];
        proc.bx_write_pc(target)?;
        Ok(true)
    }

    fn name(&self) -> String {
        "bx".into()
    }

    fn args(&self, _pc: u32) -> String {
        self.rm.to_string()
    }
}

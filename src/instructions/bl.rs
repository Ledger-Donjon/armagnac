//! Implements BL (Branch with Link) instruction.

use super::Encoding::{self, T1};
use super::Instruction;
use super::{
    ArmVersion::{V6M, V7EM, V7M, V8M},
    Pattern,
};
use crate::arm::Effect;
use crate::{
    arith::sign_extend,
    arm::{ArmProcessor, RunError},
    decoder::DecodeError,
    instructions::unpredictable,
    it_state::ItState,
};

/// BL instruction.
#[derive(Clone, Copy)]
pub struct Bl {
    /// Offset.
    pub imm32: i32,
}

impl Instruction for Bl {
    fn patterns() -> &'static [Pattern] {
        &[Pattern {
            encoding: T1,
            versions: &[V6M, V7M, V7EM, V8M],
            expression: "11110xxxxxxxxxxx11x1xxxxxxxxxxxx",
        }]
    }

    fn try_decode(encoding: Encoding, ins: u32, state: ItState) -> Result<Self, DecodeError> {
        debug_assert_eq!(encoding, T1);
        unpredictable(state.in_it_block_not_last())?;
        let s = (ins >> 26) & 1;
        let i1 = 1 ^ ((ins >> 13) & 1) ^ s;
        let i2 = 1 ^ ((ins >> 11) & 1) ^ s;
        let imm25 = (s << 24)
            | (i1 << 23)
            | (i2 << 22)
            | (((ins >> 16) & 0x3ff) << 12)
            | ((ins & 0x7ff) << 1);
        Ok(Self {
            imm32: sign_extend(imm25, 25),
        })
    }

    fn execute(&self, proc: &mut ArmProcessor) -> Result<Effect, RunError> {
        let pc = proc.pc();
        let address = (pc as i32 + self.imm32) as u32;
        proc.set_lr(pc | 1);
        proc.set_pc(address);
        Ok(Effect::Branch)
    }

    fn name(&self) -> String {
        "bl".into()
    }

    fn args(&self, pc: u32) -> String {
        // PC value of a Thumb instruction is it's address + 4
        let label = ((pc as i32).wrapping_add(self.imm32) as u32).wrapping_add(4);
        format!("0x{:x}", label)
    }
}

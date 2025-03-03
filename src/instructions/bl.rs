//! Implements BL (Branch with Link) instruction.

use crate::{
    arith::sign_extend,
    arm::{Arm7Processor, RunError},
    decoder::DecodeError,
    instructions::unpredictable,
    it_state::ItState,
};

use super::Instruction;

/// BL instruction.
pub struct Bl {
    /// Offset.
    imm32: i32,
}

impl Instruction for Bl {
    fn patterns() -> &'static [&'static str] {
        &["11110xxxxxxxxxxx11x1xxxxxxxxxxxx"]
    }

    fn try_decode(tn: usize, ins: u32, state: ItState) -> Result<Self, DecodeError> {
        debug_assert_eq!(tn, 1);
        unpredictable(state.in_it_block_not_last())?;
        let s = ins >> 26 & 1;
        let i1 = 1 ^ (ins >> 13 & 1) ^ s;
        let i2 = 1 ^ (ins >> 11 & 1) ^ s;
        let imm25 = s << 24 | i1 << 23 | i2 << 22 | (ins >> 16 & 0x3ff) << 12 | (ins & 0x7ff) << 1;
        Ok(Self {
            imm32: sign_extend(imm25, 25),
        })
    }

    fn execute(&self, proc: &mut Arm7Processor) -> Result<bool, RunError> {
        let pc = proc.pc();
        let address = (pc as i32 + self.imm32) as u32;
        proc.registers.lr = pc | 1;
        proc.set_pc(address);
        Ok(true)
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

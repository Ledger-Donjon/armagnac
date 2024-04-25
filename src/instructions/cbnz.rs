//! Implements CBNZ and CBZ instructions.

use crate::{
    arm::{Arm7Processor, RunError},
    decoder::DecodeError,
    instructions::{reg, unpredictable},
    it_state::ItState,
    registers::RegisterIndex,
};

use super::Instruction;

pub struct Cbnz {
    /// First operand register.
    rn: RegisterIndex,
    /// Branch offset.
    imm32: u32,
    /// True to branch on non-zero, false to branch on zero.
    non_zero: bool,
}

impl Instruction for Cbnz {
    fn patterns() -> &'static [&'static str] {
        &["1011x0x1xxxxxxxx"]
    }

    fn try_decode(tn: usize, ins: u32, state: ItState) -> Result<Self, DecodeError> {
        debug_assert_eq!(tn, 1);
        unpredictable(state.in_it_block())?;
        Ok(Self {
            rn: reg(ins & 7),
            imm32: (ins >> 9 & 1) << 6 | (ins >> 3 & 0x1f) << 1,
            non_zero: ins >> 11 & 1 != 0,
        })
    }

    fn execute(&self, proc: &mut Arm7Processor) -> Result<bool, RunError> {
        if (proc.registers[self.rn].val() == 0) ^ self.non_zero {
            proc.set_pc(proc.pc() + self.imm32);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn name(&self) -> String {
        if self.non_zero { "cbnz" } else { "cbz" }.into()
    }

    fn args(&self, pc: u32) -> String {
        // PC value of a Thumb instruction is it's address + 4
        let label = pc.wrapping_add(self.imm32).wrapping_add(4);
        format!("0x{:x}", label)
    }
}

//! Implements CBNZ (Compare and Branch on Non-Zero) and CBZ (Compare and Branch on Zero) instructions.

use super::Instruction;
use super::{
    ArmVersion::{V7M, V8M},
    Pattern,
};
use crate::{
    arm::{ArmProcessor, RunError},
    decoder::DecodeError,
    helpers::BitAccess,
    instructions::{unpredictable, DecodeHelper},
    it_state::ItState,
    registers::RegisterIndex,
};

pub struct Cbnz {
    /// First operand register.
    rn: RegisterIndex,
    /// Branch offset.
    imm32: u32,
    /// True to branch on non-zero, false to branch on zero.
    non_zero: bool,
}

impl Instruction for Cbnz {
    fn patterns() -> &'static [Pattern] {
        &[Pattern {
            tn: 1,
            versions: &[V7M, V8M],
            expression: "1011x0x1xxxxxxxx",
        }]
    }

    fn try_decode(tn: usize, ins: u32, state: ItState) -> Result<Self, DecodeError> {
        debug_assert_eq!(tn, 1);
        unpredictable(state.in_it_block())?;
        Ok(Self {
            rn: ins.reg3(0),
            imm32: (((ins >> 9) & 1) << 6) | (((ins >> 3) & 0x1f) << 1),
            non_zero: ins.bit(11),
        })
    }

    fn execute(&self, proc: &mut ArmProcessor) -> Result<bool, RunError> {
        if (proc[self.rn] == 0) ^ self.non_zero {
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
        format!("{}, 0x{:x}", self.rn, label)
    }
}

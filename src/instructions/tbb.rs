//! Implements TBB (Table Branch Byte) instruction.

use super::Encoding::{self, T1};
use super::Instruction;
use super::{
    ArmVersion::{V7EM, V7M, V8M},
    Pattern,
};
use crate::{
    core::ItState,
    core::{Processor, Effect, RunError},
    decoder::DecodeError,
    helpers::BitAccess,
    instructions::{unpredictable, DecodeHelper},
    registers::RegisterIndex,
};

/// TBB and TBH instruction
pub struct Tbb {
    /// Base register.
    rn: RegisterIndex,
    /// Index register.
    rm: RegisterIndex,
    /// True if instruction is TBH
    is_tbh: bool,
}

impl Instruction for Tbb {
    fn patterns() -> &'static [Pattern] {
        &[Pattern {
            encoding: T1,
            versions: &[V7M, V7EM, V8M],
            expression: "111010001101xxxx(1)(1)(1)(1)(0)(0)(0)(0)000xxxxx",
        }]
    }

    fn try_decode(encoding: Encoding, ins: u32, state: ItState) -> Result<Self, DecodeError> {
        debug_assert_eq!(encoding, T1);
        let rn = ins.reg4(16);
        let rm = ins.reg4(0);
        unpredictable(rn.is_sp() || rm.is_sp_or_pc())?;
        let is_tbh = ins.bit(4);
        unpredictable(state.in_it_block_not_last())?;
        Ok(Self { rn, rm, is_tbh })
    }

    fn execute(&self, proc: &mut Processor) -> Result<Effect, RunError> {
        let halfwords = if self.is_tbh {
            let address = proc[self.rn].wrapping_add(proc[self.rm] << 1);
            proc.read_u16_unaligned(address)? as u32
        } else {
            let address = proc[self.rn].wrapping_add(proc[self.rm]);
            proc.read_u8(address)? as u32
        };
        let address = proc.pc() + 2 * halfwords;
        proc.set_pc(address);
        Ok(Effect::Branch)
    }

    fn name(&self) -> String {
        if self.is_tbh { "tbh" } else { "tbb" }.into()
    }

    fn args(&self, _pc: u32) -> String {
        if self.is_tbh {
            format!("[{}, {}, lsl #1]", self.rn, self.rm)
        } else {
            format!("[{}, {}]", self.rn, self.rm)
        }
    }
}

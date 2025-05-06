//! Implements STRD (Store Register Dual) instruction.

use super::{AddOrSub, Instruction};
use super::{
    ArmVersion::{V7EM, V7M, V8M},
    Pattern,
};
use crate::arm::{ArmProcessor, RunError};
use crate::decoder::DecodeError;
use crate::helpers::BitAccess;
use crate::instructions::{indexing_args, unpredictable, DecodeHelper};
use crate::it_state::ItState;
use crate::registers::RegisterIndex;

/// STRD (immediate) instruction.
pub struct StrdImm {
    /// First source register.
    rt: RegisterIndex,
    /// Second source register.
    rt2: RegisterIndex,
    /// Base register.
    rn: RegisterIndex,
    /// Immediate offset.
    imm32: u32,
    /// True to store with indexing.
    index: bool,
    /// True to add offset, false to subtract.
    add: bool,
    /// True to write new offset value back to Rn.
    wback: bool,
}

impl Instruction for StrdImm {
    fn patterns() -> &'static [Pattern] {
        &[Pattern {
            tn: 1,
            versions: &[V7M, V7EM, V8M],
            expression: "1110100xx1x0xxxxxxxxxxxxxxxxxxxx",
        }]
    }

    fn try_decode(tn: usize, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        debug_assert_eq!(tn, 1);
        let rt = ins.reg4(12);
        let rt2 = ins.reg4(8);
        let rn = ins.reg4(16);
        let index = ins.bit(24);
        let add = ins.bit(23);
        let wback = ins.bit(21);
        unpredictable(wback && (rn == rt || rn == rt2))?;
        unpredictable(rn.is_pc() || rt.is_sp_or_pc() || rt2.is_sp_or_pc())?;
        Ok(Self {
            rt,
            rt2,
            rn,
            imm32: ins.imm8(0) << 2,
            index,
            add,
            wback,
        })
    }

    fn execute(&self, proc: &mut ArmProcessor) -> Result<bool, RunError> {
        let rn = proc[self.rn];
        let offset_addr = rn.wrapping_add_or_sub(self.imm32, self.add);
        let address = if self.index { offset_addr } else { rn };
        let rt = proc[self.rt];
        let rt2 = proc[self.rt2];
        proc.write_u32_aligned(address, rt)?;
        proc.write_u32_aligned(address.wrapping_add(4), rt2)?;
        if self.wback {
            proc.set(self.rn, offset_addr);
        }
        Ok(false)
    }

    fn name(&self) -> String {
        "strd".into()
    }

    fn args(&self, _pc: u32) -> String {
        format!(
            "{}, {}, {}",
            self.rt,
            self.rt2,
            indexing_args(self.rn, self.imm32, false, self.index, self.add, self.wback,)
        )
    }
}

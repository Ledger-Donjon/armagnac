//! Implements BLX (Branch with Link and Exchange) instruction.

use super::Instruction;
use super::{
    ArmVersion::{V6M, V7M, V8M},
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

/// BLX (register) instruction.
pub struct Blx {
    /// Branch target address register.
    rm: RegisterIndex,
    /// Non secure bit.
    /// Only for ArmV8-M, false for other architectures.
    ns: bool,
}

impl Instruction for Blx {
    fn patterns() -> &'static [Pattern] {
        &[
            Pattern {
                tn: 1,
                versions: &[V6M, V7M],
                expression: "010001111xxxx(0)(0)(0)",
            },
            Pattern {
                tn: 1,
                versions: &[V8M],
                expression: "010001111xxxxx(0)(0)",
            },
        ]
    }

    fn try_decode(tn: usize, ins: u32, state: ItState) -> Result<Self, DecodeError> {
        debug_assert_eq!(tn, 1);
        let rm = ins.reg4(3);
        unpredictable(rm.is_pc())?;
        unpredictable(state.in_it_block_not_last())?;
        Ok(Self { rm, ns: ins.bit(2) })
    }

    fn execute(&self, proc: &mut ArmProcessor) -> Result<bool, RunError> {
        if self.ns {
            // ArmV8-M only, to be implemented.
            todo!()
        }
        let target = proc[self.rm];
        proc.set_lr((proc.pc() - 2) | 1);
        proc.blx_write_pc(target);
        Ok(true)
    }

    fn name(&self) -> String {
        if self.ns { "blxns" } else { "blx" }.into()
    }

    fn args(&self, _pc: u32) -> String {
        self.rm.to_string()
    }
}

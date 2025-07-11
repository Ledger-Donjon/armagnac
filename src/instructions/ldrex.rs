//! Implements LDREX (Load Register Exclusive) instruction.

use super::{Encoding::T1, Pattern};
use crate::{
    core::{
        ArmVersion::{V7EM, V7M, V8M},
        Effect, ItState, Processor, RunError,
    },
    decoder::DecodeError,
    instructions::{indexing_args, unpredictable, DecodeHelper, Encoding, Instruction},
    registers::RegisterIndex,
};

/// LDREX instruction.
///
/// Load Register Exclusive.
pub struct Ldrex {
    /// Destination register.
    rt: RegisterIndex,
    /// Base register.
    rn: RegisterIndex,
    /// Offset to be added to Rn.
    imm32: u32,
}

impl Instruction for Ldrex {
    fn patterns() -> &'static [Pattern] {
        &[Pattern {
            encoding: T1,
            versions: &[V7M, V7EM, V8M],
            expression: "111010000101xxxxxxxx(1)(1)(1)(1)xxxxxxxx",
        }]
    }

    fn try_decode(encoding: Encoding, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        debug_assert_eq!(encoding, T1);
        let rt = ins.reg4(12);
        let rn = ins.reg4(16);
        unpredictable(rt.is_sp_or_pc() || rn.is_pc())?;
        Ok(Self {
            rt,
            rn,
            imm32: ins.imm8(0) << 2,
        })
    }

    fn execute(&self, proc: &mut Processor) -> Result<Effect, RunError> {
        let address = proc[self.rn] + self.imm32;
        proc.set_exclusive_monitors(address, 4);
        let value = proc.read_u32_aligned(address)?;
        proc.set(self.rt, value);
        Ok(Effect::None)
    }

    fn name(&self) -> String {
        "ldrex".into()
    }

    fn args(&self, _pc: u32) -> String {
        format!(
            "{}, {}",
            self.rt,
            indexing_args(self.rn, self.imm32, false, true, true, false)
        )
    }
}

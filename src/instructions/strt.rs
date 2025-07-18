//! Implements STRT (Store Register Unprivileged) instruction.

use super::{
    Encoding::{self, T1},
    Pattern,
};
use crate::{
    core::{
        ArmVersion::{V7EM, V7M, V8M},
        Effect, ItState, Processor, RunError,
    },
    decoder::DecodeError,
    instructions::{indexing_args, undefined, unpredictable, DecodeHelper, Instruction},
    registers::RegisterIndex,
};

/// STRT instruction.
///
/// Store Register Unprivileged.
pub struct Strt {
    /// Source register.
    rt: RegisterIndex,
    /// Base register.
    rn: RegisterIndex,
    /// Offset to be added to Rn.
    imm32: u32,
}

impl Instruction for Strt {
    fn patterns() -> &'static [Pattern] {
        &[Pattern {
            encoding: T1,
            versions: &[V7M, V7EM, V8M],
            expression: "111110000100xxxxxxxx1110xxxxxxxx",
        }]
    }

    fn try_decode(encoding: Encoding, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        debug_assert_eq!(encoding, T1);
        let rn = ins.reg4(16);
        let rt = ins.reg4(12);
        undefined(rn.is_pc())?;
        unpredictable(rt.is_sp_or_pc())?;
        Ok(Self {
            rt,
            rn,
            imm32: ins.imm8(0),
        })
    }

    fn execute(&self, proc: &mut Processor) -> Result<Effect, RunError> {
        let address = proc[self.rn].wrapping_add(self.imm32);
        let data = proc[self.rt];
        proc.write_u32_unaligned_with_priv(address, data, false)?;
        Ok(Effect::None)
    }

    fn name(&self) -> String {
        "strt".into()
    }

    fn args(&self, _pc: u32) -> String {
        format!(
            "{}, {}",
            self.rt,
            indexing_args(self.rn, self.imm32, false, true, true, false)
        )
    }
}

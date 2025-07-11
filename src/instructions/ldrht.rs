//! Implements LDRHT (Load Register Halfword Unprivileged) instruction.

use super::Encoding::{self, T1};
use super::Instruction;
use super::{
    ArmVersion::{V7EM, V7M, V8M},
    Pattern,
};
use crate::instructions::indexing_args;
use crate::{
    core::ItState,
    core::{Effect, Processor, RunError},
    decoder::DecodeError,
    instructions::{other, unpredictable, DecodeHelper},
    registers::RegisterIndex,
};

/// LDRHT instruction.
///
/// Load Register Halfword Unprivileged.
pub struct Ldrht {
    // Destination register.
    rt: RegisterIndex,
    // Base register.
    rn: RegisterIndex,
    // Immediate offset added to the value of Rn.
    imm32: u32,
}

impl Instruction for Ldrht {
    fn patterns() -> &'static [Pattern] {
        &[Pattern {
            encoding: T1,
            versions: &[V7M, V7EM, V8M],
            expression: "111110000011xxxxxxxx1110xxxxxxxx",
        }]
    }

    fn try_decode(encoding: Encoding, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        debug_assert_eq!(encoding, T1);
        let rn = ins.reg4(16);
        other(rn.is_pc())?;
        let rt = ins.reg4(12);
        unpredictable(rt.is_sp_or_pc())?;
        Ok(Self {
            rt,
            rn,
            imm32: ins.imm8(0),
        })
    }

    fn execute(&self, proc: &mut Processor) -> Result<Effect, RunError> {
        let address = proc[self.rn].wrapping_add(self.imm32);
        let data = proc.read_u16_unaligned_with_priv(address, false)?;
        proc.set(self.rt, data as u32);
        Ok(Effect::None)
    }

    fn name(&self) -> String {
        "ldrht".into()
    }

    fn args(&self, _pc: u32) -> String {
        format!(
            "{}, {}",
            self.rt,
            indexing_args(self.rn, self.imm32, false, true, true, false)
        )
    }
}

//! Implements LDRBT (Load Register Byte Unprivileged) instruction.

use super::Encoding::{self, T1};
use super::Instruction;
use super::{
    ArmVersion::{V7EM, V7M, V8M},
    Pattern,
};
use crate::instructions::indexing_args;
use crate::{
    core::ItState,
    core::{Processor, Effect, RunError},
    decoder::DecodeError,
    instructions::{other, unpredictable, DecodeHelper},
    registers::RegisterIndex,
};

/// LDRBT instruction.
///
/// Load Register Byte Unprivileged.
pub struct Ldrbt {
    /// Destination register.
    rt: RegisterIndex,
    /// Base register.
    rn: RegisterIndex,
    /// Offset to be added to Rn.
    imm32: u32,
}

impl Instruction for Ldrbt {
    fn patterns() -> &'static [Pattern] {
        &[Pattern {
            encoding: T1,
            versions: &[V7M, V7EM, V8M],
            expression: "111110000001xxxxxxxx1110xxxxxxxx",
        }]
    }

    fn try_decode(encoding: Encoding, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        debug_assert_eq!(encoding, T1);
        let rn = ins.reg4(16);
        other(rn.is_pc())?; // LDRB (literal)
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
        let data = proc.read_u8_with_priv(address, false)?;
        proc.set(self.rt, data as u32);
        Ok(Effect::None)
    }

    fn name(&self) -> String {
        "ldrbt".into()
    }

    fn args(&self, _pc: u32) -> String {
        format!(
            "{}, {}",
            self.rt,
            indexing_args(self.rn, self.imm32, false, true, true, false)
        )
    }
}

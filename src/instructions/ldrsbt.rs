//! Implements LDRSBT (Load Register Signed Byte Unprivileged) instruction.

use super::Encoding;
use crate::{
    core::{
        ArmVersion::{V7EM, V7M, V8M},
        Effect, ItState, Processor, RunError,
    },
    decoder::DecodeError,
    instructions::{
        indexing_args, other, unpredictable, DecodeHelper, Encoding::T1, Instruction, Pattern,
    },
    registers::RegisterIndex,
};

/// LDRSBT instruction.
///
/// Load Register Signed Byte Unprivileged.
pub struct Ldrsbt {
    /// Destination register.
    rt: RegisterIndex,
    /// Base register.
    rn: RegisterIndex,
    /// Offset to be added to Rn.
    imm32: u32,
}

impl Instruction for Ldrsbt {
    fn patterns() -> &'static [Pattern] {
        &[Pattern {
            encoding: T1,
            versions: &[V7M, V7EM, V8M],
            expression: "111110010001xxxxxxxx1110xxxxxxxx",
        }]
    }

    fn try_decode(encoding: Encoding, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        debug_assert_eq!(encoding, T1);
        let rt = ins.reg4(12);
        let rn = ins.reg4(16);
        other(rn.is_pc())?; // LDRSB (literal)
        unpredictable(rt.is_sp_or_pc())?;
        Ok(Self {
            rt,
            rn,
            imm32: ins.imm8(0),
        })
    }

    fn execute(&self, proc: &mut Processor) -> Result<Effect, RunError> {
        let address = proc[self.rn].wrapping_add(self.imm32);
        let value = proc.read_u8_with_priv(address, false)? as i8 as i32 as u32;
        proc.set(self.rt, value);
        Ok(Effect::None)
    }

    fn name(&self) -> String {
        "ldrsbt".into()
    }

    fn args(&self, _pc: u32) -> String {
        format!(
            "{}, {}",
            self.rt,
            indexing_args(self.rn, self.imm32, false, true, true, false)
        )
    }
}

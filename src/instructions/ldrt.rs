//! Implements LDRT (Load Register Unprivileged) instruction.

use crate::{
    arm::{ArmProcessor, RunError},
    decoder::DecodeError,
    instructions::{other, unpredictable, DecodeHelper},
    it_state::ItState,
    registers::RegisterIndex,
};

use super::Instruction;

/// LDRT instruction.
///
/// Load Register Unprivileged.
pub struct Ldrt {
    /// Destination register.
    rt: RegisterIndex,
    /// Base register.
    rn: RegisterIndex,
    /// Offset to be added to Rn.
    imm32: u32,
}

impl Instruction for Ldrt {
    fn patterns() -> &'static [&'static str] {
        &["111110000101xxxxxxxx1110xxxxxxxx"]
    }

    fn try_decode(tn: usize, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        debug_assert_eq!(tn, 1);
        let rn = ins.reg4(16);
        other(rn.is_pc())?; // LDR (literal)
        let rt = ins.reg4(12);
        unpredictable(rt.is_sp_or_pc())?;
        Ok(Self {
            rt,
            rn,
            imm32: ins.imm8(0),
        })
    }

    fn execute(&self, proc: &mut ArmProcessor) -> Result<bool, RunError> {
        let address = proc[self.rn] + self.imm32;
        let data = proc.read_u32_unaligned_with_priv(address, false)?;
        proc.set(self.rt, data);
        Ok(false)
    }

    fn name(&self) -> String {
        "ldrt".into()
    }

    fn args(&self, _pc: u32) -> String {
        let offset = if self.imm32 != 0 {
            format!(", #{}", self.imm32)
        } else {
            "".into()
        };
        format!("{}, [{}{}]", self.rt, self.rn, offset)
    }
}

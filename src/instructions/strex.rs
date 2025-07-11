//! Implements STREX (Store Register Exclusive) instruction.

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

/// STREX instruction.
///
/// Store Register Exclusive
pub struct Strex {
    /// Destination register for the returned status value.
    rd: RegisterIndex,
    /// Source register.
    rt: RegisterIndex,
    /// Base register.
    rn: RegisterIndex,
    /// Offset to be added to Rn.
    imm32: u32,
}

impl Instruction for Strex {
    fn patterns() -> &'static [Pattern] {
        &[Pattern {
            encoding: T1,
            versions: &[V7M, V7EM, V8M],
            expression: "111010000100xxxxxxxxxxxxxxxxxxxx",
        }]
    }

    fn try_decode(encoding: Encoding, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        debug_assert_eq!(encoding, T1);
        let rd = ins.reg4(8);
        let rt = ins.reg4(12);
        let rn = ins.reg4(16);
        unpredictable(rd.is_sp_or_pc() || rt.is_sp_or_pc() || rn.is_pc())?;
        unpredictable(rd == rn || rd == rt)?;
        Ok(Self {
            rd,
            rt,
            rn,
            imm32: ins.imm8(0) << 2,
        })
    }

    fn execute(&self, proc: &mut Processor) -> Result<Effect, RunError> {
        let address = proc[self.rn].wrapping_add(self.imm32);
        if proc.exclusive_monitors_pass(address, 4)? {
            let value = proc[self.rt];
            proc.write_u32_aligned(address, value)?;
            proc.set(self.rd, 0);
        } else {
            proc.set(self.rd, 1);
        }
        Ok(Effect::None)
    }

    fn name(&self) -> String {
        "strex".into()
    }

    fn args(&self, _pc: u32) -> String {
        format!(
            "{}, {}, {}",
            self.rd,
            self.rt,
            indexing_args(self.rn, self.imm32, false, true, true, false)
        )
    }
}

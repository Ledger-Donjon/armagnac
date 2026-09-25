//! Implements UQADD8 (Unsigned Saturating Add 8) instruction.
//!
//! UQADD8 performs four unsigned 8-bit integer additions, saturating the results
//! to the range 0-255.

use super::Encoding::{self, T1};
use super::Instruction;
use super::{
    ArmVersion::{V7EM, V8M},
    Pattern,
};
use crate::{
    core::ItState,
    core::{Effect, Processor, RunError},
    decoder::DecodeError,
    instructions::{unpredictable, DecodeHelper},
    registers::RegisterIndex,
};

/// UQADD8 instruction.
pub struct Uqadd8 {
    /// Destination register.
    rd: RegisterIndex,
    /// First operand register.
    rn: RegisterIndex,
    /// Second operand register.
    rm: RegisterIndex,
}

impl Instruction for Uqadd8 {
    fn patterns() -> &'static [Pattern] {
        &[Pattern {
            encoding: T1,
            versions: &[V7EM, V8M],
            // UQADD8: 111110101000xxxx1111xxxx0101xxxx
            expression: "111110101000xxxx1111xxxx0101xxxx",
        }]
    }

    fn try_decode(encoding: Encoding, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        debug_assert_eq!(encoding, T1);
        let rd = ins.reg4(8);
        let rn = ins.reg4(16);
        let rm = ins.reg4(0);
        unpredictable(rd.is_sp_or_pc() || rn.is_sp_or_pc() || rm.is_sp_or_pc())?;
        Ok(Self { rd, rn, rm })
    }

    fn execute(&self, proc: &mut Processor) -> Result<Effect, RunError> {
        let rn = proc[self.rn];
        let rm = proc[self.rm];
        // Unsigned saturating add for each byte lane
        let b0 = ((rn & 0xFF) + (rm & 0xFF)).min(255);
        let b1 = (((rn >> 8) & 0xFF) + ((rm >> 8) & 0xFF)).min(255);
        let b2 = (((rn >> 16) & 0xFF) + ((rm >> 16) & 0xFF)).min(255);
        let b3 = (((rn >> 24) & 0xFF) + ((rm >> 24) & 0xFF)).min(255);
        let result = (b3 << 24) | (b2 << 16) | (b1 << 8) | b0;
        proc.set(self.rd, result);
        Ok(Effect::None)
    }

    fn name(&self) -> String {
        "uqadd8".into()
    }

    fn args(&self, _pc: u32) -> String {
        format!("{}, {}, {}", self.rd, self.rn, self.rm)
    }
}

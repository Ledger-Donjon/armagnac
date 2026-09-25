//! Implements SXTAH (Signed Extend and Add Halfword) instruction.
//!
//! SXTAH extracts a 16-bit value from a register, sign-extends it to 32 bits,
//! adds it to a value in another register, and writes the result to the destination register.
//! An optional rotation can be applied to the source register before extracting the halfword.

use super::Encoding::{self, T1};
use super::{unpredictable, DecodeHelper, Instruction, Qualifier};
use super::{
    ArmVersion::{V7EM, V7M, V8M},
    Pattern,
};
use crate::arith::Shift;
use crate::{
    arith::ror,
    core::ItState,
    core::{Effect, Processor, RunError},
    decoder::DecodeError,
    registers::RegisterIndex,
};

/// SXTAH instruction.
pub struct Sxtah {
    /// Destination register.
    rd: RegisterIndex,
    /// First operand register (value to add to).
    rn: RegisterIndex,
    /// Second operand register (halfword to extract from).
    rm: RegisterIndex,
    /// Rotation applied to Rm before halfword extraction.
    rotation: u8,
}

impl Instruction for Sxtah {
    fn patterns() -> &'static [Pattern] {
        &[
            // T1 encoding: SXTAH<c> <Rd>, <Rn>, <Rm>{, <rotation>}
            // 11111010000xxxxx1111xxxx1(0)xxxxxx
            Pattern {
                encoding: T1,
                versions: &[V7M, V7EM, V8M],
                expression: "11111010000xxxxx1111xxxx1(0)xxxxxx",
            },
        ]
    }

    fn try_decode(encoding: Encoding, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        debug_assert_eq!(encoding, T1);
        let rd = ins.reg4(8);
        let rn = ins.reg4(16);
        let rm = ins.reg4(0);
        unpredictable(rd.is_sp_or_pc() || rn.is_pc() || rm.is_sp_or_pc())?;
        Ok(Self {
            rd,
            rn,
            rm,
            rotation: (ins.imm2(4) << 3) as u8,
        })
    }

    fn execute(&self, proc: &mut Processor) -> Result<Effect, RunError> {
        let rotated = ror(proc[self.rm], self.rotation as u32);
        // Sign-extend the bottom 16 bits
        let halfword = rotated as i16 as i32 as u32;
        let result = proc[self.rn].wrapping_add(halfword);
        proc.set(self.rd, result);
        Ok(Effect::None)
    }

    fn name(&self) -> String {
        "sxtah".into()
    }

    fn qualifier(&self) -> Qualifier {
        Qualifier::Wide
    }

    fn args(&self, _pc: u32) -> String {
        format!(
            "{}, {}, {}{}",
            self.rd,
            self.rn,
            self.rm,
            Shift::ror(self.rotation as u32).arg_string()
        )
    }
}

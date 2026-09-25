//! Implements SMULxy (Signed Multiply Halfwords) instructions.
//!
//! SMULBB/SMULBT/SMULTB/SMULTT multiply two signed 16-bit halfwords
//! from the source registers, producing a 32-bit result.

use super::Encoding::{self, T1};
use super::{unpredictable, DecodeHelper, Instruction, Qualifier};
use super::{
    ArmVersion::{V7EM, V7M, V8M},
    Pattern,
};
use crate::{
    core::ItState,
    core::{Effect, Processor, RunError},
    decoder::DecodeError,
    registers::RegisterIndex,
};

/// SMULxy instruction (SMULBB, SMULBT, SMULTB, SMULTT).
pub struct Smulxy {
    /// Destination register.
    rd: RegisterIndex,
    /// First operand register.
    rn: RegisterIndex,
    /// Second operand register.
    rm: RegisterIndex,
    /// Use top halfword of Rn (false = bottom).
    n_high: bool,
    /// Use top halfword of Rm (false = bottom).
    m_high: bool,
}

impl Instruction for Smulxy {
    fn patterns() -> &'static [Pattern] {
        &[
            // T1 encoding: SMUL<x><y> <Rd>, <Rn>, <Rm>
            // 111110110001xxxx1111xxxx00xxxxxx
            Pattern {
                encoding: T1,
                versions: &[V7M, V7EM, V8M],
                expression: "111110110001xxxx1111xxxx00xxxxxx",
            },
        ]
    }

    fn try_decode(encoding: Encoding, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        debug_assert_eq!(encoding, T1);
        let rd = ins.reg4(8);
        let rn = ins.reg4(16);
        let rm = ins.reg4(0);
        let n_high = (ins >> 5) & 1 != 0;
        let m_high = (ins >> 4) & 1 != 0;
        unpredictable(rd.is_sp_or_pc() || rn.is_sp_or_pc() || rm.is_sp_or_pc())?;
        Ok(Self {
            rd,
            rn,
            rm,
            n_high,
            m_high,
        })
    }

    fn execute(&self, proc: &mut Processor) -> Result<Effect, RunError> {
        let operand1 = if self.n_high {
            (proc[self.rn] >> 16) as i16 as i32
        } else {
            proc[self.rn] as i16 as i32
        };
        let operand2 = if self.m_high {
            (proc[self.rm] >> 16) as i16 as i32
        } else {
            proc[self.rm] as i16 as i32
        };
        let result = operand1.wrapping_mul(operand2);
        proc.set(self.rd, result as u32);
        Ok(Effect::None)
    }

    fn name(&self) -> String {
        let n = if self.n_high { "t" } else { "b" };
        let m = if self.m_high { "t" } else { "b" };
        format!("smul{}{}", n, m)
    }

    fn qualifier(&self) -> Qualifier {
        Qualifier::None
    }

    fn args(&self, _pc: u32) -> String {
        format!("{}, {}, {}", self.rd, self.rn, self.rm)
    }
}

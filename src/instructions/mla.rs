//! Implements MLA (Multiply Accumulate) instruction.

use super::Encoding::{self, T1};
use super::Instruction;
use super::{
    ArmVersion::{V7EM, V7M, V8M},
    Pattern,
};
use crate::{
    core::ItState,
    core::{Processor, Effect, RunError},
    decoder::DecodeError,
    instructions::{other, unpredictable, DecodeHelper},
    registers::RegisterIndex,
};

/// MLA instruction.
pub struct Mla {
    /// Destination register.
    rd: RegisterIndex,
    /// First operand register.
    rn: RegisterIndex,
    /// Second operand register.
    rm: RegisterIndex,
    /// Accumulator register
    ra: RegisterIndex,
}

impl Instruction for Mla {
    fn patterns() -> &'static [Pattern] {
        &[Pattern {
            encoding: T1,
            versions: &[V7M, V7EM, V8M],
            expression: "111110110000xxxxxxxxxxxx0000xxxx",
        }]
    }

    fn try_decode(encoding: Encoding, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        debug_assert_eq!(encoding, T1);
        let rd = ins.reg4(8);
        let rn = ins.reg4(16);
        let rm = ins.reg4(0);
        let ra = ins.reg4(12);
        other(ra.is_pc())?; // MUL
        unpredictable(rd.is_sp_or_pc() || rn.is_sp_or_pc() || rm.is_sp_or_pc() || ra.is_sp())?;
        Ok(Self { rd, rn, rm, ra })
    }

    fn execute(&self, proc: &mut Processor) -> Result<Effect, RunError> {
        let op1 = proc[self.rn] as i32;
        let op2 = proc[self.rm] as i32;
        let addend = proc[self.ra] as i32;
        let result = op1.wrapping_mul(op2).wrapping_add(addend);
        proc.set(self.rd, result as u32);
        Ok(Effect::None)
    }

    fn name(&self) -> String {
        "mla".into()
    }

    fn args(&self, _pc: u32) -> String {
        format!("{}, {}, {}, {}", self.rd, self.rn, self.rm, self.ra)
    }
}

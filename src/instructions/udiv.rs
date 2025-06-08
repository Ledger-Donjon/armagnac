//! Implements UDIV (Unsigned Divide) instruction.

use super::Encoding::{self, T1};
use super::Instruction;
use super::{
    ArmVersion::{V7EM, V7M, V8M},
    Pattern,
};
use crate::{
    core::{ArmProcessor, Effect, RunError},
    decoder::DecodeError,
    instructions::{unpredictable, DecodeHelper, ItState},
    registers::RegisterIndex,
};

pub struct Udiv {
    /// Destination register.
    rd: RegisterIndex,
    /// Dividend register.
    rn: RegisterIndex,
    /// Divisor register.
    rm: RegisterIndex,
}

impl Instruction for Udiv {
    fn patterns() -> &'static [Pattern] {
        &[Pattern {
            encoding: T1,
            versions: &[V7M, V7EM, V8M],
            expression: "111110111011xxxx(1)(1)(1)(1)xxxx1111xxxx",
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

    fn execute(&self, proc: &mut ArmProcessor) -> Result<Effect, RunError> {
        let divisor = proc[self.rm];
        if divisor == 0 {
            unimplemented!("Division by zero handling")
        }
        let result = proc[self.rn] / divisor;
        proc.set(self.rd, result);
        Ok(Effect::None)
    }

    fn name(&self) -> String {
        "udiv".into()
    }

    fn args(&self, _pc: u32) -> String {
        format!("{}, {}, {}", self.rd, self.rn, self.rm)
    }
}

//! Implements SDIV (Signed Divide) instruction.

use super::Instruction;
use super::{
    ArmVersion::{V7EM, V7M, V8M},
    Pattern,
};
use crate::{
    arm::{ArmProcessor, RunError},
    decoder::DecodeError,
    instructions::{unpredictable, DecodeHelper},
    it_state::ItState,
    registers::RegisterIndex,
};

pub struct Sdiv {
    /// Destination register.
    rd: RegisterIndex,
    /// Dividend register.
    rn: RegisterIndex,
    /// Divisor register.
    rm: RegisterIndex,
}

impl Instruction for Sdiv {
    fn patterns() -> &'static [Pattern] {
        &[Pattern {
            tn: 1,
            versions: &[V7M, V7EM, V8M],
            expression: "111110111001xxxx(1)(1)(1)(1)xxxx1111xxxx",
        }]
    }

    fn try_decode(tn: usize, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        debug_assert_eq!(tn, 1);
        let rd = ins.reg4(8);
        let rn = ins.reg4(16);
        let rm = ins.reg4(0);
        unpredictable(rd.is_sp_or_pc() || rn.is_sp_or_pc() || rm.is_sp_or_pc())?;
        Ok(Self { rd, rn, rm })
    }

    fn execute(&self, proc: &mut ArmProcessor) -> Result<bool, RunError> {
        let rm = proc[self.rm];
        let result = if rm == 0 {
            todo!()
        } else {
            (proc[self.rn] as i32 / rm as i32) as u32
        };
        proc.set(self.rd, result);
        Ok(false)
    }

    fn name(&self) -> String {
        "sdiv".into()
    }

    fn args(&self, _pc: u32) -> String {
        format!("{}, {}, {}", self.rd, self.rn, self.rm)
    }
}

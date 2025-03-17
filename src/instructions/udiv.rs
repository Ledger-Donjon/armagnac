//! Implements UDIV (Unsigned Divide) instruction.

use crate::{
    arm::{ArmProcessor, RunError},
    decoder::DecodeError,
    instructions::{rdn_args_string, unpredictable, DecodeHelper, ItState},
    registers::RegisterIndex,
};

use super::Instruction;

pub struct Udiv {
    /// Destination register.
    rd: RegisterIndex,
    /// Dividend register.
    rn: RegisterIndex,
    /// Divisor register.
    rm: RegisterIndex,
}

impl Instruction for Udiv {
    fn patterns() -> &'static [&'static str] {
        &["111110111011xxxx(1)(1)(1)(1)xxxx1111xxxx"]
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
        let divisor = proc[self.rm];
        if divisor == 0 {
            unimplemented!("Division by zero handling")
        }
        let result = proc[self.rn] / divisor;
        proc.set(self.rd, result);
        Ok(false)
    }

    fn name(&self) -> String {
        "udiv".into()
    }

    fn args(&self, _pc: u32) -> String {
        format!("{}, {}", rdn_args_string(self.rd, self.rn), self.rm)
    }
}

//! Implements UDIV instruction.

use crate::{
    arm::{Arm7Processor, RunError},
    decoder::DecodeError,
    instructions::{rdn_args_string, reg, unpredictable, ItState},
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
        let rd = reg(ins >> 8 & 0xf);
        let rn = reg(ins >> 16 & 0xf);
        let rm = reg(ins & 0xf);
        unpredictable(rd.is_sp_or_pc() || rn.is_sp_or_pc() || rm.is_sp_or_pc())?;
        Ok(Self { rd, rn, rm })
    }

    fn execute(&self, proc: &mut Arm7Processor) -> Result<bool, RunError> {
        let divisor = proc.registers[self.rm];
        if divisor == 0 {
            unimplemented!("Division by zero handling")
        }
        let result = proc.registers[self.rn] / divisor;
        proc.registers[self.rd] = result;
        Ok(false)
    }

    fn name(&self) -> String {
        "udiv".into()
    }

    fn args(&self, _pc: u32) -> String {
        format!("{}, {}", rdn_args_string(self.rd, self.rn), self.rm)
    }
}

//! Implements RBIT (Reverse Bits) instruction.

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
    instructions::{unpredictable, DecodeHelper},
    registers::RegisterIndex,
};

pub struct Rbit {
    /// Destination register
    rd: RegisterIndex,
    /// Operand register
    rm: RegisterIndex,
}

impl Instruction for Rbit {
    fn patterns() -> &'static [Pattern] {
        &[Pattern {
            encoding: T1,
            versions: &[V7M, V7EM, V8M],
            expression: "111110101001xxxx1111xxxx1010xxxx",
        }]
    }

    fn try_decode(encoding: Encoding, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        debug_assert_eq!(encoding, T1);
        let rm1 = ins.reg4(16);
        let rm2 = ins.reg4(0);
        let rd = ins.reg4(8);
        unpredictable(rm1 != rm2)?;
        unpredictable(rm1.is_sp_or_pc() || rd.is_sp_or_pc())?;
        Ok(Self { rd, rm: rm1 })
    }

    fn execute(&self, proc: &mut Processor) -> Result<Effect, RunError> {
        let mut a = proc[self.rm];
        let mut b = 0;
        for _ in 0..32 {
            b = (b << 1) | a & 1;
            a >>= 1;
        }
        proc.set(self.rd, b);
        Ok(Effect::None)
    }

    fn name(&self) -> String {
        "rbit".into()
    }

    fn args(&self, _pc: u32) -> String {
        format!("{}, {}", self.rd, self.rm)
    }
}

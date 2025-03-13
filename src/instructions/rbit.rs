//! Implements RBIT (Reverse Bits) instruction.

use crate::{
    arm::{ArmProcessor, RunError},
    decoder::DecodeError,
    instructions::{unpredictable, DecodeHelper},
    it_state::ItState,
    registers::RegisterIndex,
};

use super::Instruction;

pub struct Rbit {
    /// Destination register
    rd: RegisterIndex,
    /// Operand register
    rm: RegisterIndex,
}

impl Instruction for Rbit {
    fn patterns() -> &'static [&'static str] {
        &["111110101001xxxx1111xxxx1010xxxx"]
    }

    fn try_decode(tn: usize, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        debug_assert_eq!(tn, 1);
        let rm1 = ins.reg4(16);
        let rm2 = ins.reg4(0);
        let rd = ins.reg4(8);
        unpredictable(rm1 != rm2)?;
        unpredictable(rm1.is_sp_or_pc() || rd.is_sp_or_pc())?;
        Ok(Self { rd, rm: rm1 })
    }

    fn execute(&self, proc: &mut ArmProcessor) -> Result<bool, RunError> {
        let mut a = proc.registers[self.rm];
        let mut b = 0;
        for _ in 0..32 {
            b = (b << 1) | a & 1;
            a >>= 1;
        }
        proc.registers.set(self.rd, b);
        Ok(false)
    }

    fn name(&self) -> String {
        "rbit".into()
    }

    fn args(&self, _pc: u32) -> String {
        format!("{}, {}", self.rd, self.rm)
    }
}

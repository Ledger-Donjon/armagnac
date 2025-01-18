//! Implements CLZ instruction.

use crate::{
    arm::{Arm7Processor, RunError},
    decoder::DecodeError,
    instructions::{unpredictable, DecodeHelper},
    it_state::ItState,
    registers::RegisterIndex,
};

use super::Instruction;

/// CLZ instruction.
pub struct Clz {
    /// Destination register
    rd: RegisterIndex,
    /// Operand register
    rm: RegisterIndex,
}

impl Instruction for Clz {
    fn patterns() -> &'static [&'static str] {
        &["111110101011xxxx1111xxxx1000xxxx"]
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

    fn execute(&self, proc: &mut Arm7Processor) -> Result<bool, RunError> {
        let mut x = proc.registers[self.rm];
        let mut count = 0;
        while x & 1 << 31 == 0 {
            count += 1;
            x = x << 1 | 1;
        }
        proc.registers.set(self.rd, count);
        Ok(false)
    }

    fn name(&self) -> String {
        "clz".into()
    }

    fn args(&self, _pc: u32) -> String {
        format!("{}, {}", self.rd, self.rm)
    }
}

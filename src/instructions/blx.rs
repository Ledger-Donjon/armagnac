//! Implements BLX instruction.

use crate::{
    arm::{Arm7Processor, RunError},
    decoder::DecodeError,
    instructions::{reg, unpredictable},
    it_state::ItState,
    registers::RegisterIndex,
};

use super::Instruction;

/// BLX (register) instruction.
pub struct Blx {
    /// Branch target address register.
    rm: RegisterIndex,
}

impl Instruction for Blx {
    fn patterns() -> &'static [&'static str] {
        &["010001111xxxx(0)(0)(0)"]
    }

    fn try_decode(tn: usize, ins: u32, state: ItState) -> Result<Self, DecodeError> {
        debug_assert_eq!(tn, 1);
        let rm = reg(ins >> 3 & 0xf);
        unpredictable(rm.is_pc())?;
        unpredictable(state.in_it_block_not_last())?;
        Ok(Self { rm })
    }

    fn execute(&self, proc: &mut Arm7Processor) -> Result<bool, RunError> {
        let target = proc.registers[self.rm].val();
        proc.registers.lr.set_val(proc.pc() - 2 | 1);
        proc.blx_write_pc(target);
        Ok(true)
    }

    fn name(&self) -> String {
        "blx".into()
    }

    fn args(&self, _pc: u32) -> String {
        self.rm.to_string()
    }
}

use crate::{
    arm::{Arm7Processor, RunError},
    decoder::DecodeError,
    helpers::BitAccess,
    instructions::{unpredictable, DecodeHelper},
    it_state::ItState,
    registers::RegisterIndex,
};

use super::Instruction;

/// TBB and TBH instruction
pub struct Tbb {
    /// Base register.
    rn: RegisterIndex,
    /// Index register.
    rm: RegisterIndex,
    /// True if instruction is TBH
    is_tbh: bool,
}

impl Instruction for Tbb {
    fn patterns() -> &'static [&'static str] {
        &["111010001101xxxx(1)(1)(1)(1)(0)(0)(0)(0)000xxxxx"]
    }

    fn try_decode(tn: usize, ins: u32, state: ItState) -> Result<Self, DecodeError> {
        debug_assert_eq!(tn, 1);
        let rn = ins.reg4(16);
        let rm = ins.reg4(0);
        unpredictable(rn.is_sp() || rm.is_sp_or_pc())?;
        let is_tbh = ins.bit(4);
        unpredictable(state.in_it_block_not_last())?;
        Ok(Self { rn, rm, is_tbh })
    }

    fn execute(&self, proc: &mut Arm7Processor) -> Result<bool, RunError> {
        let halfwords = if self.is_tbh {
            let address = proc.registers[self.rn] + (proc.registers[self.rm] << 1);
            proc.u16le_at(address)? as u32
        } else {
            let address = proc.registers[self.rn] + proc.registers[self.rm];
            proc.u8_at(address)? as u32
        };
        let address = proc.pc() + 2 * halfwords;
        proc.set_pc(address);
        Ok(true)
    }

    fn name(&self) -> String {
        if self.is_tbh { "tbh" } else { "tbb" }.into()
    }

    fn args(&self, _pc: u32) -> String {
        if self.is_tbh {
            format!("[{}, {}, lsl #1]", self.rn, self.rm)
        } else {
            format!("[{}, {}]", self.rn, self.rm)
        }
    }
}

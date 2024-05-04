//! Implements STMDB and STMFD instructions.

use crate::{
    arm::{Arm7Processor, RunError},
    decoder::DecodeError,
    instructions::{other, reg, unpredictable, ItState},
    registers::{MainRegisterList, RegisterIndex},
};

use super::Instruction;

/// STMDB instruction.
pub struct Stmdb {
    /// Base register.
    ///
    /// [RegisterIndex::Sp] in case of PUSH instruction.
    pub rn: RegisterIndex,
    /// Wether Rn is written back with a modified value.
    ///
    /// True in case of PUSH instruction.
    pub wback: bool,
    /// Registers to be pushed to the stack.
    pub registers: MainRegisterList,
}

impl Instruction for Stmdb {
    fn patterns() -> &'static [&'static str] {
        &["1110100100x0xxxx(0)x(0)xxxxxxxxxxxxx"]
    }

    fn try_decode(tn: usize, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        debug_assert_eq!(tn, 1);
        let rn = reg(ins >> 16 & 0xf);
        let registers = MainRegisterList::new((ins & 0x5fff) as u16);
        let wback = ins >> 21 & 1 != 0;
        other(wback && rn.is_sp())?; // PUSH
        unpredictable(rn.is_pc() || registers.len() < 2)?;
        unpredictable(wback && registers.contains(&rn))?;
        Ok(Self {
            rn,
            wback,
            registers,
        })
    }

    fn execute(&self, proc: &mut Arm7Processor) -> Result<bool, RunError> {
        // SP and PC cannot be pushed.
        // Instruction decoder should prevent this to happen
        debug_assert!(!self.registers.has_sp() && !self.registers.has_pc());
        let mut addr = proc.registers[self.rn];
        for reg in self.registers.iter().rev() {
            addr = addr.wrapping_sub(4);
            proc.set_u32le_at(addr, proc.registers[reg])?;
        }
        if self.wback {
            proc.registers[self.rn] = addr;
        }
        Ok(false)
    }

    fn name(&self) -> String {
        "stmdb".into()
    }

    fn args(&self, _pc: u32) -> String {
        let wback = if self.wback { "!" } else { "" };
        format!("{}{}{{{}}}", self.rn, wback, self.registers)
    }
}

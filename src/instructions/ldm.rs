//! Implements LDM (Load Multiple), LDMIA (Load Multiple Increment After) and LDMFD (Load Multiple
//! Full Descending) instructions.

use super::{other, unpredictable, DecodeHelper, Instruction};
use crate::{
    arm::{ArmProcessor, RunError},
    decoder::DecodeError,
    helpers::BitAccess,
    it_state::ItState,
    registers::{MainRegisterList, RegisterIndex},
};

/// LDM instruction.
pub struct Ldm {
    /// Base register.
    rn: RegisterIndex,
    /// Loaded registers list.
    registers: MainRegisterList,
    /// Wether Rn is written back with a modified value.
    wback: bool,
}

impl Instruction for Ldm {
    fn patterns() -> &'static [&'static str] {
        &["11001xxxxxxxxxxx", "1110100010x1xxxxxx(0)xxxxxxxxxxxxx"]
    }

    fn try_decode(tn: usize, ins: u32, state: ItState) -> Result<Self, DecodeError> {
        Ok(match tn {
            1 => {
                let rn = ins.reg3(8);
                let registers = MainRegisterList::new((ins & 0xff) as u16);
                unpredictable(registers.is_empty())?;
                Self {
                    rn,
                    registers,
                    wback: !registers.contains(&rn),
                }
            }
            2 => {
                let wback = ins.bit(21);
                let rn = ins.reg4(16);
                other(wback && rn.is_sp())?;
                let registers = MainRegisterList::new((ins & 0xdfff) as u16);
                unpredictable(rn.is_pc() || registers.len() < 2 || (ins & 0xc000 == 0xc000))?;
                unpredictable(registers.has_pc() && state.in_it_block_not_last())?;
                unpredictable(wback && registers.contains(&rn))?;
                Self {
                    rn,
                    registers,
                    wback,
                }
            }
            _ => panic!(),
        })
    }

    fn execute(&self, proc: &mut ArmProcessor) -> Result<bool, RunError> {
        // The ordering of loads into the register must respect the ARM specification,
        // because memory operations may not be commutative if address targets a peripheral.
        let mut address = proc.registers[self.rn];
        let mut jump = false;
        for reg in self.registers.iter() {
            let value = proc.u32le_at(address)?;
            if reg.is_pc() {
                proc.bx_write_pc(value)?;
                jump = true;
            } else {
                proc.registers.set(reg, value);
            }
            address = address.wrapping_add(4);
        }
        if self.wback && !self.registers.contains(&self.rn) {
            proc.registers.set(self.rn, address);
        }
        Ok(jump)
    }

    fn name(&self) -> String {
        "ldm".into()
    }

    fn args(&self, _pc: u32) -> String {
        let ws = if self.wback { "!" } else { "" };
        format!("{}{ws}, {{{}}}", self.rn, self.registers)
    }
}

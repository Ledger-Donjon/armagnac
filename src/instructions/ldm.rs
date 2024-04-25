//! Implements LDM, LDMIA and LDMFD instructions.

use crate::{
    arm::{Arm7Processor, RunError},
    decoder::DecodeError,
    it_state::ItState,
    registers::{MainRegisterList, RegisterIndex},
};

use super::{other, reg, unpredictable, Instruction};

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
                let rn = reg(ins >> 8 & 7);
                let registers = MainRegisterList::new((ins & 0xff) as u16);
                unpredictable(registers.is_empty())?;
                Self {
                    rn,
                    registers,
                    wback: !registers.contains(&rn),
                }
            }
            2 => {
                let wback = ins >> 21 & 1 != 0;
                let rn = reg(ins >> 16 & 0xf);
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

    fn execute(&self, proc: &mut Arm7Processor) -> Result<bool, RunError> {
        // The ordering of loads into the register must respect the ARM specification,
        // because memory operations may not be commutative if address targets a peripheral.
        let mut address = proc.registers[self.rn].val();
        let mut jump = false;
        for reg in self.registers.iter() {
            let value = proc.u32le_at(address)?;
            if reg.is_pc() {
                proc.bx_write_pc(value);
                jump = true;
            } else {
                proc.registers[reg].set_val(value);
            }
            address = address.wrapping_add(4);
        }
        if self.wback {
            proc.registers[self.rn].set_val(address);
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

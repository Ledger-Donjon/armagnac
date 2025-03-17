//! Implements STM (Store Multiple), STMIA (Store Multiple Increment After) and STMEA (Store
//! Multiple Empty Ascending) instructions.

use crate::{
    arm::{ArmProcessor, RunError},
    decoder::DecodeError,
    helpers::BitAccess,
    it_state::ItState,
    registers::{MainRegisterList, RegisterIndex},
};

use super::{unpredictable, DecodeHelper, Instruction};

/// STM instruction.
pub struct Stm {
    /// Base register.
    rn: RegisterIndex,
    /// List of registers to be stored.
    registers: MainRegisterList,
    /// True to write new offset value back to Rn.
    pub wback: bool,
}

impl Instruction for Stm {
    fn patterns() -> &'static [&'static str] {
        &["11000xxxxxxxxxxx", "1110100010x0xxxx(0)x(0)xxxxxxxxxxxxx"]
    }

    fn try_decode(tn: usize, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        Ok(match tn {
            1 => {
                let registers = MainRegisterList::new((ins & 0xff) as u16);
                unpredictable(registers.is_empty())?;
                Self {
                    rn: ins.reg3(8),
                    registers,
                    wback: true,
                }
            }
            2 => {
                let rn = ins.reg4(16);
                let registers = MainRegisterList::new((ins & 0x5fff) as u16);
                let wback = ins.bit(21);
                unpredictable(rn.is_pc() || registers.len() < 2)?;
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
        // The ordering of register stores must respect the ARM specification, because memory
        // operations may not be commutative if address targets a peripheral.
        let mut address = proc[self.rn];
        let lowest = self.registers.lowest();
        for reg in self.registers.iter() {
            // lowest.unwrap is possible here: if we are iterating, there is at least one register
            // in the list.
            if !(self.wback && reg == self.rn && reg != lowest.unwrap()) {
                proc.set_u32le_at(address, proc[reg])?
            }
            address = address.wrapping_add(4);
        }
        if self.wback {
            let mut rn = proc[self.rn];
            rn = rn.wrapping_add(4 * (self.registers.len() as u32));
            proc.set(self.rn, rn);
        }
        Ok(false)
    }

    fn name(&self) -> String {
        "stm".into()
    }

    fn args(&self, _pc: u32) -> String {
        let ws = if self.wback { "!" } else { "" };
        format!("{}{ws}, {{{}}}", self.rn, self.registers)
    }
}

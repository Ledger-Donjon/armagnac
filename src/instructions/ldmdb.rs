use crate::{
    arm::{Arm7Processor, RunError},
    decoder::DecodeError,
    helpers::BitAccess,
    instructions::{unpredictable, DecodeHelper},
    it_state::ItState,
    registers::{MainRegisterList, RegisterIndex},
};

use super::Instruction;

/// LDMDB, LDMEA instructions.
///
/// Load Multiple Decrement Before, Load Multiple Empty Ascending.
pub struct Ldmdb {
    /// Base register.
    rn: RegisterIndex,
    /// Loaded registers list.
    registers: MainRegisterList,
    /// Wether Rn is written back with a modified value.
    wback: bool,
}

impl Instruction for Ldmdb {
    fn patterns() -> &'static [&'static str] {
        &["1110100100x1xxxxxx(0)xxxxxxxxxxxxx"]
    }

    fn try_decode(tn: usize, ins: u32, state: ItState) -> Result<Self, DecodeError> {
        assert_eq!(tn, 1);
        let registers = MainRegisterList::new((ins & 0xdfff) as u16);
        let wback = ins.bit(21);
        let rn = ins.reg4(16);
        unpredictable(rn.is_pc() || registers.len() < 2 || (ins & 0xc000 == 0xc000))?;
        unpredictable(registers.has_pc() && state.in_it_block_not_last())?;
        unpredictable(wback && registers.contains(&rn))?;
        Ok(Self {
            rn,
            registers,
            wback,
        })
    }

    fn execute(&self, proc: &mut Arm7Processor) -> Result<bool, RunError> {
        // The ordering of loads into the register must respect the ARM specification,
        // because memory operations may not be commutative if address targets a peripheral.
        let wback_address = proc.registers[self.rn].wrapping_sub(4 * self.registers.len() as u32);
        let mut address = wback_address;
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
            proc.registers.set(self.rn, wback_address);
        }
        Ok(jump)
    }

    fn name(&self) -> String {
        "ldmdb".into()
    }

    fn args(&self, _pc: u32) -> String {
        let ws = if self.wback { "!" } else { "" };
        format!("{}{ws}, {{{}}}", self.rn, self.registers)
    }
}

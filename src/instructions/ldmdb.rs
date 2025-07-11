//! Implements LDMDB (Load Multiple Decrement Before) and LDMEA (Load Multiple Empty Ascending)
//! instructions.

use super::Encoding::{self, T1};
use super::Instruction;
use super::{
    ArmVersion::{V7EM, V7M, V8M},
    Pattern,
};
use crate::{
    core::ItState,
    core::{Effect, Processor, RunError},
    decoder::DecodeError,
    helpers::BitAccess,
    instructions::{unpredictable, DecodeHelper},
    registers::{MainRegisterList, RegisterIndex},
};

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
    fn patterns() -> &'static [Pattern] {
        &[Pattern {
            encoding: T1,
            versions: &[V7M, V7EM, V8M],
            expression: "1110100100x1xxxxxx(0)xxxxxxxxxxxxx",
        }]
    }

    fn try_decode(encoding: Encoding, ins: u32, state: ItState) -> Result<Self, DecodeError> {
        debug_assert_eq!(encoding, T1);
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

    fn execute(&self, proc: &mut Processor) -> Result<Effect, RunError> {
        // The ordering of loads into the register must respect the ARM specification,
        // because memory operations may not be commutative if address targets a peripheral.
        let wback_address = proc[self.rn].wrapping_sub(4 * self.registers.len() as u32);
        let mut address = wback_address;
        let mut action = Effect::None;
        for reg in self.registers.iter() {
            let value = proc.read_u32_aligned(address)?;
            if reg.is_pc() {
                proc.bx_write_pc(value)?;
                action = Effect::Branch;
            } else {
                proc.set(reg, value);
            }
            address = address.wrapping_add(4);
        }
        if self.wback && !self.registers.contains(&self.rn) {
            proc.set(self.rn, wback_address);
        }
        Ok(action)
    }

    fn name(&self) -> String {
        "ldmdb".into()
    }

    fn args(&self, _pc: u32) -> String {
        let ws = if self.wback { "!" } else { "" };
        format!("{}{ws}, {{{}}}", self.rn, self.registers)
    }
}

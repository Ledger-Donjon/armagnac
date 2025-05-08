//! Implements LDM (Load Multiple), LDMIA (Load Multiple Increment After) and LDMFD (Load Multiple
//! Full Descending) instructions.

use super::Encoding::{self, T1, T2, T3};
use super::{other, unpredictable, DecodeHelper, Instruction, Qualifier};
use super::{
    ArmVersion::{V6M, V7EM, V7M, V8M},
    Pattern,
};
use crate::qualifier_wide_match;
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
    /// Encoding.
    encoding: Encoding,
}

impl Instruction for Ldm {
    fn patterns() -> &'static [Pattern] {
        &[
            Pattern {
                encoding: T1,
                versions: &[V6M, V7M, V7EM, V8M],
                expression: "11001xxxxxxxxxxx",
            },
            Pattern {
                encoding: T2,
                versions: &[V7M, V7EM, V8M],
                expression: "1110100010x1xxxxxx(0)xxxxxxxxxxxxx",
            },
            Pattern {
                encoding: T3,
                versions: &[V8M],
                expression: "1011110xxxxxxxxx",
            },
        ]
    }

    fn try_decode(encoding: Encoding, ins: u32, state: ItState) -> Result<Self, DecodeError> {
        Ok(match encoding {
            T1 => {
                let rn = ins.reg3(8);
                let registers = MainRegisterList::new((ins & 0xff) as u16);
                unpredictable(registers.is_empty())?;
                Self {
                    rn,
                    registers,
                    wback: !registers.contains(&rn),
                    encoding,
                }
            }
            T2 => {
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
                    encoding,
                }
            }
            T3 => {
                let registers = MainRegisterList::new(((ins.imm1(8) << 15) | (ins.imm8(0))) as u16);
                Self {
                    rn: RegisterIndex::Sp,
                    registers,
                    wback: true,
                    encoding,
                }
            }
            _ => panic!(),
        })
    }

    fn execute(&self, proc: &mut ArmProcessor) -> Result<bool, RunError> {
        // The ordering of loads into the register must respect the ARM specification,
        // because memory operations may not be commutative if address targets a peripheral.
        let mut address = proc[self.rn];
        let mut jump = false;
        for reg in self.registers.iter() {
            let value = proc.read_u32_aligned(address)?;
            if reg.is_pc() {
                proc.bx_write_pc(value)?;
                jump = true;
            } else {
                proc.set(reg, value);
            }
            address = address.wrapping_add(4);
        }
        if self.wback && !self.registers.contains(&self.rn) {
            proc.set(self.rn, address);
        }
        Ok(jump)
    }

    fn name(&self) -> String {
        "ldm".into()
    }

    fn qualifier(&self) -> Qualifier {
        qualifier_wide_match!(self.encoding, T2)
    }

    fn args(&self, _pc: u32) -> String {
        let ws = if self.wback { "!" } else { "" };
        format!("{}{ws}, {{{}}}", self.rn, self.registers)
    }
}

//! Implements POP (Pop Multiple Registers) instruction.

use super::Encoding::{self, T1, T2, T3};
use super::{unpredictable, Instruction, Qualifier};
use super::{
    ArmVersion::{V6M, V7EM, V7M, V8M},
    Pattern,
};
use crate::qualifier_wide_match;
use crate::{
    core::{ArmProcessor, Effect, RunError},
    decoder::DecodeError,
    instructions::ItState,
    registers::{MainRegisterList, RegisterIndex},
};

/// POP instruction.
pub struct Pop {
    /// Registers to be poped from the stack.
    registers: MainRegisterList,
    /// Encoding.
    encoding: Encoding,
}

impl Instruction for Pop {
    fn patterns() -> &'static [Pattern] {
        // TODO: better support for ArmV8-M, encoding T4 is missing.
        &[
            Pattern {
                encoding: T1,
                versions: &[V6M, V7M, V7EM],
                expression: "1011110xxxxxxxxx",
            },
            Pattern {
                encoding: T2,
                versions: &[V7M, V7EM, V8M],
                expression: "1110100010111101xx(0)xxxxxxxxxxxxx",
            },
            Pattern {
                encoding: T3,
                versions: &[V7M, V7EM, V8M],
                expression: "1111100001011101xxxx101100000100",
            },
        ]
    }

    fn try_decode(encoding: Encoding, ins: u32, state: ItState) -> Result<Self, DecodeError> {
        Ok(match encoding {
            T1 => {
                let registers =
                    MainRegisterList::new(((((ins >> 8) & 1) << 15) | ins & 0xff) as u16);
                unpredictable(registers.is_empty())?;
                Self {
                    registers,
                    encoding,
                }
            }
            T2 => {
                let registers = MainRegisterList::new((ins & 0xdfff) as u16);
                unpredictable(registers.len() < 2 || (registers.has_pc() && registers.has_lr()))?;
                unpredictable(registers.has_pc() && state.in_it_block_not_last())?;
                Self {
                    registers,
                    encoding,
                }
            }
            T3 => {
                let rt = (ins >> 12) & 0xf;
                let registers = MainRegisterList::new((1 << rt) as u16);
                let rt = RegisterIndex::new_main(rt);
                unpredictable(rt.is_sp() || (rt.is_pc() && state.in_it_block_not_last()))?;
                Self {
                    registers,
                    encoding,
                }
            }
            _ => panic!(),
        })
    }

    fn execute(&self, proc: &mut ArmProcessor) -> Result<Effect, RunError> {
        let mut addr = proc.sp();
        // Note: In the ARM Architecture Reference manual, the reference implementation for POP
        // updates the SP register at the end of the procedure. However, if PC is in the registers
        // list, `LoadWritePC()` would be called before POP effect on SP is applied, and in case of
        // exception return the frame pointer would be wrong.
        // I believe the specification is wrong here, moving the update of SP before seems to fix
        // that.
        *proc.registers.sp_mut() += 4 * self.registers.len() as u32;
        let mut action = Effect::None;
        for reg in self.registers.iter() {
            let val = proc.read_u32_aligned(addr)?;
            if reg.is_pc() {
                action = Effect::Branch;
                proc.load_write_pc(val)?
            } else {
                proc.set(reg, val);
            }
            addr = addr.wrapping_add(4);
        }
        Ok(action)
    }

    fn name(&self) -> String {
        "pop".into()
    }

    fn qualifier(&self) -> Qualifier {
        qualifier_wide_match!(self.encoding, T2 | T3)
    }

    fn args(&self, _pc: u32) -> String {
        format!("{{{}}}", self.registers)
    }
}

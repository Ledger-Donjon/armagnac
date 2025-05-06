//! Implements POP (Pop Multiple Registers) instruction.

use super::{unpredictable, Instruction, Qualifier};
use super::{
    ArmVersion::{V6M, V7EM, V7M, V8M},
    Pattern,
};
use crate::qualifier_wide_match;
use crate::{
    arm::{ArmProcessor, RunError},
    decoder::DecodeError,
    instructions::ItState,
    registers::{MainRegisterList, RegisterIndex},
};

/// POP instruction.
pub struct Pop {
    /// Registers to be poped from the stack.
    registers: MainRegisterList,
    /// Encoding.
    tn: usize,
}

impl Instruction for Pop {
    fn patterns() -> &'static [Pattern] {
        // TODO: better support for ArmV8-M, encoding T4 is missing.
        &[
            Pattern {
                tn: 1,
                versions: &[V6M, V7M, V7EM],
                expression: "1011110xxxxxxxxx",
            },
            Pattern {
                tn: 2,
                versions: &[V7M, V7EM, V8M],
                expression: "1110100010111101xx(0)xxxxxxxxxxxxx",
            },
            Pattern {
                tn: 3,
                versions: &[V7M, V7EM, V8M],
                expression: "1111100001011101xxxx101100000100",
            },
        ]
    }

    fn try_decode(tn: usize, ins: u32, state: ItState) -> Result<Self, DecodeError> {
        Ok(match tn {
            1 => {
                let registers =
                    MainRegisterList::new(((((ins >> 8) & 1) << 15) | ins & 0xff) as u16);
                unpredictable(registers.is_empty())?;
                Self { registers, tn }
            }
            2 => {
                let registers = MainRegisterList::new((ins & 0xdfff) as u16);
                unpredictable(registers.len() < 2 || (registers.has_pc() && registers.has_lr()))?;
                unpredictable(registers.has_pc() && state.in_it_block_not_last())?;
                Self { registers, tn }
            }
            3 => {
                let rt = (ins >> 12) & 0xf;
                let registers = MainRegisterList::new((1 << rt) as u16);
                let rt = RegisterIndex::new_main(rt);
                unpredictable(rt.is_sp() || (rt.is_pc() && state.in_it_block_not_last()))?;
                Self { registers, tn }
            }
            _ => panic!(),
        })
    }

    fn execute(&self, proc: &mut ArmProcessor) -> Result<bool, RunError> {
        let mut addr = proc.sp();
        // Note: In the ARM Architecture Reference manual, the reference implementation for POP
        // updates the SP register at the end of the procedure. However, if PC is in the registers
        // list, `LoadWritePC()` would be called before POP effect on SP is applied, and in case of
        // exception return the frame pointer would be wrong.
        // I believe the specification is wrong here, moving the update of SP before seems to fix
        // that.
        *proc.registers.sp_mut() += 4 * self.registers.len() as u32;
        let mut jump = false;
        for reg in self.registers.iter() {
            let val = proc.read_u32_aligned(addr)?;
            if reg.is_pc() {
                jump = true;
                proc.load_write_pc(val)?
            } else {
                proc.set(reg, val);
            }
            addr = addr.wrapping_add(4);
        }
        Ok(jump)
    }

    fn name(&self) -> String {
        "pop".into()
    }

    fn qualifier(&self) -> Qualifier {
        qualifier_wide_match!(self.tn, 2 | 3)
    }

    fn args(&self, _pc: u32) -> String {
        format!("{{{}}}", self.registers)
    }
}

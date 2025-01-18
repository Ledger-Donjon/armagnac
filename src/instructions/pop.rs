//! Implements POP instruction.

use crate::{
    arm::{Arm7Processor, RunError},
    decoder::DecodeError,
    instructions::ItState,
    registers::MainRegisterList,
};

use super::{reg, unpredictable, Instruction};

/// POP instruction.
pub struct Pop {
    /// Registers to be poped from the stack.
    registers: MainRegisterList,
}

impl Instruction for Pop {
    fn patterns() -> &'static [&'static str] {
        &[
            "1011110xxxxxxxxx",
            "1110100010111101xx(0)xxxxxxxxxxxxx",
            "1111100001011101xxxx101100000100",
        ]
    }

    fn try_decode(tn: usize, ins: u32, state: ItState) -> Result<Self, DecodeError> {
        Ok(match tn {
            1 => {
                let registers = MainRegisterList::new(((ins >> 8 & 1) << 15 | ins & 0xff) as u16);
                unpredictable(registers.len() == 0)?;
                Self { registers }
            }
            2 => {
                let registers = MainRegisterList::new((ins & 0xdfff) as u16);
                unpredictable(registers.len() < 2 || (registers.has_pc() && registers.has_lr()))?;
                unpredictable(registers.has_pc() && state.in_it_block_not_last())?;
                Self { registers }
            }
            3 => {
                let rt = ins >> 12 & 0xf;
                let registers = MainRegisterList::new((1 << rt) as u16);
                let rt = reg(rt);
                unpredictable(rt.is_sp() || (rt.is_pc() && state.in_it_block_not_last()))?;
                Self { registers }
            }
            _ => panic!(),
        })
    }

    fn execute(&self, proc: &mut Arm7Processor) -> Result<bool, RunError> {
        let mut addr = proc.sp();
        let mut has_pc = false;
        for reg in self.registers.iter() {
            let mut val = proc.u32le_at(addr)?;
            if reg.is_pc() {
                has_pc = true;
                if val & 1 == 1 {
                    // Thumb mode jump
                    val -= 1;
                } else {
                    todo!("pop into PC in arm mode");
                }
            }
            proc.registers.set(reg, val);
            addr = addr.wrapping_add(4);
        }
        *proc.registers.sp_mut() = addr;
        Ok(has_pc)
    }

    fn name(&self) -> String {
        "pop".into()
    }

    fn args(&self, _pc: u32) -> String {
        format!("{{{}}}", self.registers)
    }
}

//! Implements PUSH instruction.

use crate::{
    arm::{Arm7Processor, RunError},
    decoder::DecodeError,
    instructions::ItState,
    registers::{MainRegisterList, RegisterIndex},
};

use super::{reg, stmdb::Stmdb, unpredictable, Instruction};

/// PUSH instruction.
pub struct Push {
    /// Registers to be pushed to the stack.
    registers: MainRegisterList,
}

impl Instruction for Push {
    fn patterns() -> &'static [&'static str] {
        &[
            "1011010xxxxxxxxx",
            "1110100100101101(0)x(0)xxxxxxxxxxxxx",
            "1111100001001101xxxx110100000100",
        ]
    }

    fn try_decode(tn: usize, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        Ok(match tn {
            1 => {
                let registers = MainRegisterList::new(((ins >> 8 & 1) << 14 | ins & 0xff) as u16);
                unpredictable(registers.len() == 0)?;
                Self { registers }
            }
            2 => {
                let registers = MainRegisterList::new((ins & 0x5fff) as u16);
                unpredictable(registers.len() < 2)?;
                Self { registers }
            }
            3 => {
                let rt = ins >> 12 & 0xf;
                let registers = MainRegisterList::new((1 << rt) as u16);
                let rt = reg(rt);
                unpredictable(rt.is_sp_or_pc())?;
                Self { registers }
            }
            _ => panic!(),
        })
    }

    fn execute(&self, proc: &mut Arm7Processor) -> Result<bool, RunError> {
        // PUSH is equivalent to STMDB if Rn is SP and wback is true.
        // We use the STMDB implementation.
        let stmdb = Stmdb {
            rn: RegisterIndex::Sp,
            wback: true,
            registers: self.registers,
        };
        stmdb.execute(proc)
    }

    fn name(&self) -> String {
        "push".into()
    }

    fn args(&self, _pc: u32) -> String {
        format!("{{{}}}", self.registers)
    }
}

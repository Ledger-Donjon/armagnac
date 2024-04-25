//! Implements MRS instruction.

use crate::{
    arm::{Arm7Processor, RunError},
    decoder::DecodeError,
    instructions::{reg, unpredictable, ItState},
    registers::RegisterIndex,
};

use super::Instruction;

pub struct Mrs {
    /// Destination register.
    rd: RegisterIndex,
    /// Source special register.
    sysm: RegisterIndex,
}

impl Instruction for Mrs {
    fn patterns() -> &'static [&'static str] {
        &["11110011111(0)(1)(1)(1)(1)10(0)0xxxxxxxxxxxx"]
    }

    fn try_decode(tn: usize, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        debug_assert_eq!(tn, 1);
        let rd = reg(ins >> 8 & 0xf);
        let sysm = ins & 0xff;
        let good_sysm = match sysm {
            0..=3 | 5..=9 | 16..=20 => true,
            _ => false,
        };
        unpredictable(rd.is_sp_or_pc() || !good_sysm)?;
        Ok(Self {
            rd,
            sysm: RegisterIndex::new_sys(sysm),
        })
    }

    fn execute(&self, proc: &mut Arm7Processor) -> Result<bool, RunError> {
        let val = match self.sysm {
            RegisterIndex::Apsr => todo!(),
            RegisterIndex::Iapsr => todo!(),
            RegisterIndex::Eapsr => todo!(),
            RegisterIndex::Xpsr => todo!(),
            RegisterIndex::Epsr => Some(0), // EPSR reads as 0 for MRS
            RegisterIndex::Iepsr => todo!(),
            RegisterIndex::Msp => {
                if proc.is_privileged() {
                    Some(proc.registers.msp.val())
                } else {
                    None
                }
            }
            RegisterIndex::Psp => {
                if proc.is_privileged() {
                    Some(proc.registers.psp.val())
                } else {
                    None
                }
            }
            RegisterIndex::Primask => todo!(),
            _ => panic!(),
        };
        if let Some(val) = val {
            proc.registers[self.rd].set_val(val);
        }
        Ok(false)
    }

    fn name(&self) -> String {
        "mrs".into()
    }

    fn args(&self, _pc: u32) -> String {
        format!("{}, {}", self.rd, self.sysm)
    }
}

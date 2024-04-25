//! Implements MSR (register) instruction.

use crate::{
    arm::{Arm7Processor, RunError},
    decoder::DecodeError,
    instructions::{reg, unpredictable, ItState},
    registers::RegisterIndex,
};

use super::Instruction;

pub struct Msr {
    /// Destination special register.
    sysm: RegisterIndex,
    /// Source register.
    rn: RegisterIndex,
}

impl Instruction for Msr {
    fn patterns() -> &'static [&'static str] {
        &["11110011100(0)xxxx10(0)0(1)(0)(0)(0)xxxxxxxx"]
    }

    fn try_decode(tn: usize, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        debug_assert_eq!(tn, 1);
        let rn = reg(ins >> 16 & 0xf);
        let sysm = ins & 0xff;
        let good_sysm = match sysm {
            0..=3 | 5..=9 | 16..=20 => true,
            _ => false,
        };
        unpredictable(rn.is_sp_or_pc() || !good_sysm)?;
        Ok(Self {
            sysm: RegisterIndex::new_sys(sysm),
            rn,
        })
    }

    fn execute(&self, proc: &mut Arm7Processor) -> Result<bool, RunError> {
        let val = proc.registers[self.rn].val();
        match self.sysm {
            RegisterIndex::Apsr => todo!(),
            RegisterIndex::Iapsr => todo!(),
            RegisterIndex::Eapsr => todo!(),
            RegisterIndex::Xpsr => todo!(),
            RegisterIndex::Epsr => {} // Writes are ignored for EPSR with MSR
            RegisterIndex::Iepsr => todo!(),
            RegisterIndex::Msp => {
                if proc.is_privileged() {
                    proc.registers.msp.set_val(val)
                }
            }
            RegisterIndex::Psp => {
                if proc.is_privileged() {
                    proc.registers.psp.set_val(val)
                }
            }
            RegisterIndex::Primask => todo!(),
            _ => panic!(),
        }
        Ok(false)
    }

    fn name(&self) -> String {
        "msr".into()
    }

    fn args(&self, _pc: u32) -> String {
        format!("{}, {}", self.sysm, self.rn)
    }
}

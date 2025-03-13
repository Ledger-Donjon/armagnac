//! Implements MSR (Move to Special Register) instruction.

use crate::{
    arm::{ArmProcessor, RunError},
    decoder::DecodeError,
    helpers::BitAccess,
    instructions::{unpredictable, DecodeHelper, ItState},
    registers::{Mode, RegisterIndex},
};

use super::Instruction;

/// MSR (register) instruction.
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
        let rn = ins.reg4(16);
        let sysm = ins.imm8(0);
        let good_sysm = matches!(sysm, 0..=3 | 5..=9 | 16..=20);
        unpredictable(rn.is_sp_or_pc() || !good_sysm)?;
        Ok(Self {
            sysm: RegisterIndex::new_sys(sysm),
            rn,
        })
    }

    fn execute(&self, proc: &mut ArmProcessor) -> Result<bool, RunError> {
        let val = proc.registers[self.rn];
        match self.sysm {
            RegisterIndex::Apsr => todo!(),
            RegisterIndex::Iapsr => todo!(),
            RegisterIndex::Eapsr => todo!(),
            RegisterIndex::Xpsr => todo!(),
            RegisterIndex::Epsr => {} // Writes are ignored for EPSR with MSR
            RegisterIndex::Iepsr => todo!(),
            RegisterIndex::Msp => {
                if proc.is_privileged() {
                    proc.registers.msp = val
                }
            }
            RegisterIndex::Psp => {
                if proc.is_privileged() {
                    proc.registers.psp = val
                }
            }
            RegisterIndex::Primask => todo!(),
            RegisterIndex::Basepri => todo!(),
            RegisterIndex::BasepriMask => todo!(),
            RegisterIndex::FaultMask => todo!(),
            RegisterIndex::Control => {
                if proc.is_privileged() {
                    proc.registers.control.set_privileged_bit(val.bit(0));
                    if proc.registers.mode != Mode::Handler {
                        proc.registers.control.set_spsel(val.bit(1))
                    }
                }
            }
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

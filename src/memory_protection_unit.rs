use std::fmt::DebugList;

use num_enum::TryFromPrimitive;

use crate::memory::{
    Env, MemoryAccessError, MemoryReadResult, MemoryWriteResult, RegistersMemoryInterface,
};

/// MPU Control Register.
struct CtrlRegister(u32);

impl CtrlRegister {
    fn write(&mut self, value: u32) -> MemoryWriteResult {
        if value & 0xfffffff8 != 0 {
            return Err(MemoryAccessError::InvalidValue);
        }
        self.0 = value;
        Ok(())
    }
}

impl Default for CtrlRegister {
    fn default() -> Self {
        Self(0)
    }
}

/// MPU Region Number Register.
struct RnrRegister(u32);

impl RnrRegister {
    fn write(&mut self, value: u32) -> MemoryWriteResult {
        if value & 0xffffff00 != 0 {
            return Err(MemoryAccessError::InvalidValue);
        }
        self.0 = value;
        Ok(())
    }

    /// Returns region field value.
    fn region(&self) -> u8 {
        self.0 as u8
    }
}

impl Default for RnrRegister {
    fn default() -> Self {
        Self(0)
    }
}

/// MPU_RBAR register.
#[derive(Clone, Copy)]
struct RbarRegister(u32);

impl RbarRegister {
    fn write(&mut self, value: u32) -> MemoryWriteResult {
        let sh = value >> 3 & 3;
        if sh == 1 {
            // Reserved value
            return Err(MemoryAccessError::InvalidValue);
        }
        self.0 = value;
        Ok(())
    }
}

impl Default for RbarRegister {
    fn default() -> Self {
        Self(0)
    }
}

/// MPU_RLAR register.
#[derive(Clone, Copy)]
struct RlarRegister(u32);

impl Default for RlarRegister {
    fn default() -> Self {
        Self(0)
    }
}

/// MPU_MAIR0 or MPU_MAIR1 register.
struct MairRegister(u32);

impl MairRegister {}

impl Default for MairRegister {
    fn default() -> Self {
        Self(0)
    }
}

#[derive(TryFromPrimitive)]
#[repr(u32)]
pub enum MemoryProtectionUnitRegisterV8M {
    Type = 0x00,
    Ctrl = 0x04,
    Rnr = 0x08,
    Rbar = 0x0c,
    Rlar = 0x10,
    RbarAn = 0x14,
    RlarAn = 0x18,
    Mair0 = 0x30,
    Mair1 = 0x34,
}

/// Memory Protection Unit form Arm-v8-M.
pub struct MemoryProtectionUnitV8M {
    /// MPU_CTRL register.
    ctrl: CtrlRegister,
    /// MPU_RNR register.
    rnr: RnrRegister,
    /// MPU_RBAR registers.
    rbar: Vec<RbarRegister>,
    /// MPU_RBAR registers.
    rlar: Vec<RlarRegister>,
    /// MPU_MAIR0 register.
    mair0: MairRegister,
    /// MPU_MAIR1 register.
    mair1: MairRegister,
}

impl MemoryProtectionUnitV8M {
    pub fn new(region_count: usize) -> Self {
        Self {
            ctrl: Default::default(),
            rnr: Default::default(),
            rbar: vec![Default::default(); region_count],
            rlar: vec![Default::default(); region_count],
            mair0: Default::default(),
            mair1: Default::default(),
        }
    }
}

impl RegistersMemoryInterface for MemoryProtectionUnitV8M {
    type Register = MemoryProtectionUnitRegisterV8M;

    fn read32(&mut self, reg: Self::Register, env: &mut Env) -> MemoryReadResult<u32> {
        Ok(match reg {
            MemoryProtectionUnitRegisterV8M::Type => todo!(),
            MemoryProtectionUnitRegisterV8M::Ctrl => {
                if !env.privileged {
                    return Err(MemoryAccessError::PrivilegedOnly);
                }
                self.ctrl.0
            }
            MemoryProtectionUnitRegisterV8M::Rnr => todo!(),
            MemoryProtectionUnitRegisterV8M::Rbar => todo!(),
            MemoryProtectionUnitRegisterV8M::Rlar => {
                if !env.privileged {
                    return Err(MemoryAccessError::PrivilegedOnly);
                }
                self.rlar
                    .get(self.rnr.region() as usize)
                    .ok_or(MemoryAccessError::InvalidAddress)?
                    .0
            }
            MemoryProtectionUnitRegisterV8M::RbarAn => todo!(),
            MemoryProtectionUnitRegisterV8M::RlarAn => todo!(),
            MemoryProtectionUnitRegisterV8M::Mair0 => todo!(),
            MemoryProtectionUnitRegisterV8M::Mair1 => todo!(),
        })
    }

    fn write32(&mut self, reg: Self::Register, value: u32, env: &mut Env) -> MemoryWriteResult {
        match reg {
            MemoryProtectionUnitRegisterV8M::Type => todo!(),
            MemoryProtectionUnitRegisterV8M::Ctrl => {
                if !env.privileged {
                    return Err(MemoryAccessError::PrivilegedOnly);
                }
                self.ctrl.write(value)?
            }
            MemoryProtectionUnitRegisterV8M::Rnr => {
                if !env.privileged {
                    return Err(MemoryAccessError::PrivilegedOnly);
                }
                self.rnr.write(value)?
            }
            MemoryProtectionUnitRegisterV8M::Rbar => {
                if !env.privileged {
                    return Err(MemoryAccessError::PrivilegedOnly);
                }
                let rbar = self
                    .rbar
                    .get_mut(self.rnr.region() as usize)
                    .ok_or(MemoryAccessError::InvalidAddress)?
                    .write(value)?;
            }
            MemoryProtectionUnitRegisterV8M::Rlar => {
                if !env.privileged {
                    return Err(MemoryAccessError::PrivilegedOnly);
                }
                let rlar = self
                    .rlar
                    .get_mut(self.rnr.region() as usize)
                    .ok_or(MemoryAccessError::InvalidAddress)?
                    .0 = value;
            }
            MemoryProtectionUnitRegisterV8M::RbarAn => todo!(),
            MemoryProtectionUnitRegisterV8M::RlarAn => todo!(),
            MemoryProtectionUnitRegisterV8M::Mair0 => {
                if !env.privileged {
                    return Err(MemoryAccessError::PrivilegedOnly);
                }
                self.mair0.0 = value
            }
            MemoryProtectionUnitRegisterV8M::Mair1 => todo!(),
        }
        Ok(())
    }

    fn size(&self) -> u32 {
        14 * 4
    }
}

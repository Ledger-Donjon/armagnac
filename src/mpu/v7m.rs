use crate::memory::{MemoryAccessError, MemoryWriteResult, RegistersMemoryInterface};
use num_enum::TryFromPrimitive;

use super::Ctrl;

#[derive(TryFromPrimitive)]
#[repr(u32)]
pub enum Register {
    Type = 0x00,
    Ctrl = 0x04,
    Rnr = 0x08,
    Rbar = 0x0c,
    Rasr = 0x10,
    RbarA1 = 0x14,
    RasrA1 = 0x18,
    RbarA2 = 0x1c,
    RasrA2 = 0x20,
    RbarA3 = 0x24,
    RasrA3 = 0x28,
}

/// Memory Protection Unit for ARMv7M.
pub struct MpuV7M {
    /// MPU_CTRL register.
    ctrl: Ctrl,
    /// MPU_RBAR register.
    rbar: Rbar,
    /// MPU_RASR register.
    rasr: Rasr,
}

impl MpuV7M {
    pub fn new() -> Self {
        Self {
            ctrl: Default::default(),
            rbar: Default::default(),
            rasr: Default::default(),
        }
    }
}

impl RegistersMemoryInterface for MpuV7M {
    type Register = Register;

    fn read32(
        &mut self,
        reg: Self::Register,
        _env: &mut crate::memory::Env,
    ) -> crate::memory::MemoryReadResult<u32> {
        Ok(match reg {
            Register::Type => todo!(),
            Register::Ctrl => self.ctrl.0,
            Register::Rnr => todo!(),
            Register::Rbar => self.rbar.0,
            Register::Rasr => self.rasr.0,
            Register::RbarA1 => todo!(),
            Register::RasrA1 => todo!(),
            Register::RbarA2 => todo!(),
            Register::RasrA2 => todo!(),
            Register::RbarA3 => todo!(),
            Register::RasrA3 => todo!(),
        })
    }

    fn write32(
        &mut self,
        reg: Self::Register,
        value: u32,
        _env: &mut crate::memory::Env,
    ) -> crate::memory::MemoryWriteResult {
        match reg {
            Register::Type => todo!(),
            Register::Ctrl => self.ctrl.write(value)?,
            Register::Rnr => todo!(),
            Register::Rbar => self.rbar.0 = value,
            Register::Rasr => self.rasr.write(value)?,
            Register::RbarA1 => todo!(),
            Register::RasrA1 => todo!(),
            Register::RbarA2 => todo!(),
            Register::RasrA2 => todo!(),
            Register::RbarA3 => todo!(),
            Register::RasrA3 => todo!(),
        }
        Ok(())
    }

    fn size(&self) -> u32 {
        4 * 11
    }
}

/// MPU_RBAR register for ARMv7M.
#[derive(Default)]
struct Rbar(u32);

/// MPU_RASR register for ARMv7M.
#[derive(Default)]
struct Rasr(u32);

impl Rasr {
    pub fn write(&mut self, value: u32) -> MemoryWriteResult {
        if value & 0x000000c0 != 0 {
            return Err(MemoryAccessError::InvalidValue);
        }
        self.0 = value;
        Ok(())
    }
}

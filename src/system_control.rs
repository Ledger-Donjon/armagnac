use crate::{
    helpers::{BitAccess, MaskedRegister},
    irq::Irq,
    memory::{
        Env, MemoryAccessError, MemoryOpAction, MemoryReadResult, MemoryWriteResult,
        RegistersMemoryInterface,
    },
};

pub enum SystemControlRegister {
    // Master control register is at 0x000 but is reserved.
    Actlr,
    Stcsr,
    Strvr,
    Stcvr,
    Stcr,
    Cpuid,
    Icsr,
    Vtor,
    Aircr,
    Scr,
    Ccr,
    Shpr(u8),
    Shcsr,
    Cfsr,
    Hfsr,
    Dfsr,
    Mmfar,
    Bfar,
    IdIsar0,
    IdIsar1,
    IdIsar2,
    IdIsar3,
    IdIsar4,
    Cpacr,
    NvicIser(u8),
    NvicIcer(u8),
    NvicIpr(u8),
}

impl TryFrom<u32> for SystemControlRegister {
    type Error = ();

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        if value % 4 != 0 {
            return Err(());
        }
        Ok(match value {
            0x008 => SystemControlRegister::Actlr,
            0x010 => SystemControlRegister::Stcsr,
            0x014 => SystemControlRegister::Strvr,
            0x018 => SystemControlRegister::Stcvr,
            0x01c => SystemControlRegister::Stcr,
            0xd00 => SystemControlRegister::Cpuid,
            0xd04 => SystemControlRegister::Icsr,
            0xd08 => SystemControlRegister::Vtor,
            0xd0c => SystemControlRegister::Aircr,
            0xd10 => SystemControlRegister::Scr,
            0xd14 => SystemControlRegister::Ccr,
            0xd18..=0xd20 => SystemControlRegister::Shpr(((value - 0xd18) / 4) as u8),
            0xd24 => SystemControlRegister::Shcsr,
            0xd28 => SystemControlRegister::Cfsr,
            0xd2c => SystemControlRegister::Hfsr,
            0xd30 => SystemControlRegister::Dfsr,
            0xd34 => SystemControlRegister::Mmfar,
            0xd38 => SystemControlRegister::Bfar,
            0xd60 => SystemControlRegister::IdIsar0,
            0xd64 => SystemControlRegister::IdIsar1,
            0xd68 => SystemControlRegister::IdIsar2,
            0xd6c => SystemControlRegister::IdIsar3,
            0xd70 => SystemControlRegister::IdIsar4,
            0xd88 => SystemControlRegister::Cpacr,
            0x100..=0x13c => SystemControlRegister::NvicIser(((value - 0x100) / 4) as u8),
            0x180..=0x19c => SystemControlRegister::NvicIcer(((value - 0x180) / 4) as u8),
            0x400..=0x7ec => SystemControlRegister::NvicIpr(((value - 0x400) / 4) as u8),
            _ => return Err(()),
        })
    }
}

/// STCSR register.
#[derive(Default)]
struct Stcsr(u32);

impl Stcsr {
    fn read(&mut self) -> u32 {
        let result = self.0;
        self.set_countflag(false);
        result
    }

    fn write(&mut self, value: u32) -> Result<(), MemoryAccessError> {
        if value & 0xfffefff8 != 0 {
            return Err(MemoryAccessError::InvalidValue);
        }
        self.0 = value & 7;
        Ok(())
    }

    /// Sets COUNTFLAG bit value.
    fn set_countflag(&mut self, value: bool) {
        self.0.set_bit(16, value)
    }

    /// Returns TICKINT bit value.
    fn tickint(&self) -> bool {
        self.0.bit(1)
    }

    /// Returns ENABLE bit value.
    fn enable(&self) -> bool {
        self.0.bit(0)
    }
}

pub struct Vtor(u32);

impl Vtor {
    /// Changes the register value. Returns an error if new value is invalid.
    ///
    /// # Arguments
    ///
    /// * `value` - New value. 7 least significant bits must be zero, otherwise
    ///   [`MemoryAccessError::InvalidValue`] error is returned.
    pub fn write(&mut self, value: u32) -> Result<(), MemoryAccessError> {
        if value & 0x7f != 0 {
            // Reserved bits
            return Err(MemoryAccessError::InvalidValue);
        }
        self.0 = value;
        Ok(())
    }

    /// Returns vector table offset.
    pub fn offset(&self) -> u32 {
        self.0
    }
}

impl Default for Vtor {
    fn default() -> Self {
        Self(0)
    }
}

struct Aircr(u32);

impl Aircr {
    fn write(&mut self, value: u32, env: &mut Env) -> Result<(), MemoryAccessError> {
        if value & 0x000078f8 != 0 {
            // Reserved
            return Err(MemoryAccessError::InvalidValue);
        }
        if value >> 16 == 0x5fa {
            self.0 = (value & 0x00000705) | 0xfa050000;
            if value.bit(0) {
                // VECTRESET
                env.actions.push(MemoryOpAction::Reset)
            }
        }
        Ok(())
    }
}

impl Default for Aircr {
    fn default() -> Self {
        Self(0xfa050000)
    }
}

/// Configuration and Control Register.
pub struct Ccr(u32);

impl Ccr {
    /// Set new value.
    pub fn write(&mut self, value: u32) -> MemoryWriteResult {
        let mask = 0x0000031f;
        if value & !mask != 0 {
            return Err(MemoryAccessError::InvalidValue);
        }
        self.0 = value & mask;
        Ok(())
    }

    /// Returns NONBASETHRDENA bit value.
    pub fn nonbasethrdena(&self) -> bool {
        self.0.bit(0)
    }

    /// Returns STKALIGN bit value.
    pub fn stkalign(&self) -> bool {
        self.0.bit(9)
    }

    /// Sets STKALIGN bit value.
    pub fn set_stkalign(&mut self, value: bool) {
        self.0.set_bit(9, value);
    }
}

impl Default for Ccr {
    fn default() -> Self {
        // ARM recommend that STKALIGN is reset to 1 by default, although this is implementation
        // defined.
        Self(1 << 9)
    }
}

/// SHCSR register.
#[derive(Default)]
struct Shcsr(u32);

impl Shcsr {
    fn write(&mut self, value: u32) -> Result<(), MemoryAccessError> {
        let reserved_mask = 0xfff80274;
        self.0 = value & !reserved_mask;
        if value & reserved_mask != 0 {
            Err(MemoryAccessError::InvalidValue)
        } else {
            Ok(())
        }
    }
}

/// CAPCR register.
///
/// Current implementation defines the reset value at 0, which may not be accurate depending on the
/// platform.
#[derive(Default)]
struct Cpacr(u32);

impl Cpacr {
    fn write(&mut self, value: u32) -> Result<(), MemoryAccessError> {
        let mask = 0x00f0ffff;
        if value & !mask != 0 {
            return Err(MemoryAccessError::InvalidValue);
        }
        self.0 = value & mask;
        Ok(())
    }
}

pub struct SystemControl {
    stcsr: Stcsr,
    strvr: MaskedRegister,
    stcvr: u32,
    pub vtor: Vtor,
    aircr: Aircr,
    pub ccr: Ccr,
    shpr: [u32; 3],
    shcsr: Shcsr,
    cpacr: Cpacr,
    nvic_iser: [u32; 16],
    nvic_icer: [u32; 16],
    nvic_ipr: [u32; 124],
}

impl SystemControl {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for SystemControl {
    fn default() -> Self {
        Self {
            stcsr: Default::default(),
            strvr: MaskedRegister::new(0).reserved(0xff000000),
            stcvr: Default::default(),
            vtor: Default::default(),
            aircr: Default::default(),
            ccr: Default::default(),
            shpr: Default::default(),
            shcsr: Default::default(),
            cpacr: Default::default(),
            nvic_iser: Default::default(),
            nvic_icer: Default::default(),
            nvic_ipr: [0; 124],
        }
    }
}

impl RegistersMemoryInterface for SystemControl {
    type Register = SystemControlRegister;

    fn read32(&mut self, reg: SystemControlRegister, _env: &mut Env) -> MemoryReadResult<u32> {
        Ok(match reg {
            SystemControlRegister::Actlr => todo!(),
            SystemControlRegister::Stcsr => self.stcsr.read(),
            SystemControlRegister::Strvr => self.strvr.value,
            SystemControlRegister::Stcvr => self.stcvr,
            SystemControlRegister::Stcr => todo!(),
            SystemControlRegister::Cpuid => todo!(),
            SystemControlRegister::Icsr => todo!(),
            SystemControlRegister::Vtor => todo!(),
            SystemControlRegister::Aircr => self.aircr.0,
            SystemControlRegister::Scr => todo!(),
            SystemControlRegister::Ccr => self.ccr.0,
            SystemControlRegister::Shpr(i) => self.shpr[i as usize].into(),
            SystemControlRegister::Shcsr => self.shcsr.0,
            SystemControlRegister::Cfsr => todo!(),
            SystemControlRegister::Hfsr => todo!(),
            SystemControlRegister::Dfsr => todo!(),
            SystemControlRegister::Mmfar => todo!(),
            SystemControlRegister::Bfar => todo!(),
            SystemControlRegister::IdIsar0 => todo!(),
            SystemControlRegister::IdIsar1 => todo!(),
            SystemControlRegister::IdIsar2 => todo!(),
            SystemControlRegister::IdIsar3 => todo!(),
            SystemControlRegister::IdIsar4 => todo!(),
            SystemControlRegister::Cpacr => self.cpacr.0,
            SystemControlRegister::NvicIser(i) => self.nvic_iser[i as usize].into(),
            SystemControlRegister::NvicIcer(i) => self.nvic_icer[i as usize].into(),
            SystemControlRegister::NvicIpr(i) => self.nvic_ipr[i as usize].into(),
        })
    }

    fn write32(
        &mut self,
        reg: SystemControlRegister,
        value: u32,
        env: &mut Env,
    ) -> MemoryWriteResult {
        match reg {
            SystemControlRegister::Actlr => todo!(),
            SystemControlRegister::Stcsr => self.stcsr.write(value)?,
            SystemControlRegister::Strvr => self.strvr.write(value)?,
            SystemControlRegister::Stcvr => self.stcvr = 0,
            SystemControlRegister::Stcr => todo!(),
            SystemControlRegister::Cpuid => todo!(),
            SystemControlRegister::Icsr => todo!(),
            SystemControlRegister::Vtor => self.vtor.write(value)?,
            SystemControlRegister::Aircr => self.aircr.write(value, env)?,
            SystemControlRegister::Scr => todo!(),
            SystemControlRegister::Ccr => self.ccr.write(value)?,
            SystemControlRegister::Shpr(i) => self.shpr[i as usize] = value,
            SystemControlRegister::Shcsr => self.shcsr.write(value)?,
            SystemControlRegister::Cfsr => todo!(),
            SystemControlRegister::Hfsr => todo!(),
            SystemControlRegister::Dfsr => todo!(),
            SystemControlRegister::Mmfar => todo!(),
            SystemControlRegister::Bfar => todo!(),
            SystemControlRegister::IdIsar0 => todo!(),
            SystemControlRegister::IdIsar1 => todo!(),
            SystemControlRegister::IdIsar2 => todo!(),
            SystemControlRegister::IdIsar3 => todo!(),
            SystemControlRegister::IdIsar4 => todo!(),
            SystemControlRegister::Cpacr => self.cpacr.write(value)?,
            SystemControlRegister::NvicIser(i) => self.nvic_iser[i as usize] = value,
            SystemControlRegister::NvicIcer(i) => self.nvic_icer[i as usize] = value,
            SystemControlRegister::NvicIpr(i) => self.nvic_ipr[i as usize] = value,
        }
        Ok(())
    }

    fn size(&self) -> u32 {
        0xd90 // Up to MPU area
    }

    fn update(&mut self, env: &mut Env) {
        if self.stcsr.enable() {
            if self.stcvr > 0 {
                self.stcvr -= 1;
            } else {
                self.stcvr = self.strvr.value;
                if self.stcsr.tickint() {
                    env.request_interrupt(Irq::SysTick);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::memory::MemoryAccessError;

    use super::Vtor;

    #[test]
    fn test_vtor_register() {
        let mut reg = Vtor::default();
        assert_eq!(reg.write(0xffffff80), Ok(()));
        assert_eq!(reg.0, 0xffffff80);
        assert_eq!(reg.write(0xffffffff), Err(MemoryAccessError::InvalidValue));
        assert_eq!(reg.write(0), Ok(()));
        assert_eq!(reg.0, 0);
    }
}

use crate::{
    helpers::BitAccess,
    memory::{
        Env, MemoryAccessError, MemoryInterface, MemoryOpAction, MemoryReadResult,
        MemoryWriteResult, RegistersMemoryInterface,
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
    Stir,
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
            0xf00 => SystemControlRegister::Stir,
            0x100..=0x13c => SystemControlRegister::NvicIser(((value - 0x100) / 4) as u8),
            0x180..=0x19c => SystemControlRegister::NvicIcer(((value - 0x180) / 4) as u8),
            0x400..=0x7ec => SystemControlRegister::NvicIpr(((value - 0x400) / 4) as u8),
            _ => return Err(()),
        })
    }
}

struct VtorRegister(u32);

impl VtorRegister {
    /// Changes the register value. Returns an error if new value is invalid.
    ///
    /// # Arguments
    ///
    /// * `value` - New value. 7 least significant bits must be zero.
    pub fn set(&mut self, value: u32) -> Result<(), ()> {
        if value & 0x7f != 0 {
            return Err(());
        }
        self.0 = value;
        Ok(())
    }
}

impl Default for VtorRegister {
    fn default() -> Self {
        Self(0)
    }
}

pub struct SystemControl {
    vtor: VtorRegister,
    shpr: [u32; 3],
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
            vtor: Default::default(),
            shpr: Default::default(),
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
            SystemControlRegister::Stcsr => todo!(),
            SystemControlRegister::Strvr => todo!(),
            SystemControlRegister::Stcvr => todo!(),
            SystemControlRegister::Stcr => todo!(),
            SystemControlRegister::Cpuid => todo!(),
            SystemControlRegister::Icsr => todo!(),
            SystemControlRegister::Vtor => todo!(),
            SystemControlRegister::Aircr => todo!(),
            SystemControlRegister::Scr => todo!(),
            SystemControlRegister::Ccr => todo!(),
            SystemControlRegister::Shpr(i) => self.shpr[i as usize].into(),
            SystemControlRegister::Shcsr => todo!(),
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
            SystemControlRegister::Cpacr => todo!(),
            SystemControlRegister::Stir => todo!(),
            SystemControlRegister::NvicIser(i) => self.nvic_iser[i as usize].into(),
            SystemControlRegister::NvicIcer(i) => self.nvic_icer[i as usize].into(),
            SystemControlRegister::NvicIpr(i) => self.nvic_ipr[i as usize].into(),
        })
    }

    fn write32(&mut self, reg: SystemControlRegister, value: u32, _env: &mut Env) -> MemoryWriteResult {
        match reg {
            SystemControlRegister::Actlr => todo!(),
            SystemControlRegister::Stcsr => todo!(),
            SystemControlRegister::Strvr => todo!(),
            SystemControlRegister::Stcvr => todo!(),
            SystemControlRegister::Stcr => todo!(),
            SystemControlRegister::Cpuid => todo!(),
            SystemControlRegister::Icsr => todo!(),
            SystemControlRegister::Vtor => self
                .vtor
                .set(value)
                .map_err(|_| MemoryAccessError::InvalidValue)?,
            SystemControlRegister::Aircr => todo!(),
            SystemControlRegister::Scr => todo!(),
            SystemControlRegister::Ccr => todo!(),
            SystemControlRegister::Shpr(i) => self.shpr[i as usize] = value,
            SystemControlRegister::Shcsr => todo!(),
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
            SystemControlRegister::Cpacr => todo!(),
            SystemControlRegister::Stir => todo!(),
            SystemControlRegister::NvicIser(i) => self.nvic_iser[i as usize] = value,
            SystemControlRegister::NvicIcer(i) => self.nvic_icer[i as usize] = value,
            SystemControlRegister::NvicIpr(i) => self.nvic_ipr[i as usize] = value,
        }
        Ok(())
    }

    fn size(&self) -> u32 {
        0x1000
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

use crate::{
    core::Irq,
    helpers::{BitAccess, MaskedRegister},
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

#[derive(Default)]
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

pub struct Aircr(u32);

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

    /// Returns ENDIANESS bit value.
    /// This bit indicates the system endianess:
    /// - 0: little endian,
    /// - 1: big endian.
    pub fn endianess(&self) -> bool {
        self.0.bit(15)
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

    /// Sets NONBASETHRDENA bit value.
    pub fn set_nonbasethrdena(&mut self, value: bool) {
        self.0.set_bit(0, value);
    }

    /// Returns USERSETMPEND bit value.
    pub fn usersetmpend(&self) -> bool {
        self.0.bit(1)
    }

    /// Sets USERSETMPEND bit value.
    pub fn set_usersetmpend(&mut self, value: bool) {
        self.0.set_bit(1, value)
    }

    /// Returns UNALIGN_TRP bit value.
    pub fn unalign_trp(&self) -> bool {
        self.0.bit(3)
    }

    /// Sets UNALIGN_TRP bit value.
    pub fn set_unalign_trp(&mut self, value: bool) {
        self.0.set_bit(3, value)
    }

    /// Returns DIV_0_TRP bit value.
    pub fn div_0_trp(&self) -> bool {
        self.0.bit(4)
    }

    /// Sets DIV_0_TRP bit value.
    pub fn set_div_0_trp(&mut self, value: bool) {
        self.0.set_bit(4, value)
    }

    /// Returns BFHFNMIGN bit value.
    pub fn bfhfnmign(&self) -> bool {
        self.0.bit(8)
    }

    /// Sets BFHFNMIGN bit value.
    pub fn set_bfhfnmign(&mut self, value: bool) {
        self.0.set_bit(8, value)
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
    pub aircr: Aircr,
    pub ccr: Ccr,
    shpr: [u32; 3],
    shcsr: Shcsr,
    pub cfsr: Cfsr,
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
            cfsr: Default::default(),
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
            SystemControlRegister::Shpr(i) => self.shpr[i as usize],
            SystemControlRegister::Shcsr => self.shcsr.0,
            SystemControlRegister::Cfsr => self.cfsr.0,
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
            SystemControlRegister::NvicIser(i) => self.nvic_iser[i as usize],
            SystemControlRegister::NvicIcer(i) => self.nvic_icer[i as usize],
            SystemControlRegister::NvicIpr(i) => self.nvic_ipr[i as usize],
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
            SystemControlRegister::Cfsr => self.cfsr.write(value)?,
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

/// CFSR (Configurable Fault Status Register).
#[derive(Default)]
pub struct Cfsr(u32);

impl Cfsr {
    /// Sets new register value.
    /// Returns [MemoryAccessError::InvalidValue] when attempting to write a reserved bit.
    pub fn write(&mut self, value: u32) -> MemoryWriteResult {
        if value & !0x030fbfbb != 0 {
            return Err(MemoryAccessError::InvalidValue);
        }
        self.0 = value;
        Ok(())
    }

    /// Returns IACCVIOL bit value.
    pub fn iaccviol(&self) -> bool {
        self.0.bit(0)
    }

    /// Sets IACCVIOL bit value.
    pub fn set_iaccviol(&mut self, value: bool) {
        self.0.set_bit(0, value)
    }

    /// Returns DACCVIOL bit value.
    pub fn daccviol(&self) -> bool {
        self.0.bit(1)
    }

    /// Sets DACCVIOL bit value.
    pub fn set_daccviol(&mut self, value: bool) {
        self.0.set_bit(1, value);
    }

    /// Returns MUNSTKER bit value.
    pub fn munstker(&self) -> bool {
        self.0.bit(3)
    }

    /// Sets MUNSTKER bit value.
    pub fn set_munstker(&mut self, value: bool) {
        self.0.set_bit(3, value)
    }

    /// Returns MSTKERR bit value.
    pub fn mstkerr(&self) -> bool {
        self.0.bit(4)
    }

    /// Sets MSTKERR bit value.
    pub fn set_mstkerr(&mut self, value: bool) {
        self.0.set_bit(4, value)
    }

    /// Returns MLSPERR bit value.
    pub fn mlsperr(&self) -> bool {
        self.0.bit(5)
    }

    /// Sets MLSPERR bit value.
    pub fn set_mlsperr(&mut self, value: bool) {
        self.0.set_bit(5, value)
    }

    /// Returns MMARVALID bit value.
    pub fn mmarvalid(&self) -> bool {
        self.0.bit(7)
    }

    /// Sets MMARVALID bit value.
    pub fn set_mmarvalid(&mut self, value: bool) {
        self.0.set_bit(7, value)
    }

    /// Returns IBUSERR bit value.
    pub fn ibuserr(&self) -> bool {
        self.0.bit(8)
    }

    /// Sets IBUSERR bit value.
    pub fn set_ibuserr(&mut self, value: bool) {
        self.0.set_bit(8, value)
    }

    /// Returns PRECISERR bit value.
    pub fn preciserr(&self) -> bool {
        self.0.bit(9)
    }

    /// Sets PRECISERR bit value.
    pub fn set_preciserr(&mut self, value: bool) {
        self.0.set_bit(9, value)
    }

    /// Returns IMPRECISERR bit value.
    pub fn impreciserr(&self) -> bool {
        self.0.bit(10)
    }

    /// Sets IMPRECISERR bit value.
    pub fn set_impreciserr(&mut self, value: bool) {
        self.0.set_bit(10, value)
    }

    /// Returns UNSTKERR bit value.
    pub fn unstkerr(&self) -> bool {
        self.0.bit(11)
    }

    /// Sets UNSTKERR bit value.
    pub fn set_unstkerr(&mut self, value: bool) {
        self.0.set_bit(11, value)
    }

    /// Returns STKERR bit value.
    pub fn stkerr(&self) -> bool {
        self.0.bit(12)
    }

    /// Sets STKERR bit value.
    pub fn set_stkerr(&mut self, value: bool) {
        self.0.set_bit(12, value)
    }

    /// Returns LSPERR bit value.
    pub fn lsperr(&self) -> bool {
        self.0.bit(13)
    }

    /// Sets LSPERR bit value.
    pub fn set_lsperr(&mut self, value: bool) {
        self.0.set_bit(13, value)
    }

    /// Returns BFARVALID bit value.
    pub fn bfarvalid(&self) -> bool {
        self.0.bit(15)
    }

    /// Sets BFARVALID bit value.
    pub fn set_bfarvalid(&mut self, value: bool) {
        self.0.set_bit(15, value)
    }

    /// Returns UNDEFINSTR bit value.
    pub fn undefinstr(&self) -> bool {
        self.0.bit(16)
    }

    /// Sets UNDEFINSTR bit value.
    pub fn set_undefinstr(&mut self, value: bool) {
        self.0.set_bit(16, value);
    }

    /// Returns INVSTATE bit value.
    pub fn invstate(&self) -> bool {
        self.0.bit(17)
    }

    /// Sets INVSTATE bit value.
    pub fn set_invstate(&mut self, value: bool) {
        self.0.set_bit(17, value);
    }

    /// Returns INVPC bit value.
    pub fn invpc(&self) -> bool {
        self.0.bit(18)
    }

    /// Sets INVPC bit value.
    pub fn set_invpc(&mut self, value: bool) {
        self.0.set_bit(18, value)
    }

    /// Returns NOCP bit value.
    pub fn nocp(&self) -> bool {
        self.0.bit(19)
    }

    /// Sets NOCP bit value.
    pub fn set_nocp(&mut self, value: bool) {
        self.0.set_bit(19, value)
    }

    /// Returns UNALIGNED bit value.
    pub fn unaligned(&self) -> bool {
        self.0.bit(24)
    }

    /// Sets UNALIGNED bit value.
    pub fn set_unaligned(&mut self, value: bool) {
        self.0.set_bit(24, value)
    }

    /// Returns DIVBYZERO bit value.
    pub fn divbyzero(&self) -> bool {
        self.0.bit(25)
    }

    /// Sets DIVBYZERO bit value.
    pub fn set_divbyzero(&mut self, value: bool) {
        self.0.set_bit(25, value)
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

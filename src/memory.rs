/// Possible actions a `MemoryInterface` can request to the processor.
pub enum MemoryOpAction {
    /// Software reset request
    Reset,
    /// Peripheral update request in n cycles
    Update(u32),
}

pub type MemoryReadResult<T> = Result<T, MemoryAccessError>;
pub type MemoryWriteResult = Result<(), MemoryAccessError>;

#[derive(Debug, Clone, Copy)]
pub enum MemoryAccessError {
    InvalidAddress,
    InvalidSize,
    InvalidValue,
    ReadOnly,
}

/// Extra data passed to peripherals when performing read, write or update operations. For
/// instance, it stores the current time, which may be required for peripherals whose state evolves
/// with time.
///
/// This structure also enable issuing actions triggered by read/write operations and to be
/// processed by the processor, such as software reset requests.
pub struct Env {
    /// Current time in CPU cycles
    pub cycles: u64,
    /// Actions
    pub actions: Vec<MemoryOpAction>,
}

impl Env {
    pub fn new(cycles: u64) -> Self {
        Self {
            cycles,
            actions: Vec::new()
        }
    }
}

pub trait MemoryInterface8 {
    fn read_u8(&mut self, _address: u32, env: &mut Env) -> MemoryReadResult<u8> {
        Err(MemoryAccessError::InvalidSize)
    }

    fn write_u8(&mut self, _address: u32, _value: u8, env: &mut Env) -> MemoryWriteResult {
        Err(MemoryAccessError::InvalidSize)
    }
}

pub trait MemoryInterface16 {
    fn read_u16le(&mut self, address: u32, env: &mut Env) -> MemoryReadResult<u16> {
        Err(MemoryAccessError::InvalidSize)
    }

    fn write_u16le(&mut self, _address: u32, _value: u16, env: &mut Env) -> MemoryWriteResult {
        Err(MemoryAccessError::InvalidSize)
    }
}

pub trait MemoryInterface32 {
    fn read_u32le(&mut self, _address: u32, env: &mut Env) -> MemoryReadResult<u32> {
        Err(MemoryAccessError::InvalidSize)
    }

    fn write_u32le(&mut self, _address: u32, _value: u32, env: &mut Env) -> MemoryWriteResult {
        Err(MemoryAccessError::InvalidSize)
    }
}

pub trait MemoryInterface: MemoryInterface32 + MemoryInterface16 + MemoryInterface8 {
    fn size(&self) -> u32;

    fn update(&mut self, env: &mut Env) {}
}

/// Similair to [MemoryInterface] for peripherals that use an enumeration to identify registers.
///
/// A blanket implementation for MemoryInterface exists and manages the conversion from the [u32]
/// address to read/write into the register enumeration value. When the address is invalid this
/// blanket implementation returns an [MemoryAccessError::InvalidAddress] error.
pub trait RegistersMemoryInterface32 {
    type Register: TryFrom<u32>;
    fn read32(&mut self, reg: Self::Register, env: &mut Env) -> MemoryReadResult<u32>;
    fn write32(&mut self, reg: Self::Register, value: u32, env: &mut Env) -> MemoryWriteResult;
}

impl<T: RegistersMemoryInterface32> MemoryInterface32 for T {
    fn read_u32le(&mut self, address: u32, env: &mut Env) -> MemoryReadResult<u32> {
        if let Ok(reg) = address.try_into() {
            self.read32(reg, env)
        } else {
            Err(MemoryAccessError::InvalidAddress)
        }
    }

    fn write_u32le(&mut self, address: u32, value: u32, env: &mut Env) -> MemoryWriteResult {
        if let Ok(reg) = address.try_into() {
            self.write32(reg, value, env)
        } else {
            Err(MemoryAccessError::InvalidAddress)
        }
    }
}

/// Mapped RAM memory
pub struct RamMemory {
    pub data: Vec<u8>,
    write: bool,
}

impl RamMemory {
    pub fn new_zero(size: u32) -> RamMemory {
        let mut v = Vec::new();
        v.resize(size as usize, 0);
        RamMemory {
            data: v,
            write: true,
        }
    }

    pub fn new_from_slice(data: &[u8]) -> RamMemory {
        assert!(data.len() < 0x100000000);
        RamMemory {
            data: Vec::from(data),
            write: true,
        }
    }
}

impl MemoryInterface for RamMemory {
    fn size(&self) -> u32 {
        self.data.len() as u32
    }
}

impl MemoryInterface8 for RamMemory {
    fn read_u8(&mut self, address: u32, env: &mut Env) -> MemoryReadResult<u8> {
        if let Some(val) = self.data.get(address as usize) {
            Ok(*val)
        } else {
            Err(MemoryAccessError::InvalidAddress)
        }
    }

    fn write_u8(&mut self, address: u32, value: u8, env: &mut Env) -> MemoryWriteResult {
        if self.write {
            if let Some(dest) = self.data.get_mut(address as usize) {
                *dest = value;
                Ok(())
            } else {
                Err(MemoryAccessError::InvalidAddress)
            }
        } else {
            Err(MemoryAccessError::ReadOnly)
        }
    }
}

impl MemoryInterface16 for RamMemory {
    fn read_u16le(&mut self, address: u32, env: &mut Env) -> MemoryReadResult<u16> {
        let b0 = self.read_u8(address, env)? as u16;
        let b1 = self.read_u8(address + 1, env)? as u16;
        Ok((b1 << 8) | b0)
    }

    fn write_u16le(&mut self, address: u32, value: u16, env: &mut Env) -> MemoryWriteResult {
        self.write_u8(address + 1, (value >> 8 & 0xff) as u8, env)?;
        self.write_u8(address, (value & 0xff) as u8, env)?;
        Ok(())
    }
}

impl MemoryInterface32 for RamMemory {
    fn read_u32le(&mut self, address: u32, env: &mut Env) -> MemoryReadResult<u32> {
        let b0 = self.read_u8(address, env)? as u32;
        let b1 = self.read_u8(address + 1, env)? as u32;
        let b2 = self.read_u8(address + 2, env)? as u32;
        let b3 = self.read_u8(address + 3, env)? as u32;
        Ok((b3 << 24) | (b2 << 16) | (b1 << 8) | b0)
    }

    fn write_u32le(&mut self, address: u32, value: u32, env: &mut Env) -> MemoryWriteResult {
        self.write_u8(address + 3, (value >> 24) as u8, env)?;
        self.write_u8(address + 2, (value >> 16 & 0xff) as u8, env)?;
        self.write_u8(address + 1, (value >> 8 & 0xff) as u8, env)?;
        self.write_u8(address, (value & 0xff) as u8, env)?;
        Ok(())
    }
}

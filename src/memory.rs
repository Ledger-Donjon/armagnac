//! Memory peripherals and interfacing.
//!
//! For a given target device, it is possible to implement a particular peripheral that may be
//! required for emulation. This is done by implementing the [MemoryInterface] trait. For
//! instance, the following snippet defines the implementation of a Random Number Generator, with
//! one register at offet 0 which can be read get a 32-bit random number:
//!
//! ```
//! use armagnac::memory::{MemoryInterface, Env, MemoryReadResult};
//!
//! struct RngPeripheral {}
//!
//! impl MemoryInterface for RngPeripheral {
//!     fn read_u32le(&mut self, _address: u32, _env: &mut Env) -> MemoryReadResult<u32> {
//!         return Ok(4); // chosen by fair dice roll.
//!     }
//!
//!     fn size(&self) -> u32 {
//!         // There is only one 4 bytes register.
//!         return 4;
//!     }
//! }
//! ```
//!
//! This peripheral can then be mapped during processor creating, at the address 0x4000_0000.
//! A small binary accessing this peripheral is then executed:
//!
//! ```
//! use std::cell::RefCell;
//! use std::rc::Rc;
//! use armagnac::core::{ArmProcessor, Config, RunOptions, Emulator};
//! # use armagnac::memory::{MemoryInterface, Env, MemoryReadResult};
//!
//! fn main() {
//!     let mut proc = ArmProcessor::new(Config::v7m());
//!     // Create RNG peripheral and map it at address 0x40000000
//!     let rng = Rc::new(RefCell::new(RngPeripheral {}));
//!     proc.map_iface(0x40000000, rng).unwrap();
//!     // movs r3, #128 ; 0x80
//!     // lsls r3, r3, #23
//!     // movs r0, #0
//!     // ldr r3, [r3, #0]
//!     proc.map(0x1000, &[0x80, 0x23, 0xdb, 0x05, 0x00, 0x20, 0x1b, 0x68]);
//!     proc.set_pc(0x1000);
//!     // Run and limit execution to 3 instructions
//!     proc.run(RunOptions::new().gas(4)).unwrap();
//!     assert_eq!(proc.registers.r3, 4);
//! }
//! # struct RngPeripheral {}
//! # impl MemoryInterface for RngPeripheral {
//! #     fn read_u32le(&mut self, _address: u32, _env: &mut Env) -> MemoryReadResult<u32> {
//! #         return Ok(4); // chosen by fair dice roll.
//! #     }
//! #     fn size(&self) -> u32 {
//! #         return 4;
//! #     }
//! # }
//! ```

use crate::core::Irq;
use std::iter::repeat_n;

/// Possible actions a `MemoryInterface` can request to the processor.
pub enum MemoryOpAction {
    /// Software reset request
    Reset,
    /// Peripheral update request in n cycles
    Update(u32),
    /// Interrupt request.
    Irq(Irq),
}

pub type MemoryReadResult<T> = Result<T, MemoryAccessError>;
pub type MemoryWriteResult = Result<(), MemoryAccessError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryAccessError {
    InvalidAddress,
    InvalidSize,
    InvalidValue,
    InvalidAlignment,
    ReadOnly,
    Illegal,
    /// Access denied because of unsufficient privileged.
    PrivilegedOnly,
    HardwareError,
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
    /// True if access is privileged.
    pub privileged: bool,
}

impl Env {
    pub fn new(cycles: u64, privileged: bool) -> Self {
        Self {
            cycles,
            actions: Vec::new(),
            privileged,
        }
    }

    pub fn request_interrupt(&mut self, irq: Irq) {
        self.actions.push(MemoryOpAction::Irq(irq))
    }
}

/// This trait must be implemented by any platform peripheral which is connected to the processor
/// memory bus.
pub trait MemoryInterface {
    fn read_u8(&mut self, _address: u32, _env: &mut Env) -> MemoryReadResult<u8> {
        Err(MemoryAccessError::InvalidSize)
    }

    fn write_u8(&mut self, _address: u32, _value: u8, _env: &mut Env) -> MemoryWriteResult {
        Err(MemoryAccessError::InvalidSize)
    }

    fn read_u16le(&mut self, _address: u32, _env: &mut Env) -> MemoryReadResult<u16> {
        Err(MemoryAccessError::InvalidSize)
    }

    fn write_u16le(&mut self, _address: u32, _value: u16, _env: &mut Env) -> MemoryWriteResult {
        Err(MemoryAccessError::InvalidSize)
    }

    fn read_u32le(&mut self, _address: u32, _env: &mut Env) -> MemoryReadResult<u32> {
        Err(MemoryAccessError::InvalidSize)
    }

    fn write_u32le(&mut self, _address: u32, _value: u32, _env: &mut Env) -> MemoryWriteResult {
        Err(MemoryAccessError::InvalidSize)
    }

    fn size(&self) -> u32;

    fn update(&mut self, _env: &mut Env) {}
}

/// Similair to [MemoryInterface] for peripherals that use an enumeration to identify registers.
///
/// A blanket implementation for MemoryInterface exists and manages the conversion from the [u32]
/// address to read/write into the register enumeration value. When the address is invalid this
/// blanket implementation returns an [MemoryAccessError::InvalidAddress] error.
pub trait RegistersMemoryInterface {
    type Register: TryFrom<u32>;
    fn read32(&mut self, reg: Self::Register, env: &mut Env) -> MemoryReadResult<u32>;
    fn write32(&mut self, reg: Self::Register, value: u32, env: &mut Env) -> MemoryWriteResult;
    fn size(&self) -> u32;
    fn update(&mut self, _env: &mut Env) {}
}

impl<T: RegistersMemoryInterface> MemoryInterface for T {
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

    fn read_u8(&mut self, address: u32, env: &mut Env) -> MemoryReadResult<u8> {
        let value = self.read_u32le(address & 0xfffffffc, env)?;
        Ok(match address % 4 {
            0 => (value >> 24) as u8,
            1 => ((value >> 16) & 0xff) as u8,
            2 => ((value >> 8) & 0xff) as u8,
            3 => value as u8,
            _ => panic!(),
        })
    }

    fn write_u8(&mut self, address: u32, value: u8, env: &mut Env) -> MemoryWriteResult {
        let address_aligned = address & 0xfffffffc;
        let read = self.read_u32le(address_aligned, env)?;
        let value = match address % 4 {
            0 => read & 0x00ffffff | ((value as u32) << 24),
            1 => read & 0xff00ffff | ((value as u32) << 16),
            2 => read & 0xffff00ff | ((value as u32) << 8),
            3 => read & 0xffffff00 | value as u32,
            _ => panic!(),
        };
        self.write_u32le(address_aligned, value, env)
    }

    fn size(&self) -> u32 {
        RegistersMemoryInterface::size(self)
    }

    fn update(&mut self, env: &mut Env) {
        RegistersMemoryInterface::update(self, env)
    }
}

/// RAM memory.
pub struct RamMemory {
    /// RAM memory content.
    pub data: Vec<u8>,
    /// If `false` the memory is read-only.
    /// Any attempt from the system to write data will return an [MemoryAccessError::ReadOnly] error.
    pub write: bool,
}

impl RamMemory {
    /// Creates a new RAM memory with `size` capacity, all bytes initialized to zero.
    pub fn new_zero(size: usize) -> RamMemory {
        RamMemory {
            data: vec![0; size],
            write: true,
        }
    }

    /// Creates a new RAM memory with `size` capacity, all bytes initialized to `value`.
    pub fn new_from_value(size: usize, value: u8) -> RamMemory {
        let v = repeat_n(value, size).collect();
        RamMemory {
            data: v,
            write: true,
        }
    }

    /// Creates a new RAM memory with `data` as initial content. The size of the created memory is
    /// the same as `data`.
    pub fn new_from_slice(data: &[u8]) -> RamMemory {
        assert!(data.len() < 0x100000000);
        RamMemory {
            data: Vec::from(data),
            write: true,
        }
    }

    /// Configure the memory as read-only.
    pub fn read_only(self) -> Self {
        Self {
            write: false,
            ..self
        }
    }
}

impl MemoryInterface for RamMemory {
    fn size(&self) -> u32 {
        self.data.len() as u32
    }

    fn read_u8(&mut self, address: u32, _env: &mut Env) -> MemoryReadResult<u8> {
        if let Some(val) = self.data.get(address as usize) {
            Ok(*val)
        } else {
            Err(MemoryAccessError::InvalidAddress)
        }
    }

    fn write_u8(&mut self, address: u32, value: u8, _env: &mut Env) -> MemoryWriteResult {
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

    fn read_u16le(&mut self, address: u32, env: &mut Env) -> MemoryReadResult<u16> {
        let b0 = self.read_u8(address, env)? as u16;
        let b1 = self.read_u8(address + 1, env)? as u16;
        Ok((b1 << 8) | b0)
    }

    fn write_u16le(&mut self, address: u32, value: u16, env: &mut Env) -> MemoryWriteResult {
        self.write_u8(address + 1, ((value >> 8) & 0xff) as u8, env)?;
        self.write_u8(address, (value & 0xff) as u8, env)?;
        Ok(())
    }

    fn read_u32le(&mut self, address: u32, env: &mut Env) -> MemoryReadResult<u32> {
        let b0 = self.read_u8(address, env)? as u32;
        let b1 = self.read_u8(address + 1, env)? as u32;
        let b2 = self.read_u8(address + 2, env)? as u32;
        let b3 = self.read_u8(address + 3, env)? as u32;
        Ok((b3 << 24) | (b2 << 16) | (b1 << 8) | b0)
    }

    fn write_u32le(&mut self, address: u32, value: u32, env: &mut Env) -> MemoryWriteResult {
        self.write_u8(address + 3, (value >> 24) as u8, env)?;
        self.write_u8(address + 2, ((value >> 16) & 0xff) as u8, env)?;
        self.write_u8(address + 1, ((value >> 8) & 0xff) as u8, env)?;
        self.write_u8(address, (value & 0xff) as u8, env)?;
        Ok(())
    }
}

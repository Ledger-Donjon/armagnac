//! Helpers for making registers implementation and manipulation easiers.

use crate::memory::MemoryAccessError;

pub trait BitAccess {
    /// Returns true if bit is set.
    ///
    /// # Arguments
    ///
    /// * `index` - Bit index, 0 being the LSB.
    fn bit(&self, index: usize) -> bool;

    /// Modify a bit.
    ///
    /// # Arguments
    ///
    /// * `index` - Bit index, 0 being the LSB.
    /// * `value` - New bit value.
    fn set_bit(&mut self, index: usize, value: bool);
}

impl BitAccess for u32 {
    fn bit(&self, index: usize) -> bool {
        self >> index & 1 != 0
    }

    fn set_bit(&mut self, index: usize, value: bool) {
        *self = (*self & !(1 << index)) | (value as u32) << index;
    }
}

impl BitAccess for u16 {
    fn bit(&self, index: usize) -> bool {
        self >> index & 1 != 0
    }

    fn set_bit(&mut self, index: usize, value: bool) {
        *self = (*self & !(1 << index)) | (value as u16) << index;
    }
}

impl BitAccess for u8 {
    fn bit(&self, index: usize) -> bool {
        self >> index & 1 != 0
    }

    fn set_bit(&mut self, index: usize, value: bool) {
        *self = (*self & !(1 << index)) | (value as u8) << index;
    }
}

/// 32-bit register with access rules such as reserved bits, write mask, etc.
#[derive(Clone, Copy)]
pub struct MaskedRegister {
    pub value: u32,
    pub reserved_mask: u32,
    pub write_mask: u32,
    pub set_mask: u32,
    pub clear_at_one_mask: u32,
}

impl MaskedRegister {
    pub fn new(value: u32) -> Self {
        Self {
            value,
            reserved_mask: 0,
            write_mask: 0xffffffff,
            set_mask: 0,
            clear_at_one_mask: 0,
        }
    }

    /// Sets the bits which are reserved and must not be written to one.
    ///
    /// Bits defined as 1 in the mask are considered reserved.
    pub fn reserved(self, mask: u32) -> Self {
        Self {
            reserved_mask: mask,
            ..self
        }
    }

    /// Defines which bits can be written, considering others as reserved.
    ///
    /// Bits defined as 1 in the mask are writable.
    pub fn write_mask_reserved(self, mask: u32) -> Self {
        Self {
            reserved_mask: !mask,
            write_mask: mask,
            ..self
        }
    }

    pub fn write_mask(self, mask: u32) -> Self {
        Self {
            write_mask: mask,
            ..self
        }
    }

    pub fn set_mask(self, mask: u32) -> Self {
        Self {
            set_mask: mask,
            ..self
        }
    }

    pub fn clear_at_one(self, mask: u32) -> Self {
        Self {
            clear_at_one_mask: mask,
            ..self
        }
    }

    pub fn write(&mut self, value: u32) -> Result<(), MemoryAccessError> {
        self.value = ((value & self.write_mask) | (value & self.set_mask))
            & !(value & self.clear_at_one_mask);
        if value & self.reserved_mask != 0 {
            return Err(MemoryAccessError::InvalidValue);
        }
        Ok(())
    }
}

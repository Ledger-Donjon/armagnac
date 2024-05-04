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

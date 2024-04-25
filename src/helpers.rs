pub trait TestBit {
    /// Returns true if bit is set.
    ///
    /// # Arguments
    ///
    /// * `index` - Bit index, 0 being the LSB.
    fn bit(&self, index: usize) -> bool;
}

impl TestBit for u32 {
    fn bit(&self, index: usize) -> bool {
        self >> index & 1 != 0
    }
}

impl TestBit for u16 {
    fn bit(&self, index: usize) -> bool {
        self >> index & 1 != 0
    }
}

impl TestBit for u8 {
    fn bit(&self, index: usize) -> bool {
        self >> index & 1 != 0
    }
}

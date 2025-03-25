pub trait Align {
    fn align(&self, n: usize) -> Self;
    fn is_aligned(&self, n: usize) -> bool;
}

impl Align for u32 {
    fn align(&self, n: usize) -> Self {
        self - (self % (n as u32))
    }

    fn is_aligned(&self, n: usize) -> bool {
        *self == self.align(n)
    }
}

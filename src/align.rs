pub trait Align {
    fn align(&self, n: usize) -> Self;
}

impl Align for u32 {
    fn align(&self, n: usize) -> Self {
        self - (self % (n as u32))
    }
}

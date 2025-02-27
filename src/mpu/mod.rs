use crate::memory::{MemoryAccessError, MemoryWriteResult};

pub mod v7m;
pub mod v8m;

/// MPU Control Register.
///
/// For ARMv7M and ARMv8M.
#[derive(Default)]
struct Ctrl(u32);

impl Ctrl {
    fn write(&mut self, value: u32) -> MemoryWriteResult {
        if value & 0xfffffff8 != 0 {
            return Err(MemoryAccessError::InvalidValue);
        }
        self.0 = value;
        Ok(())
    }
}

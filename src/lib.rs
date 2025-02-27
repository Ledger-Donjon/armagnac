//! Armagnac is an experimental ARMv7-M processor emulator designed for embedded systems emulation.
//! It is slow, incomplete and unoptimized. However, it is fully written in Rust!

mod align;
mod arith;
pub mod arm;
mod condition;
mod decoder;
pub mod helpers;
mod instructions;
pub mod irq;
mod it_state;
pub mod memory;
pub mod mpu;
pub mod registers;
mod system_control;

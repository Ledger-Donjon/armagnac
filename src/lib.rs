//! Armagnac is an experimental ARMv7-M processor emulator designed for embedded systems emulation.
//! It is slow, incomplete and unoptimized. However, it is fully written in Rust!
//!
//! Check [arm::ArmProcessor] to know how to quickly emulate an ARM binary.
#![feature(concat_idents)]

mod align;
mod arith;
pub mod arm;
mod condition;
pub mod decoder;
pub mod harness;
pub mod helpers;
pub mod instructions;
pub mod irq;
mod it_state;
pub mod memory;
pub mod mpu;
pub mod registers;
pub mod symbols;
mod system_control;

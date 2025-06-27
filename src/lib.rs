//! Armagnac is a simple ARM Thumb emulation library written in Rust which can be used to emulate
//! simple embedded systems. The library gives high control on the processor execution, allowing to
//! run instruction by instruction, create hooks, inspect or modify the system state on the fly.
//! Integration of custom peripherals in the memory space is made easy, allowing custom platforms
//! emulation.
//!
//! Check [core::Processor] to know how to quickly emulate an ARM binary.
//!
//! Check [memory] to see how peripherals can be implemented and mapped into the processor memory
//! space.

mod align;
mod arith;
pub mod core;
pub mod decoder;
pub mod harness;
pub mod helpers;
pub mod instructions;
pub mod memory;
pub mod mpu;
pub mod registers;
pub mod symbols;
pub mod system_control;

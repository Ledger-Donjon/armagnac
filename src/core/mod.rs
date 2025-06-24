//! Arm processor emulation main module.

mod arm;
mod condition;
mod coprocessor;
mod exclusive_monitor;
mod irq;
mod it_state;

pub use arm::{
    ArmProcessor, ArmVersion, Config, Effect, Emulator, Event, MapConflict, RunError, RunOptions,
};
pub use condition::Condition;
pub use coprocessor::Coprocessor;
pub use exclusive_monitor::{LocalMonitor, MonitorState};
pub use irq::Irq;
pub use it_state::{ItState, ItThenElse};

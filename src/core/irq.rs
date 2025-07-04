//! Defines [Irq] enumeration.

/// Enumeration listing possible interrupts.
///
/// Some interrupt may be specific to the platform running the ARM core (specific peripheral
/// interrupts for instance), those are defined as [Irq::External] interrupts.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum Irq {
    Reset,
    Nmi,
    HardFault,
    MemManage,
    BusFault,
    UsageFault,
    SVCall,
    DebugMonitor,
    PendSV,
    SysTick,
    /// External interrupt.
    /// Value from 0 to (0xffff - 16).
    External(u16),
}

impl Irq {
    /// Returns corresponding exception number.
    pub fn number(&self) -> u16 {
        match self {
            Irq::Reset => 1,
            Irq::Nmi => 2,
            Irq::HardFault => 3,
            Irq::MemManage => 4,
            Irq::BusFault => 5,
            Irq::UsageFault => 6,
            Irq::SVCall => 11,
            Irq::DebugMonitor => 12,
            Irq::PendSV => 14,
            Irq::SysTick => 15,
            Irq::External(n) => 16 + n,
        }
    }
}

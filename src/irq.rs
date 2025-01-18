#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum Irq {
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
            Irq::UsageFault => 6,
            Irq::SVCall => 11,
            Irq::DebugMonitor => 12,
            Irq::PendSV => 14,
            Irq::SysTick => 15,
            Irq::External(n) => 16 + n,
        }
    }
}

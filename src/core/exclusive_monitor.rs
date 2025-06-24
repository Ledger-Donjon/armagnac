/// Local monitor state as defined in the Arm Architecture Reference Manual.
pub enum MonitorState {
    OpenAccess,
    ExclusiveAccess { address: u32 },
}

/// Implements local monitor for exclusive accesses.
pub struct LocalMonitor {
    /// Current monitor state.
    pub state: MonitorState,
    /// Exclusives reservation granule.
    /// Must be a power of two.
    pub granule: u32,
}

impl LocalMonitor {
    pub fn new(granule: u32) -> Self {
        assert!(
            granule.is_power_of_two(),
            "Exclusives reservation granule must be a power of two."
        );
        assert!(
            (granule >= 2) && (granule <= 512),
            "Exclusive reservation granule must be in [2, 512]."
        );
        Self {
            state: MonitorState::OpenAccess,
            granule,
        }
    }
}

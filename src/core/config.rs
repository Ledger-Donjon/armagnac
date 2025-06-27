use crate::core::ArmVersion;

/// Configuration builder used to build instances of [`Processor`].
pub struct Config {
    /// Arm architecture version. Must be defined.
    pub(crate) version: ArmVersion,
    /// Number of platform specific exceptions.
    pub(crate) external_exceptions: usize,
    /// Reservation granule for the local monitor dealing with exclusive accesses.
    pub(crate) exclusives_reservation_granule: u32,
}

impl Config {
    pub fn v6m() -> Self {
        Self {
            version: ArmVersion::V6M,
            external_exceptions: 0,
            exclusives_reservation_granule: 4,
        }
    }

    pub fn v7m() -> Self {
        Self {
            version: ArmVersion::V7M,
            ..Self::v6m()
        }
    }

    pub fn v7em() -> Self {
        Self {
            version: ArmVersion::V7EM,
            ..Self::v6m()
        }
    }

    pub fn v8m() -> Self {
        Self {
            version: ArmVersion::V8M,
            ..Self::v6m()
        }
    }

    /// Sets the number of platform specific exceptions.
    pub fn external_exceptions(mut self, count: usize) -> Self {
        self.external_exceptions = count;
        self
    }

    /// Sets the Exclusive Reservation Granule.
    ///
    /// Value must be a power of two in [4, 512].
    pub fn exclusives_reservation_granule(mut self, granule: u32) -> Self {
        self.exclusives_reservation_granule = granule;
        self
    }
}

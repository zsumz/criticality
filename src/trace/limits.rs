//! Hard ownership limits for one trace.

use crate::retained::RetainedBytes;

/// Count and variable retained-memory limits for one trace.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct TraceLimits {
    records: usize,
    retained_bytes: RetainedBytes,
}

impl TraceLimits {
    /// Creates exact trace ownership limits.
    #[must_use]
    pub const fn new(records: usize, retained_bytes: RetainedBytes) -> Self {
        Self {
            records,
            retained_bytes,
        }
    }

    /// Returns the maximum number of retained records.
    #[must_use]
    pub const fn records(self) -> usize {
        self.records
    }

    /// Returns the maximum variable bytes retained by records.
    #[must_use]
    pub const fn retained_bytes(self) -> RetainedBytes {
        self.retained_bytes
    }
}

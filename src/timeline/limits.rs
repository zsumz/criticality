//! Hard ownership limits for an event timeline.

use crate::retained::RetainedBytes;

/// Count and variable retained-memory limits for one timeline.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TimelineLimits {
    pending_events: usize,
    retained_bytes: RetainedBytes,
}

impl TimelineLimits {
    /// Creates exact timeline ownership limits.
    #[must_use]
    pub const fn new(pending_events: usize, retained_bytes: RetainedBytes) -> Self {
        Self {
            pending_events,
            retained_bytes,
        }
    }

    /// Returns the maximum number of pending events.
    #[must_use]
    pub const fn pending_events(self) -> usize {
        self.pending_events
    }

    /// Returns the maximum variable bytes retained by pending events.
    #[must_use]
    pub const fn retained_bytes(self) -> RetainedBytes {
        self.retained_bytes
    }
}

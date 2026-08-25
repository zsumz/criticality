//! Observable bounded timeline state.

use bytebudget::ByteCount;

use crate::time::Moment;

use super::{TimelineId, TimelineLimits};

/// Copyable observation of one timeline's ownership and next delivery.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TimelineSnapshot {
    id: TimelineId,
    limits: TimelineLimits,
    now: Moment,
    pending_events: usize,
    retained_bytes: ByteCount,
    next_at: Option<Moment>,
}

impl TimelineSnapshot {
    pub(super) const fn new(
        id: TimelineId,
        limits: TimelineLimits,
        now: Moment,
        pending_events: usize,
        retained_bytes: ByteCount,
        next_at: Option<Moment>,
    ) -> Self {
        Self {
            id,
            limits,
            now,
            pending_events,
            retained_bytes,
            next_at,
        }
    }

    /// Returns the observed timeline identity.
    #[must_use]
    pub const fn id(self) -> TimelineId {
        self.id
    }

    /// Returns the configured hard limits.
    #[must_use]
    pub const fn limits(self) -> TimelineLimits {
        self.limits
    }

    /// Returns observed virtual time.
    #[must_use]
    pub const fn now(self) -> Moment {
        self.now
    }

    /// Returns the pending-event count.
    #[must_use]
    pub const fn pending_events(self) -> usize {
        self.pending_events
    }

    /// Returns variable bytes retained by pending events.
    #[must_use]
    pub const fn retained_bytes(self) -> ByteCount {
        self.retained_bytes
    }

    /// Returns the earliest pending delivery moment.
    #[must_use]
    pub const fn next_at(self) -> Option<Moment> {
        self.next_at
    }
}

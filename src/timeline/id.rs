//! Timeline-scoped stable event identities.

use crate::time::Moment;

/// Incarnation identity of one independently created timeline owner.
///
/// Consumers must not reuse an identity for another timeline while tokens from
/// the earlier incarnation may still exist. Creating an empty timeline at an
/// explicit moment starts event identity at zero; it does not restore a prior
/// incarnation's identity sequence.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TimelineId(u64);

impl TimelineId {
    /// Creates a timeline incarnation identity.
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// Returns the raw identity value.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

/// Stable insertion identity of one event within a timeline.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct EventId(u64);

impl EventId {
    pub(crate) const fn new(value: u64) -> Self {
        Self(value)
    }

    /// Returns the raw identity value.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

/// Exact capability naming one scheduled event.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct EventToken<P = ()> {
    timeline: TimelineId,
    id: EventId,
    at: Moment,
    phase: P,
}

impl<P> EventToken<P> {
    pub(super) const fn new(timeline: TimelineId, id: EventId, at: Moment, phase: P) -> Self {
        Self {
            timeline,
            id,
            at,
            phase,
        }
    }

    /// Returns the timeline incarnation identity.
    #[must_use]
    pub const fn timeline_id(&self) -> TimelineId {
        self.timeline
    }

    /// Returns the stable insertion identity.
    #[must_use]
    pub const fn id(&self) -> EventId {
        self.id
    }

    /// Returns the scheduled delivery moment.
    #[must_use]
    pub const fn at(&self) -> Moment {
        self.at
    }

    /// Borrows the consumer-defined same-moment phase.
    #[must_use]
    pub const fn phase(&self) -> &P {
        &self.phase
    }

    pub(super) fn into_key(self) -> (Moment, P, EventId) {
        (self.at, self.phase, self.id)
    }
}

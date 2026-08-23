//! Count- and byte-bounded deterministic event ownership.

use alloc::collections::BTreeMap;

use crate::{
    retained::{Retained, RetainedBytes},
    time::{Moment, VirtualClock},
};

use super::{Delivery, EventId, EventToken, TimelineId, TimelineLimits, TimelineSnapshot};

mod schedule;

type EventKey<P> = (Moment, P, EventId);

#[derive(Clone, Debug)]
struct Entry<E> {
    event: E,
    retained: RetainedBytes,
}

/// Bounded timeline ordered by moment, consumer phase, and insertion identity.
#[derive(Clone, Debug)]
pub struct Timeline<E, P = ()> {
    id: TimelineId,
    clock: VirtualClock,
    limits: TimelineLimits,
    measure: fn(&E) -> RetainedBytes,
    next_id: Option<EventId>,
    retained: RetainedBytes,
    events: BTreeMap<EventKey<P>, Entry<E>>,
}

impl<E: Retained, P: Ord> Timeline<E, P> {
    /// Creates an empty timeline at [`Moment::ORIGIN`].
    #[must_use]
    pub fn new(id: TimelineId, limits: TimelineLimits) -> Self {
        Self::with_measure(id, limits, E::retained_bytes)
    }

    /// Creates an empty timeline restored at an explicit virtual moment.
    #[must_use]
    pub fn at(id: TimelineId, now: Moment, limits: TimelineLimits) -> Self {
        Self::at_with_measure(id, now, limits, E::retained_bytes)
    }
}

impl<E, P: Ord> Timeline<E, P> {
    /// Creates an origin timeline using an explicit event measurement function.
    #[must_use]
    pub fn with_measure(
        id: TimelineId,
        limits: TimelineLimits,
        measure: fn(&E) -> RetainedBytes,
    ) -> Self {
        Self::at_with_measure(id, Moment::ORIGIN, limits, measure)
    }

    /// Creates a restored timeline using an explicit event measurement function.
    #[must_use]
    pub fn at_with_measure(
        id: TimelineId,
        now: Moment,
        limits: TimelineLimits,
        measure: fn(&E) -> RetainedBytes,
    ) -> Self {
        Self {
            id,
            clock: VirtualClock::at(now),
            limits,
            measure,
            next_id: Some(EventId::new(0)),
            retained: RetainedBytes::ZERO,
            events: BTreeMap::new(),
        }
    }

    /// Returns this timeline's stable identity.
    #[must_use]
    pub const fn id(&self) -> TimelineId {
        self.id
    }

    /// Returns current virtual time.
    #[must_use]
    pub const fn now(&self) -> Moment {
        self.clock.now()
    }

    /// Returns configured ownership limits.
    #[must_use]
    pub const fn limits(&self) -> TimelineLimits {
        self.limits
    }

    /// Returns the pending-event count.
    #[must_use]
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// Returns whether no event remains pending.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Returns the earliest pending delivery moment.
    #[must_use]
    pub fn next_at(&self) -> Option<Moment> {
        self.events.first_key_value().map(|(key, _)| key.0)
    }

    /// Captures bounded ownership and next-delivery state.
    #[must_use]
    pub fn snapshot(&self) -> TimelineSnapshot {
        TimelineSnapshot::new(
            self.id,
            self.limits,
            self.clock.now(),
            self.events.len(),
            self.retained,
            self.next_at(),
        )
    }
}

impl<E, P: Clone + Ord> Timeline<E, P> {
    /// Removes and returns the earliest event, advancing time to its moment.
    pub fn pop_next(&mut self) -> Option<Delivery<E, P>> {
        let (_, first) = self.events.first_key_value()?;
        let retained = self.retained.checked_sub(first.retained)?;
        let ((at, phase, id), entry) = self.events.pop_first()?;
        self.clock = VirtualClock::at(at);
        self.retained = retained;
        Some(Delivery::new(
            EventToken::new(self.id, id, at, phase),
            entry.event,
        ))
    }
}

impl<E, P: Ord> Timeline<E, P> {
    /// Cancels the exact pending event named by `token` and returns its value.
    ///
    /// Tokens from other timelines and tokens for events no longer pending do
    /// not mutate this timeline and return `None`.
    pub fn cancel(&mut self, token: EventToken<P>) -> Option<E> {
        if token.timeline() != self.id {
            return None;
        }
        let key = token.into_key();
        let entry = self.events.get(&key)?;
        let retained = self.retained.checked_sub(entry.retained)?;
        let entry = self.events.remove(&key)?;
        self.retained = retained;
        Some(entry.event)
    }
}

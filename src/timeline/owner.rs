//! Count- and byte-bounded deterministic event ownership.

use alloc::collections::BTreeMap;

use crate::{
    ByteCount, Retained,
    time::{Moment, VirtualClock},
};

use super::{Delivery, EventId, EventToken, TimelineId, TimelineLimits, TimelineSnapshot};

mod schedule;

type EventKey<P> = (Moment, P, EventId);

#[derive(Debug)]
struct Entry<E> {
    event: E,
    retained: ByteCount,
}

/// Bounded timeline ordered by moment, consumer phase, and insertion identity.
///
/// Phases are finite structural values. Variable retained data belongs in the
/// measured event rather than an unmeasured ordering key.
///
/// ```compile_fail
/// use criticality::timeline::Timeline;
///
/// let _: Option<Timeline<(), String>> = None;
/// ```
///
/// Charge-bearing owners are deliberately not cloneable. Cloned events are
/// new admissions and can retain a different number of bytes.
///
/// ```compile_fail
/// use criticality::timeline::Timeline;
/// fn require_clone<T: Clone>() {}
/// require_clone::<Timeline<()>>();
/// ```
#[derive(Debug)]
pub struct Timeline<E, P: Copy + Ord = ()> {
    id: TimelineId,
    clock: VirtualClock,
    limits: TimelineLimits,
    measure: fn(&E) -> ByteCount,
    next_id: Option<EventId>,
    retained: ByteCount,
    events: BTreeMap<EventKey<P>, Entry<E>>,
}

impl<E: Retained, P: Copy + Ord> Timeline<E, P> {
    /// Creates an empty timeline at [`Moment::ORIGIN`].
    #[must_use]
    pub fn new(id: TimelineId, limits: TimelineLimits) -> Self {
        Self::with_measure(id, limits, E::retained_bytes)
    }

    /// Creates an empty timeline at an explicit virtual moment.
    #[must_use]
    pub fn empty_at(id: TimelineId, now: Moment, limits: TimelineLimits) -> Self {
        Self::empty_at_with_measure(id, now, limits, E::retained_bytes)
    }
}

impl<E, P: Copy + Ord> Timeline<E, P> {
    /// Creates an origin timeline using an explicit event measurement function.
    ///
    /// `measure` must follow the same retained-storage model as [`Retained`].
    #[must_use]
    pub fn with_measure(
        id: TimelineId,
        limits: TimelineLimits,
        measure: fn(&E) -> ByteCount,
    ) -> Self {
        Self::empty_at_with_measure(id, Moment::ORIGIN, limits, measure)
    }

    /// Creates an empty timeline at an explicit moment using an event measure.
    ///
    /// `measure` must follow the same retained-storage model as [`Retained`].
    #[must_use]
    pub fn empty_at_with_measure(
        id: TimelineId,
        now: Moment,
        limits: TimelineLimits,
        measure: fn(&E) -> ByteCount,
    ) -> Self {
        Self {
            id,
            clock: VirtualClock::at(now),
            limits,
            measure,
            next_id: Some(EventId::new(0)),
            retained: ByteCount::ZERO,
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

impl<E, P: Copy + Ord> Timeline<E, P> {
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

impl<E, P: Copy + Ord> Timeline<E, P> {
    /// Cancels the exact pending event named by `token` and returns its value.
    ///
    /// Tokens from other timelines and tokens for events no longer pending do
    /// not mutate this timeline and return `None`.
    pub fn cancel(&mut self, token: EventToken<P>) -> Option<E> {
        if token.timeline_id() != self.id {
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

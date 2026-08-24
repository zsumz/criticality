//! Ownership-preserving event admission.

use crate::{
    plan::Planned,
    time::{Moment, Span},
};

use super::{Entry, Timeline};
use crate::timeline::{EventId, EventToken, ScheduleError, ScheduleFailure};

impl<E, P: Copy + Ord> Timeline<E, P> {
    /// Schedules an event at an absolute moment and explicit phase.
    ///
    /// # Errors
    ///
    /// Returns ownership of `event` when time, identity, count, or byte
    /// constraints reject admission. Rejection does not mutate the timeline.
    pub fn schedule_at_in(
        &mut self,
        at: Moment,
        phase: P,
        event: E,
    ) -> Result<EventToken<P>, ScheduleError<E>> {
        if at < self.clock.now() {
            return Err(ScheduleError::new(
                event,
                ScheduleFailure::ScheduledInPast {
                    current: self.clock.now(),
                    requested: at,
                },
            ));
        }
        if self.events.len() >= self.limits.pending_events() {
            return Err(ScheduleError::new(
                event,
                ScheduleFailure::EventCapacity {
                    limit: self.limits.pending_events(),
                },
            ));
        }
        let measured = (self.measure)(&event);
        let Some(retained) = self.retained.checked_add(measured) else {
            return Err(ScheduleError::new(
                event,
                ScheduleFailure::RetainedByteOverflow {
                    current: self.retained,
                    event: measured,
                },
            ));
        };
        if retained > self.limits.retained_bytes() {
            return Err(ScheduleError::new(
                event,
                ScheduleFailure::RetainedByteCapacity {
                    limit: self.limits.retained_bytes(),
                    current: self.retained,
                    event: measured,
                },
            ));
        }
        let Some(id) = self.next_id else {
            return Err(ScheduleError::new(
                event,
                ScheduleFailure::EventIdsExhausted,
            ));
        };
        let token = EventToken::new(self.id, id, at, phase);
        let entry = Entry {
            event,
            retained: measured,
        };
        let _ = self.events.insert((at, phase, id), entry);
        self.retained = retained;
        self.next_id = id.get().checked_add(1).map(EventId::new);
        Ok(token)
    }

    /// Schedules an event relative to current time and in an explicit phase.
    ///
    /// # Errors
    ///
    /// Returns ownership of `event` when checked time arithmetic or admission
    /// constraints reject it. Rejection does not mutate the timeline.
    pub fn schedule_after_in(
        &mut self,
        delay: Span,
        phase: P,
        event: E,
    ) -> Result<EventToken<P>, ScheduleError<E>> {
        let Some(at) = self.clock.now().checked_add(delay) else {
            return Err(ScheduleError::new(
                event,
                ScheduleFailure::TimeOverflow {
                    current: self.clock.now(),
                    delay,
                },
            ));
        };
        self.schedule_at_in(at, phase, event)
    }

    /// Schedules an owned planned event in an explicit phase.
    ///
    /// # Errors
    ///
    /// Returns ownership of the planned outcome when checked time arithmetic
    /// or admission constraints reject it.
    pub fn schedule_planned_in(
        &mut self,
        phase: P,
        planned: Planned<E>,
    ) -> Result<EventToken<P>, ScheduleError<E>> {
        let (delay, event) = planned.into_parts();
        self.schedule_after_in(delay, phase, event)
    }
}

impl<E> Timeline<E> {
    /// Schedules an event at an absolute moment using the unit phase.
    ///
    /// # Errors
    ///
    /// Returns ownership of `event` when admission fails.
    pub fn schedule_at(&mut self, at: Moment, event: E) -> Result<EventToken, ScheduleError<E>> {
        self.schedule_at_in(at, (), event)
    }

    /// Schedules an event relative to current time using the unit phase.
    ///
    /// # Errors
    ///
    /// Returns ownership of `event` when checked time arithmetic or admission fails.
    pub fn schedule_after(
        &mut self,
        delay: Span,
        event: E,
    ) -> Result<EventToken, ScheduleError<E>> {
        self.schedule_after_in(delay, (), event)
    }

    /// Schedules an owned planned event using the unit phase.
    ///
    /// # Errors
    ///
    /// Returns ownership of the planned outcome when admission fails.
    pub fn schedule_planned(
        &mut self,
        planned: Planned<E>,
    ) -> Result<EventToken, ScheduleError<E>> {
        self.schedule_planned_in((), planned)
    }
}

//! Ownership-preserving timeline admission failures.

use core::fmt;

use crate::{
    retained::RetainedBytes,
    time::{Moment, Span},
};

/// Why a timeline rejected one event.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScheduleFailure {
    /// The requested moment precedes current virtual time.
    ScheduledInPast {
        /// Current timeline moment.
        current: Moment,
        /// Rejected delivery moment.
        requested: Moment,
    },
    /// Adding a relative delay exceeded the fixed-width time domain.
    TimeOverflow {
        /// Current timeline moment.
        current: Moment,
        /// Rejected relative delay.
        delay: Span,
    },
    /// The pending-event count limit is full.
    EventCapacity {
        /// Configured pending-event limit.
        limit: usize,
    },
    /// Retained-byte addition exceeded the fixed-width accounting domain.
    RetainedByteOverflow {
        /// Bytes retained before admission.
        current: RetainedBytes,
        /// Bytes measured for the rejected event.
        event: RetainedBytes,
    },
    /// The event would exceed the retained-byte limit.
    RetainedByteCapacity {
        /// Configured retained-byte limit.
        limit: RetainedBytes,
        /// Bytes retained before admission.
        current: RetainedBytes,
        /// Bytes measured for the rejected event.
        event: RetainedBytes,
    },
    /// Every stable event identity has been issued.
    EventIdsExhausted,
}

impl fmt::Display for ScheduleFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ScheduledInPast { .. } => {
                formatter.write_str("cannot schedule before current virtual time")
            }
            Self::TimeOverflow { .. } => formatter.write_str("scheduled time would overflow"),
            Self::EventCapacity { .. } => formatter.write_str("pending event capacity was reached"),
            Self::RetainedByteOverflow { .. } => {
                formatter.write_str("retained-byte accounting would overflow")
            }
            Self::RetainedByteCapacity { .. } => {
                formatter.write_str("retained-byte capacity would be exceeded")
            }
            Self::EventIdsExhausted => formatter.write_str("event identities are exhausted"),
        }
    }
}

impl core::error::Error for ScheduleFailure {}

/// Admission failure retaining ownership of the rejected event.
#[derive(Debug)]
pub struct ScheduleError<E> {
    event: E,
    failure: ScheduleFailure,
}

impl<E> ScheduleError<E> {
    pub(super) const fn new(event: E, failure: ScheduleFailure) -> Self {
        Self { event, failure }
    }

    /// Returns the reason admission failed.
    #[must_use]
    pub const fn failure(&self) -> ScheduleFailure {
        self.failure
    }

    /// Returns ownership of the rejected event.
    #[must_use]
    pub fn into_event(self) -> E {
        self.event
    }
}

impl<E> fmt::Display for ScheduleError<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.failure.fmt(formatter)
    }
}

impl<E: fmt::Debug> core::error::Error for ScheduleError<E> {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        Some(&self.failure)
    }
}

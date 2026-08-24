//! Monotonic virtual-time primitives.

use core::fmt;

/// One absolute point in an owner-defined simulation time domain.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Moment(u64);

impl Moment {
    /// The origin of every virtual-time domain.
    pub const ORIGIN: Self = Self(0);

    /// Creates a moment from an abstract simulation tick.
    #[must_use]
    pub const fn from_tick(tick: u64) -> Self {
        Self(tick)
    }

    /// Returns the abstract simulation tick.
    #[must_use]
    pub const fn tick(self) -> u64 {
        self.0
    }

    /// Advances by `span`, returning `None` if the time domain overflows.
    #[must_use]
    pub const fn checked_add(self, span: Span) -> Option<Self> {
        match self.0.checked_add(span.0) {
            Some(tick) => Some(Self(tick)),
            None => None,
        }
    }

    /// Returns elapsed time since `earlier`, or `None` if it is later.
    #[must_use]
    pub const fn checked_duration_since(self, earlier: Self) -> Option<Span> {
        match self.0.checked_sub(earlier.0) {
            Some(ticks) => Some(Span(ticks)),
            None => None,
        }
    }
}

/// A nonnegative duration in abstract simulation ticks.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Span(u64);

impl Span {
    /// A zero-length span.
    pub const ZERO: Self = Self(0);

    /// Creates a span from abstract simulation ticks.
    #[must_use]
    pub const fn from_ticks(ticks: u64) -> Self {
        Self(ticks)
    }

    /// Returns the number of abstract simulation ticks.
    #[must_use]
    pub const fn ticks(self) -> u64 {
        self.0
    }

    /// Adds two spans, returning `None` if the time domain overflows.
    #[must_use]
    pub const fn checked_add(self, other: Self) -> Option<Self> {
        match self.0.checked_add(other.0) {
            Some(ticks) => Some(Self(ticks)),
            None => None,
        }
    }
}

/// An absolute moment by which an owner must receive another turn.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Deadline(Moment);

impl Deadline {
    /// Creates an absolute deadline.
    #[must_use]
    pub const fn at(moment: Moment) -> Self {
        Self(moment)
    }

    /// Returns the absolute deadline moment.
    #[must_use]
    pub const fn moment(self) -> Moment {
        self.0
    }

    /// Returns whether the deadline is due at `now`.
    #[must_use]
    pub const fn is_elapsed_at(self, now: Moment) -> bool {
        self.0.0 <= now.0
    }

    /// Returns the remaining span, clamped to zero once elapsed.
    #[must_use]
    pub const fn remaining_at(self, now: Moment) -> Span {
        match self.0.0.checked_sub(now.0) {
            Some(ticks) => Span(ticks),
            None => Span::ZERO,
        }
    }
}

/// A deterministic clock with no relationship to wall time.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct VirtualClock {
    now: Moment,
}

impl VirtualClock {
    /// Creates a clock at [`Moment::ORIGIN`].
    #[must_use]
    pub const fn new() -> Self {
        Self {
            now: Moment::ORIGIN,
        }
    }

    /// Creates a clock at an explicit virtual moment.
    #[must_use]
    pub const fn at(now: Moment) -> Self {
        Self { now }
    }

    /// Returns current virtual time.
    #[must_use]
    pub const fn now(&self) -> Moment {
        self.now
    }

    /// Moves time to `target` without permitting a backward transition.
    ///
    /// # Errors
    ///
    /// Returns [`ClockError::MovesBackward`] when `target` precedes the
    /// current moment. The clock remains unchanged on error.
    pub fn advance_to(&mut self, target: Moment) -> Result<(), ClockError> {
        if target < self.now {
            return Err(ClockError::MovesBackward {
                current: self.now,
                requested: target,
            });
        }
        self.now = target;
        Ok(())
    }

    /// Advances virtual time by `span` with checked arithmetic.
    ///
    /// # Errors
    ///
    /// Returns [`ClockError::Overflow`] when the resulting moment does not fit
    /// in the fixed-width time domain. The clock remains unchanged on error.
    pub fn advance_by(&mut self, span: Span) -> Result<(), ClockError> {
        let Some(target) = self.now.checked_add(span) else {
            return Err(ClockError::Overflow {
                current: self.now,
                span,
            });
        };
        self.now = target;
        Ok(())
    }
}

/// Why a requested virtual-clock transition was rejected.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClockError {
    /// The requested moment precedes current virtual time.
    MovesBackward {
        /// Current virtual time.
        current: Moment,
        /// Rejected earlier moment.
        requested: Moment,
    },
    /// Adding a span exceeded the fixed-width time domain.
    Overflow {
        /// Current virtual time.
        current: Moment,
        /// Rejected span.
        span: Span,
    },
}

impl fmt::Display for ClockError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MovesBackward { .. } => formatter.write_str("virtual time cannot move backward"),
            Self::Overflow { .. } => formatter.write_str("advancing virtual time would overflow"),
        }
    }
}

impl core::error::Error for ClockError {}

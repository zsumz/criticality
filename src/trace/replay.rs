//! Finite equality-based exact replay positions and divergence reports.

use alloc::boxed::Box;
use core::fmt;

/// A zero-based position in an exact replay sequence.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ReplayPosition(usize);

impl ReplayPosition {
    /// The first expected record.
    pub const ORIGIN: Self = Self(0);

    /// Creates a position from its index.
    #[must_use]
    pub const fn new(index: usize) -> Self {
        Self(index)
    }

    /// Returns the zero-based index.
    #[must_use]
    pub const fn get(self) -> usize {
        self.0
    }
}

/// The first reason an exact replay could not complete.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ReplayFailure {
    /// An actual record arrived after the expected sequence ended.
    Exhausted {
        /// Position where the unexpected record arrived.
        position: ReplayPosition,
    },
    /// The actual record differed from the next expected record.
    Mismatch {
        /// Position of the first unequal record.
        position: ReplayPosition,
    },
    /// Replay stopped before consuming the complete expected sequence.
    Remaining {
        /// Position of the next unconsumed record.
        position: ReplayPosition,
        /// Number of records still expected.
        remaining: usize,
    },
}

impl ReplayFailure {
    /// Returns the exact divergence or incomplete position.
    #[must_use]
    pub const fn position(self) -> ReplayPosition {
        match self {
            Self::Exhausted { position }
            | Self::Mismatch { position }
            | Self::Remaining { position, .. } => position,
        }
    }
}

impl fmt::Display for ReplayFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Exhausted { .. } => formatter.write_str("exact replay is exhausted"),
            Self::Mismatch { .. } => formatter.write_str("record differs from exact replay"),
            Self::Remaining { .. } => formatter.write_str("exact replay has remaining records"),
        }
    }
}

impl core::error::Error for ReplayFailure {}

/// A finite exact sequence of opaque consumer-owned records.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ExactReplay<T> {
    expected: Box<[T]>,
    position: ReplayPosition,
}

impl<T> ExactReplay<T> {
    /// Creates a replay from one exact finite expected sequence.
    #[must_use]
    pub fn new(expected: Box<[T]>) -> Self {
        Self {
            expected,
            position: ReplayPosition::ORIGIN,
        }
    }

    /// Returns the next expected record without advancing replay.
    #[must_use]
    pub fn expected(&self) -> Option<&T> {
        self.expected.get(self.position.get())
    }

    /// Returns the position of the next expected record.
    #[must_use]
    pub const fn position(&self) -> ReplayPosition {
        self.position
    }

    /// Returns the number of unconsumed expected records.
    #[must_use]
    pub fn remaining(&self) -> usize {
        self.expected.len().saturating_sub(self.position.get())
    }

    /// Returns whether every expected record was consumed.
    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.remaining() == 0
    }

    /// Confirms the next actual record and advances only on exact equality.
    ///
    /// # Errors
    ///
    /// Returns [`ReplayFailure::Exhausted`] after the sequence ends or
    /// [`ReplayFailure::Mismatch`] at the first unequal record. Neither failure
    /// advances the replay position.
    pub fn observe(&mut self, actual: &T) -> Result<(), ReplayFailure>
    where
        T: PartialEq,
    {
        let position = self.position;
        let Some(expected) = self.expected() else {
            return Err(ReplayFailure::Exhausted { position });
        };
        if expected != actual {
            return Err(ReplayFailure::Mismatch { position });
        }
        self.position = ReplayPosition::new(position.get() + 1);
        Ok(())
    }

    /// Confirms that no expected records remain.
    ///
    /// # Errors
    ///
    /// Returns [`ReplayFailure::Remaining`] with the next position and exact
    /// remaining count when replay is incomplete.
    pub fn finish(&self) -> Result<(), ReplayFailure> {
        let remaining = self.remaining();
        if remaining == 0 {
            Ok(())
        } else {
            Err(ReplayFailure::Remaining {
                position: self.position,
                remaining,
            })
        }
    }
}

//! Explicit retained-memory accounting.

use core::{fmt, num::TryFromIntError};

/// Variable memory retained by one value.
///
/// Fixed collection and envelope overhead is bounded separately by item count.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RetainedBytes(u64);

impl RetainedBytes {
    /// No retained variable memory.
    pub const ZERO: Self = Self(0);

    /// Creates a retained-byte value.
    #[must_use]
    pub const fn new(bytes: u64) -> Self {
        Self(bytes)
    }

    /// Returns the number of retained bytes.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }

    /// Adds two byte counts, returning `None` on overflow.
    #[must_use]
    pub const fn checked_add(self, other: Self) -> Option<Self> {
        match self.0.checked_add(other.0) {
            Some(bytes) => Some(Self(bytes)),
            None => None,
        }
    }

    /// Subtracts a byte count, returning `None` when `other` is larger.
    #[must_use]
    pub const fn checked_sub(self, other: Self) -> Option<Self> {
        match self.0.checked_sub(other.0) {
            Some(bytes) => Some(Self(bytes)),
            None => None,
        }
    }
}

impl From<u32> for RetainedBytes {
    fn from(value: u32) -> Self {
        Self(u64::from(value))
    }
}

impl From<u64> for RetainedBytes {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl TryFrom<usize> for RetainedBytes {
    type Error = RetainedBytesOverflow;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        u64::try_from(value)
            .map(Self)
            .map_err(RetainedBytesOverflow::from)
    }
}

/// A value whose variable retained memory can be measured before admission.
pub trait Retained {
    /// Returns variable bytes retained while this value remains owned.
    ///
    /// Implementations must be deterministic, fast, and independent of
    /// mutable interior state. A bounded structure measures a value once at
    /// admission and stores that measurement beside the value.
    fn retained_bytes(&self) -> RetainedBytes;
}

impl Retained for () {
    fn retained_bytes(&self) -> RetainedBytes {
        RetainedBytes::ZERO
    }
}

/// A platform `usize` could not fit in the fixed-width accounting domain.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RetainedBytesOverflow;

impl From<TryFromIntError> for RetainedBytesOverflow {
    fn from(_: TryFromIntError) -> Self {
        Self
    }
}

impl fmt::Display for RetainedBytesOverflow {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("retained byte count exceeds the u64 accounting domain")
    }
}

impl core::error::Error for RetainedBytesOverflow {}

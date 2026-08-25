//! Ownership-preserving trace admission failures.

use core::fmt;

use bytebudget::ByteCount;

/// Why a trace rejected one record.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TraceFailure {
    /// The record-count limit is full.
    RecordCapacity {
        /// Configured record limit.
        limit: usize,
    },
    /// Retained-byte addition exceeded the fixed-width accounting domain.
    RetainedByteOverflow {
        /// Bytes retained before admission.
        current: ByteCount,
        /// Bytes measured for the rejected record.
        record: ByteCount,
    },
    /// The record would exceed the retained-byte limit.
    RetainedByteCapacity {
        /// Configured retained-byte limit.
        limit: ByteCount,
        /// Bytes retained before admission.
        current: ByteCount,
        /// Bytes measured for the rejected record.
        record: ByteCount,
    },
}

impl fmt::Display for TraceFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RecordCapacity { .. } => formatter.write_str("trace record capacity was reached"),
            Self::RetainedByteOverflow { .. } => {
                formatter.write_str("trace retained-byte accounting would overflow")
            }
            Self::RetainedByteCapacity { .. } => {
                formatter.write_str("trace retained-byte capacity would be exceeded")
            }
        }
    }
}

impl core::error::Error for TraceFailure {}

/// Admission failure retaining ownership of the rejected record.
#[derive(Debug)]
pub struct TraceError<T> {
    record: T,
    failure: TraceFailure,
}

impl<T> TraceError<T> {
    pub(super) const fn new(record: T, failure: TraceFailure) -> Self {
        Self { record, failure }
    }

    /// Returns the reason admission failed.
    #[must_use]
    pub const fn failure(&self) -> TraceFailure {
        self.failure
    }

    /// Returns ownership of the rejected record.
    #[must_use]
    pub fn into_record(self) -> T {
        self.record
    }
}

impl<T> fmt::Display for TraceError<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.failure.fmt(formatter)
    }
}

impl<T: fmt::Debug> core::error::Error for TraceError<T> {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        Some(&self.failure)
    }
}

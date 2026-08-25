//! Count- and byte-bounded deterministic record ownership.

use alloc::vec::Vec;
use bytebudget::{ByteBudget, ByteCount, Retained};

use super::{ExactReplay, TraceError, TraceFailure, TraceLimits};

/// A bounded, append-only typed trace.
#[derive(Debug)]
pub struct Trace<T> {
    limits: TraceLimits,
    measure: fn(&T) -> ByteCount,
    budget: ByteBudget,
    records: Vec<T>,
}

impl<T: Clone> Clone for Trace<T> {
    fn clone(&self) -> Self {
        let mut budget = ByteBudget::new(self.budget.limit());
        // Reconstruct the exact aggregate without making live budgets cloneable.
        let reservation = budget.try_reserve(self.budget.used());
        assert!(
            reservation.is_ok(),
            "bytebudget invariant was violated while cloning a trace"
        );
        Self {
            limits: self.limits,
            measure: self.measure,
            budget,
            records: self.records.clone(),
        }
    }
}

impl<T: Retained> Trace<T> {
    /// Creates an empty trace using [`Retained`] measurement.
    #[must_use]
    pub fn new(limits: TraceLimits) -> Self {
        Self::with_measure(limits, T::retained_bytes)
    }
}

impl<T> Trace<T> {
    /// Creates an empty trace using an explicit record measurement function.
    ///
    /// `measure` must follow the same retained-storage model as [`Retained`].
    /// Each attempted record is measured at most once after count preflight.
    #[must_use]
    pub const fn with_measure(limits: TraceLimits, measure: fn(&T) -> ByteCount) -> Self {
        Self {
            limits,
            measure,
            budget: ByteBudget::new(limits.retained_bytes()),
            records: Vec::new(),
        }
    }

    /// Returns configured hard ownership limits.
    #[must_use]
    pub const fn limits(&self) -> TraceLimits {
        self.limits
    }

    /// Returns the retained record count.
    #[must_use]
    pub fn len(&self) -> usize {
        self.records.len()
    }

    /// Returns whether no record has been retained.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    /// Returns variable bytes retained by all records.
    #[must_use]
    pub const fn retained_bytes(&self) -> ByteCount {
        self.budget.used()
    }

    /// Borrows all records in exact admission order.
    #[must_use]
    pub const fn as_slice(&self) -> &[T] {
        self.records.as_slice()
    }

    /// Iterates over records in exact admission order.
    #[must_use]
    pub fn iter(&self) -> impl ExactSizeIterator<Item = &T> {
        self.records.iter()
    }

    /// Creates an exact replay borrowing this trace's bounded evidence.
    #[must_use]
    pub fn replay(&self) -> ExactReplay<'_, T> {
        ExactReplay::new(self.as_slice())
    }

    /// Admits one record or returns it unchanged on failure.
    ///
    /// # Errors
    ///
    /// Returns ownership of `record` when count or measured-byte limits reject
    /// admission. Rejection does not mutate the trace.
    pub fn try_push(&mut self, record: T) -> Result<(), TraceError<T>> {
        if self.records.len() >= self.limits.records() {
            return Err(TraceError::new(
                record,
                TraceFailure::RecordCapacity {
                    limit: self.limits.records(),
                },
            ));
        }
        let measured = (self.measure)(&record);
        let current = self.budget.used();
        if current.checked_add(measured).is_none() {
            return Err(TraceError::new(
                record,
                TraceFailure::RetainedByteOverflow {
                    current,
                    record: measured,
                },
            ));
        }
        if self.budget.try_reserve(measured).is_err() {
            return Err(TraceError::new(
                record,
                TraceFailure::RetainedByteCapacity {
                    limit: self.limits.retained_bytes(),
                    current,
                    record: measured,
                },
            ));
        }
        self.records.push(record);
        Ok(())
    }

    /// Returns all retained records in exact admission order.
    #[must_use]
    pub fn into_records(self) -> Vec<T> {
        self.records
    }
}

//! Count- and byte-bounded deterministic record ownership.

use alloc::vec::Vec;

use crate::retained::{Retained, RetainedBytes};

use super::{TraceError, TraceFailure, TraceLimits};

#[derive(Clone, Debug)]
struct Entry<T> {
    record: T,
    retained_after: RetainedBytes,
}

/// A bounded, append-only typed trace.
#[derive(Clone, Debug)]
pub struct Trace<T> {
    limits: TraceLimits,
    measure: fn(&T) -> RetainedBytes,
    records: Vec<Entry<T>>,
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
    #[must_use]
    pub const fn with_measure(limits: TraceLimits, measure: fn(&T) -> RetainedBytes) -> Self {
        Self {
            limits,
            measure,
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
    pub fn retained_bytes(&self) -> RetainedBytes {
        self.records
            .last()
            .map_or(RetainedBytes::ZERO, |entry| entry.retained_after)
    }

    /// Iterates over records in exact admission order.
    #[must_use]
    pub fn iter(&self) -> impl ExactSizeIterator<Item = &T> {
        self.records.iter().map(|entry| &entry.record)
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
        let current = self.retained_bytes();
        let Some(retained) = current.checked_add(measured) else {
            return Err(TraceError::new(
                record,
                TraceFailure::RetainedByteOverflow {
                    current,
                    record: measured,
                },
            ));
        };
        if retained > self.limits.retained_bytes() {
            return Err(TraceError::new(
                record,
                TraceFailure::RetainedByteCapacity {
                    limit: self.limits.retained_bytes(),
                    current,
                    record: measured,
                },
            ));
        }
        self.records.push(Entry {
            record,
            retained_after: retained,
        });
        Ok(())
    }

    /// Returns all retained records in exact admission order.
    #[must_use]
    pub fn into_records(self) -> Vec<T> {
        self.records.into_iter().map(|entry| entry.record).collect()
    }
}

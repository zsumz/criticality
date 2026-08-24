//! Public trace admission, accounting, and ownership contracts.

use core::cell::Cell;

use criticality::{
    retained::{Retained, RetainedBytes},
    trace::{Trace, TraceFailure, TraceLimits},
};

#[derive(Debug)]
struct Record {
    id: u8,
    bytes: RetainedBytes,
    measurements: Cell<u8>,
}

impl Record {
    const fn new(id: u8, bytes: u64) -> Self {
        Self {
            id,
            bytes: RetainedBytes::new(bytes),
            measurements: Cell::new(0),
        }
    }
}

impl Retained for Record {
    fn retained_bytes(&self) -> RetainedBytes {
        self.measurements.set(self.measurements.get() + 1);
        self.bytes
    }
}

#[test]
fn count_and_byte_rejections_preserve_record_ownership() {
    let limits = TraceLimits::new(1, RetainedBytes::new(4));
    let mut trace = Trace::new(limits);
    assert!(trace.try_push(Record::new(1, 4)).is_ok());

    let result = trace.try_push(Record::new(2, 1));
    assert!(result.is_err());
    let Err(error) = result else {
        return;
    };
    assert!(error.failure() == TraceFailure::RecordCapacity { limit: 1 });
    assert!(error.into_record().id == 2);

    let mut trace = Trace::new(TraceLimits::new(2, RetainedBytes::new(4)));
    assert!(trace.try_push(Record::new(3, 4)).is_ok());
    let result = trace.try_push(Record::new(4, 1));
    assert!(result.is_err());
    let Err(error) = result else {
        return;
    };
    assert!(is_byte_capacity(error.failure()));
    assert!(error.into_record().id == 4);
}

#[test]
fn retained_measurement_occurs_once_and_snapshot_is_exact() {
    let limits = TraceLimits::new(2, RetainedBytes::new(8));
    let mut trace = Trace::new(limits);
    assert!(trace.is_empty());
    assert!(trace.try_push(Record::new(1, 3)).is_ok());
    let snapshot = (trace.limits(), trace.len(), trace.retained_bytes());
    assert!(snapshot == (limits, 1, RetainedBytes::new(3)));
    let Some(record) = trace.iter().next() else {
        return;
    };
    assert!(record.measurements.get() == 1);
    assert!(trace.retained_bytes() == RetainedBytes::new(3));
}

#[test]
fn retained_byte_overflow_is_distinct_and_non_mutating() {
    let limits = TraceLimits::new(2, RetainedBytes::new(u64::MAX));
    let mut trace = Trace::new(limits);
    assert!(trace.try_push(Record::new(1, u64::MAX)).is_ok());
    let result = trace.try_push(Record::new(2, 1));
    assert!(result.is_err());
    let Err(error) = result else {
        return;
    };
    assert!(is_byte_overflow(error.failure()));
    assert!(error.into_record().id == 2);
    assert!(trace.len() == 1);
    assert!(trace.retained_bytes() == RetainedBytes::new(u64::MAX));
}

#[test]
fn explicit_measurement_supports_foreign_record_types() {
    let record = String::from("four");
    let measured = measure_string(&record);
    let limits = TraceLimits::new(1, measured);
    let mut trace = Trace::with_measure(limits, measure_string);
    assert!(trace.try_push(record).is_ok());
    assert!(trace.retained_bytes() == measured);
    assert!(trace.into_records() == [String::from("four")]);
}

const fn is_byte_capacity(failure: TraceFailure) -> bool {
    let TraceFailure::RetainedByteCapacity { .. } = failure else {
        return false;
    };
    true
}

const fn is_byte_overflow(failure: TraceFailure) -> bool {
    let TraceFailure::RetainedByteOverflow { .. } = failure else {
        return false;
    };
    true
}

fn measure_string(value: &String) -> RetainedBytes {
    match RetainedBytes::try_from(value.capacity()) {
        Ok(bytes) => bytes,
        Err(_) => RetainedBytes::new(u64::MAX),
    }
}

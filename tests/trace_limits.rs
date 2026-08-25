//! Public trace admission, accounting, and ownership contracts.

use bytebudget::{ByteCount, Retained};

use criticality::trace::{Trace, TraceFailure, TraceLimits};

#[derive(Clone, Copy, Debug)]
struct Record {
    id: u8,
    bytes: ByteCount,
}

impl Record {
    const fn new(id: u8, bytes: u64) -> Self {
        Self {
            id,
            bytes: ByteCount::new(bytes),
        }
    }
}

impl Retained for Record {
    fn retained_bytes(&self) -> ByteCount {
        self.bytes
    }
}

#[derive(Clone, Debug)]
struct MeasuredRecord {
    id: u8,
    bytes: ByteCount,
    measurements: core::cell::Cell<u8>,
}

impl MeasuredRecord {
    const fn new(id: u8, bytes: u64) -> Self {
        Self {
            id,
            bytes: ByteCount::new(bytes),
            measurements: core::cell::Cell::new(0),
        }
    }
}

#[test]
fn count_and_byte_rejections_preserve_record_ownership() {
    let limits = TraceLimits::new(1, ByteCount::new(4));
    let mut trace = Trace::new(limits);
    assert!(trace.try_push(Record::new(1, 4)).is_ok());

    let result = trace.try_push(Record::new(2, 1));
    assert!(result.is_err());
    let Err(error) = result else {
        return;
    };
    assert!(error.failure() == TraceFailure::RecordCapacity { limit: 1 });
    assert!(error.into_record().id == 2);

    let mut trace = Trace::new(TraceLimits::new(2, ByteCount::new(4)));
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
    let limits = TraceLimits::new(2, ByteCount::new(8));
    let mut trace = Trace::with_measure(limits, measure_record);
    assert!(trace.is_empty());
    assert!(trace.try_push(MeasuredRecord::new(1, 3)).is_ok());
    let snapshot = (trace.limits(), trace.len(), trace.retained_bytes());
    assert!(snapshot == (limits, 1, ByteCount::new(3)));
    let Some(record) = trace.iter().next() else {
        return;
    };
    assert!(record.measurements.get() == 1);
    assert!(trace.as_slice()[0].id == 1);
    assert!(trace.retained_bytes() == ByteCount::new(3));
}

#[test]
fn cloning_trace_preserves_an_independent_aggregate_budget() {
    let limits = TraceLimits::new(3, ByteCount::new(5));
    let mut trace = Trace::with_measure(limits, measure_record);
    assert!(trace.try_push(MeasuredRecord::new(1, 3)).is_ok());

    let mut cloned = trace.clone();
    assert!(cloned.retained_bytes() == ByteCount::new(3));
    assert!(cloned.as_slice()[0].measurements.get() == 1);
    assert!(cloned.try_push(MeasuredRecord::new(2, 2)).is_ok());
    let rejected = cloned.try_push(MeasuredRecord::new(3, 1));
    assert!(rejected.is_err());
    let Err(error) = rejected else {
        return;
    };
    assert!(is_byte_capacity(error.failure()));
    let rejected = error.into_record();
    assert!(rejected.id == 3);
    assert!(rejected.measurements.get() == 1);

    assert!(trace.try_push(MeasuredRecord::new(4, 2)).is_ok());

    assert!(trace.len() == 2);
    assert!(trace.retained_bytes() == ByteCount::new(5));
    assert!(cloned.len() == 2);
    assert!(cloned.retained_bytes() == ByteCount::new(5));
    assert!(trace.as_slice()[0].measurements.get() == 1);
    assert!(cloned.as_slice()[0].measurements.get() == 1);
    assert!(trace.as_slice()[1].measurements.get() == 1);
    assert!(cloned.as_slice()[1].measurements.get() == 1);
}

#[test]
fn retained_byte_overflow_is_distinct_and_non_mutating() {
    let limits = TraceLimits::new(2, ByteCount::MAX);
    let mut trace = Trace::new(limits);
    assert!(trace.try_push(Record::new(1, u64::MAX)).is_ok());
    let cloned = trace.clone();
    assert!(cloned.len() == 1);
    assert!(cloned.retained_bytes() == ByteCount::MAX);
    let result = trace.try_push(Record::new(2, 1));
    assert!(result.is_err());
    let Err(error) = result else {
        return;
    };
    assert!(is_byte_overflow(error.failure()));
    assert!(error.into_record().id == 2);
    assert!(trace.len() == 1);
    assert!(trace.retained_bytes() == ByteCount::MAX);
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

fn measure_string(value: &String) -> ByteCount {
    match ByteCount::try_from(value.capacity()) {
        Ok(bytes) => bytes,
        Err(_) => ByteCount::MAX,
    }
}

fn measure_record(record: &MeasuredRecord) -> ByteCount {
    record.measurements.set(record.measurements.get() + 1);
    record.bytes
}

//! Public timeline admission and ownership contracts.

use bytebudget::{ByteCount, Retained};

use criticality::{
    time::{Moment, Span},
    timeline::{ScheduleFailure, Timeline, TimelineId, TimelineLimits},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Event {
    id: u8,
    bytes: ByteCount,
}

impl Retained for Event {
    fn retained_bytes(&self) -> ByteCount {
        self.bytes
    }
}

#[test]
fn count_and_byte_rejections_preserve_event_ownership() {
    let mut timeline = Timeline::new(
        TimelineId::new(1),
        TimelineLimits::new(1, ByteCount::new(4)),
    );
    assert!(timeline.schedule_after(Span::ZERO, event(1, 4)).is_ok());

    let rejected = event(2, 1);
    let result = timeline.schedule_after(Span::ZERO, rejected);
    assert!(result.is_err(), "count capacity must reject the event");
    let Err(error) = result else {
        return;
    };
    assert!(error.failure() == ScheduleFailure::EventCapacity { limit: 1 });
    assert!(error.into_event() == rejected);

    let delivery = timeline.pop_next();
    assert!(delivery.is_some(), "admitted event must remain owned");
    let Some(delivery) = delivery else {
        return;
    };
    assert!(delivery.into_event() == event(1, 4));

    let mut byte_limited = Timeline::new(
        TimelineId::new(2),
        TimelineLimits::new(2, ByteCount::new(4)),
    );
    assert!(byte_limited.schedule_after(Span::ZERO, event(3, 4)).is_ok());
    let rejected = event(4, 1);
    let result = byte_limited.schedule_after(Span::ZERO, rejected);
    assert!(result.is_err(), "byte capacity must reject the event");
    let Err(error) = result else {
        return;
    };
    assert!(matches_byte_capacity(error.failure()));
    assert!(error.into_event() == rejected);
}

#[test]
fn overflow_and_past_rejections_leave_timeline_unchanged() {
    let limits = TimelineLimits::new(2, ByteCount::MAX);
    let mut timeline = Timeline::empty_at(TimelineId::new(2), Moment::from_tick(5), limits);
    let past = event(1, 1);
    let result = timeline.schedule_at(Moment::from_tick(4), past);
    assert!(result.is_err(), "past scheduling must reject");
    let Err(error) = result else {
        return;
    };
    assert!(matches_past(error.failure()));
    assert!(error.into_event() == past);

    assert!(
        timeline
            .schedule_at(Moment::from_tick(5), event(2, u64::MAX))
            .is_ok()
    );
    let overflow = event(3, 1);
    let result = timeline.schedule_at(Moment::from_tick(5), overflow);
    assert!(result.is_err(), "byte addition must not wrap");
    let Err(error) = result else {
        return;
    };
    assert!(matches_byte_overflow(error.failure()));
    assert!(error.into_event() == overflow);
    assert!(timeline.now() == Moment::from_tick(5));
    assert!(timeline.len() == 1);
}

#[test]
fn relative_time_overflow_and_zero_capacity_preserve_events() {
    let mut full_time = Timeline::empty_at(
        TimelineId::new(3),
        Moment::from_tick(u64::MAX),
        TimelineLimits::new(1, ByteCount::new(1)),
    );
    let rejected = event(1, 1);
    let result = full_time.schedule_after(Span::from_ticks(1), rejected);
    assert!(result.is_err(), "relative time overflow must reject");
    let Err(error) = result else {
        return;
    };
    assert!(matches_time_overflow(error.failure()));
    assert!(error.into_event() == rejected);

    let mut zero = Timeline::new(
        TimelineId::new(4),
        TimelineLimits::new(0, ByteCount::new(8)),
    );
    let rejected = event(2, 0);
    let result = zero.schedule_after(Span::ZERO, rejected);
    assert!(result.is_err(), "zero event capacity must reject");
    let Err(error) = result else {
        return;
    };
    assert!(error.failure() == ScheduleFailure::EventCapacity { limit: 0 });
    assert!(error.into_event() == rejected);
}

#[test]
fn explicit_measurement_supports_foreign_event_types() {
    let mut timeline = Timeline::with_measure(
        TimelineId::new(5),
        TimelineLimits::new(1, ByteCount::new(4)),
        measure_string,
    );
    assert!(
        timeline
            .schedule_after(Span::ZERO, String::from("four"))
            .is_ok()
    );
    assert!(timeline.snapshot().retained_bytes() == ByteCount::new(4));
}

const fn event(id: u8, bytes: u64) -> Event {
    Event {
        id,
        bytes: ByteCount::new(bytes),
    }
}

fn measure_string(value: &String) -> ByteCount {
    match ByteCount::try_from(value.capacity()) {
        Ok(bytes) => bytes,
        Err(_) => ByteCount::MAX,
    }
}

const fn matches_past(failure: ScheduleFailure) -> bool {
    let ScheduleFailure::ScheduledInPast { .. } = failure else {
        return false;
    };
    true
}

const fn matches_time_overflow(failure: ScheduleFailure) -> bool {
    let ScheduleFailure::TimeOverflow { .. } = failure else {
        return false;
    };
    true
}

const fn matches_byte_overflow(failure: ScheduleFailure) -> bool {
    let ScheduleFailure::RetainedByteOverflow { .. } = failure else {
        return false;
    };
    true
}

const fn matches_byte_capacity(failure: ScheduleFailure) -> bool {
    let ScheduleFailure::RetainedByteCapacity { .. } = failure else {
        return false;
    };
    true
}

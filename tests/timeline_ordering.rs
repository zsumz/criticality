//! Public timeline ordering and observation contracts.

use bytebudget::{ByteCount, Retained};

use criticality::{
    time::{Moment, Span},
    timeline::{Timeline, TimelineId, TimelineLimits},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Event(u8);

impl Retained for Event {
    fn retained_bytes(&self) -> ByteCount {
        ByteCount::ZERO
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum Phase {
    External,
    Internal,
    Timer,
}

#[test]
fn timeline_orders_by_moment_phase_and_insertion_identity() {
    let limits = TimelineLimits::new(8, ByteCount::ZERO);
    let mut timeline = Timeline::<Event, Phase>::new(TimelineId::new(7), limits);

    assert!(
        timeline
            .schedule_at_in(Moment::from_tick(5), Phase::Timer, Event(4))
            .is_ok()
    );
    assert!(
        timeline
            .schedule_at_in(Moment::from_tick(5), Phase::External, Event(1))
            .is_ok()
    );
    assert!(
        timeline
            .schedule_at_in(Moment::from_tick(5), Phase::Internal, Event(3))
            .is_ok()
    );
    assert!(
        timeline
            .schedule_at_in(Moment::from_tick(5), Phase::External, Event(2))
            .is_ok()
    );
    assert!(
        timeline
            .schedule_at_in(Moment::from_tick(2), Phase::Timer, Event(0))
            .is_ok()
    );

    for (expected, at, phase) in [
        (Event(0), Moment::from_tick(2), Phase::Timer),
        (Event(1), Moment::from_tick(5), Phase::External),
        (Event(2), Moment::from_tick(5), Phase::External),
        (Event(3), Moment::from_tick(5), Phase::Internal),
        (Event(4), Moment::from_tick(5), Phase::Timer),
    ] {
        let delivery = timeline.pop_next();
        assert!(delivery.is_some(), "expected a scheduled delivery");
        let Some(delivery) = delivery else {
            return;
        };
        assert!(delivery.event() == &expected);
        assert!(delivery.at() == at);
        assert!(delivery.phase() == &phase);
    }
    assert!(timeline.pop_next().is_none());
    assert!(timeline.now() == Moment::from_tick(5));
}

#[test]
fn snapshots_report_exact_bounded_state() {
    let limits = TimelineLimits::new(2, ByteCount::new(8));
    let mut timeline = Timeline::<Event>::new(TimelineId::new(9), limits);
    assert!(
        timeline
            .schedule_after(Span::from_ticks(3), Event(1))
            .is_ok()
    );

    let snapshot = timeline.snapshot();
    assert!(snapshot.id() == TimelineId::new(9));
    assert!(snapshot.limits() == limits);
    assert!(snapshot.now() == Moment::ORIGIN);
    assert!(snapshot.pending_events() == 1);
    assert!(snapshot.retained_bytes() == ByteCount::ZERO);
    assert!(snapshot.next_at() == Some(Moment::from_tick(3)));
}

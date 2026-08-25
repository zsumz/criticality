//! Dispatcher consumer composition with consumer-defined phases and effects.

use criticality::{
    ByteCount, Retained,
    time::Moment,
    timeline::{Timeline, TimelineId, TimelineLimits},
    trace::{Trace, TraceLimits},
};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum Phase {
    External,
    Internal,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Event {
    Dispatch(u8),
    Complete(u8),
}

impl Retained for Event {
    fn retained_bytes(&self) -> ByteCount {
        ByteCount::ZERO
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Record {
    Accepted(u8),
    Completed(u8),
}

impl Retained for Record {
    fn retained_bytes(&self) -> ByteCount {
        ByteCount::ZERO
    }
}

#[derive(Debug, Default)]
struct Dispatcher {
    accepted: u8,
    completed: u8,
}

#[test]
fn dispatcher_owns_transition_effects_and_phase_policy() {
    let at = Moment::from_tick(5);
    let mut machine = Dispatcher::default();
    let mut timeline = Timeline::new(TimelineId::new(2), TimelineLimits::new(4, ByteCount::ZERO));
    let mut trace = Trace::new(TraceLimits::new(4, ByteCount::ZERO));
    assert!(
        timeline
            .schedule_at_in(at, Phase::External, Event::Dispatch(1))
            .is_ok()
    );
    assert!(
        timeline
            .schedule_at_in(at, Phase::External, Event::Dispatch(2))
            .is_ok()
    );

    while let Some(delivery) = timeline.pop_next() {
        match *delivery.event() {
            Event::Dispatch(id) => {
                machine.accepted += 1;
                assert!(trace.try_push(Record::Accepted(id)).is_ok());
                assert!(
                    timeline
                        .schedule_at_in(at, Phase::Internal, Event::Complete(id))
                        .is_ok()
                );
            }
            Event::Complete(id) => {
                machine.completed += 1;
                assert!(trace.try_push(Record::Completed(id)).is_ok());
            }
        }
    }

    assert!(machine.accepted == 2);
    assert!(machine.completed == 2);
    assert!(
        trace.as_slice()
            == [
                Record::Accepted(1),
                Record::Accepted(2),
                Record::Completed(1),
                Record::Completed(2),
            ]
    );
}

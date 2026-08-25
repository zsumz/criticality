//! Watchdog consumer composition with heartbeat-owned deadline policy.

use criticality::{
    ByteCount, Retained,
    time::{Deadline, Moment},
    timeline::{EventToken, Timeline, TimelineId, TimelineLimits},
    trace::{Trace, TraceLimits},
};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum Phase {
    Heartbeat,
    Timer,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Event {
    Heartbeat,
    Timeout,
}

impl Retained for Event {
    fn retained_bytes(&self) -> ByteCount {
        ByteCount::ZERO
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Record {
    Heartbeat(Moment),
    Expired(Moment),
}

impl Retained for Record {
    fn retained_bytes(&self) -> ByteCount {
        ByteCount::ZERO
    }
}

#[derive(Debug)]
struct Watchdog {
    deadline: Deadline,
    timer: Option<EventToken<Phase>>,
    expired: bool,
}

#[test]
fn watchdog_owns_heartbeat_deadline_and_expiration_policy() {
    let mut timeline = Timeline::new(TimelineId::new(6), TimelineLimits::new(2, ByteCount::ZERO));
    let initial_deadline = Deadline::at(Moment::from_tick(10));
    let timer = timeline.schedule_at_in(initial_deadline.moment(), Phase::Timer, Event::Timeout);
    assert!(timer.is_ok());
    let Ok(timer) = timer else {
        return;
    };
    assert!(
        timeline
            .schedule_at_in(Moment::from_tick(5), Phase::Heartbeat, Event::Heartbeat)
            .is_ok()
    );
    let mut watchdog = Watchdog {
        deadline: initial_deadline,
        timer: Some(timer),
        expired: false,
    };
    let mut trace = Trace::new(TraceLimits::new(2, ByteCount::ZERO));

    while let Some(delivery) = timeline.pop_next() {
        match *delivery.event() {
            Event::Heartbeat => {
                assert!(trace.try_push(Record::Heartbeat(delivery.at())).is_ok());
                let cancelled = watchdog
                    .timer
                    .take()
                    .and_then(|token| timeline.cancel(token));
                assert!(cancelled == Some(Event::Timeout));
                watchdog.deadline = Deadline::at(Moment::from_tick(15));
                let replacement = timeline.schedule_at_in(
                    watchdog.deadline.moment(),
                    Phase::Timer,
                    Event::Timeout,
                );
                assert!(replacement.is_ok());
                let Ok(replacement) = replacement else {
                    return;
                };
                watchdog.timer = Some(replacement);
            }
            Event::Timeout => {
                watchdog.timer = None;
                watchdog.expired = watchdog.deadline.is_elapsed_at(delivery.at());
                assert!(trace.try_push(Record::Expired(delivery.at())).is_ok());
            }
        }
    }

    assert!(watchdog.expired);
    assert!(
        trace.as_slice()
            == [
                Record::Heartbeat(Moment::from_tick(5)),
                Record::Expired(Moment::from_tick(15)),
            ]
    );
}

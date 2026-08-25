//! Retrying-client consumer composition without framework-owned policy.

use criticality::{
    ByteCount, Retained,
    plan::{Plan, Planned},
    time::Span,
    timeline::{Timeline, TimelineId, TimelineLimits},
    trace::{Trace, TraceLimits},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Attempt(u8);

impl Retained for Attempt {
    fn retained_bytes(&self) -> ByteCount {
        ByteCount::ZERO
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Attempted(u8);

impl Retained for Attempted {
    fn retained_bytes(&self) -> ByteCount {
        ByteCount::ZERO
    }
}

#[derive(Debug)]
struct RetryingClient {
    attempts: u8,
    maximum: u8,
}

impl RetryingClient {
    const fn new(maximum: u8) -> Self {
        Self {
            attempts: 0,
            maximum,
        }
    }

    fn on_attempt(&mut self, attempt: Attempt) -> Plan<Attempt> {
        self.attempts += 1;
        if self.attempts < self.maximum {
            Plan::single(Planned::new(Span::from_ticks(2), Attempt(attempt.0 + 1)))
        } else {
            Plan::empty()
        }
    }
}

#[test]
fn retrying_client_owns_retry_policy_and_run_loop() {
    let mut client = RetryingClient::new(3);
    let mut timeline = Timeline::new(TimelineId::new(1), TimelineLimits::new(1, ByteCount::ZERO));
    let mut trace = Trace::new(TraceLimits::new(3, ByteCount::ZERO));
    assert!(timeline.schedule_after(Span::ZERO, Attempt(0)).is_ok());

    while let Some(delivery) = timeline.pop_next() {
        let attempt = *delivery.event();
        assert!(trace.try_push(Attempted(attempt.0)).is_ok());
        let plan = client.on_attempt(attempt);
        for planned in plan.into_outcomes() {
            assert!(timeline.schedule_planned(planned).is_ok());
        }
    }

    assert!(client.attempts == 3);
    assert!(timeline.now().tick() == 4);
    let expected = [Attempted(0), Attempted(1), Attempted(2)];
    let mut replay = trace.replay();
    for record in &expected {
        assert!(replay.observe(record).is_ok());
    }
    assert!(replay.finish().is_ok());
}

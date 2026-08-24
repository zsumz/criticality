//! Relay composition from exact script through replayed delivery evidence.

use std::vec::Vec;

use criticality::{
    plan::{Plan, Planned},
    retained::{Retained, RetainedBytes},
    script::{ExactScript, ScriptLimits, ScriptStep},
    time::{Moment, Span},
    timeline::{EventToken, ScheduleFailure, Timeline, TimelineId, TimelineLimits},
    trace::{Trace, TraceLimits},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Request(u8);

impl Retained for Request {
    fn retained_bytes(&self) -> RetainedBytes {
        RetainedBytes::ZERO
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Packet([u8; 16]);

const fn packet(id: u8) -> Packet {
    let mut bytes = [0; 16];
    bytes[0] = id;
    Packet(bytes)
}

impl Retained for Packet {
    fn retained_bytes(&self) -> RetainedBytes {
        RetainedBytes::ZERO
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum Phase {
    Network,
}

#[derive(Debug, Eq, PartialEq)]
struct ApplyFailure {
    failure: ScheduleFailure,
    plan: Plan<Packet>,
}

fn multipart_plan() -> Plan<Packet> {
    Plan::new(Vec::from([
        Planned::new(Span::from_ticks(3), packet(1)),
        Planned::new(Span::from_ticks(1), packet(2)),
        Planned::new(Span::from_ticks(2), packet(3)),
    ]))
}

fn overflowing_plan() -> Plan<Packet> {
    Plan::new(Vec::from([
        Planned::new(Span::ZERO, packet(1)),
        Planned::new(Span::from_ticks(1), packet(2)),
        Planned::new(Span::ZERO, packet(3)),
    ]))
}

fn apply_plan_atomically(
    timeline: &mut Timeline<Packet, Phase>,
    plan: Plan<Packet>,
) -> Result<Vec<EventToken<Phase>>, ApplyFailure> {
    let available = timeline.limits().pending_events() - timeline.len();
    if plan.len() > available {
        return Err(ApplyFailure {
            failure: ScheduleFailure::EventCapacity {
                limit: timeline.limits().pending_events(),
            },
            plan,
        });
    }

    let overflow = plan
        .iter()
        .map(Planned::delay)
        .find(|delay| timeline.now().checked_add(*delay).is_none());
    if let Some(delay) = overflow {
        return Err(ApplyFailure {
            failure: ScheduleFailure::TimeOverflow {
                current: timeline.now(),
                delay,
            },
            plan,
        });
    }

    let mut outcomes = plan.into_outcomes().into_iter();
    let mut admitted = Vec::with_capacity(outcomes.len());
    while let Some(planned) = outcomes.next() {
        let (delay, packet) = planned.into_parts();
        match timeline.schedule_after_in(delay, Phase::Network, packet) {
            Ok(token) => admitted.push((token, delay)),
            Err(error) => {
                let failure = error.failure();
                let rejected = error.into_event();
                let mut restored = Vec::with_capacity(admitted.len() + 1 + outcomes.len());
                for (token, admitted_delay) in admitted {
                    let cancelled = timeline.cancel(token);
                    assert!(
                        cancelled.is_some(),
                        "an admitted token must remain cancellable"
                    );
                    if let Some(packet) = cancelled {
                        restored.push(Planned::new(admitted_delay, packet));
                    }
                }
                restored.push(Planned::new(delay, rejected));
                restored.extend(outcomes);
                return Err(ApplyFailure {
                    failure,
                    plan: Plan::new(restored),
                });
            }
        }
    }
    Ok(admitted.into_iter().map(|(token, _)| token).collect())
}

#[test]
fn relay_composes_script_plan_timeline_trace_and_replay() {
    let step = ScriptStep::new(Request(7), multipart_plan());
    let built = ExactScript::try_new(
        ScriptLimits::new(1, 3, RetainedBytes::ZERO),
        Vec::from([step]),
    );
    assert!(built.is_ok());
    let Ok(mut script) = built else {
        return;
    };
    let plan = script.respond(&Request(7));
    assert!(plan.is_ok());
    let Ok(plan) = plan else {
        return;
    };

    let mut timeline = Timeline::new(
        TimelineId::new(4),
        TimelineLimits::new(3, RetainedBytes::ZERO),
    );
    let applied = apply_plan_atomically(&mut timeline, plan);
    assert!(applied.is_ok());
    let Ok(tokens) = applied else {
        return;
    };
    assert!(tokens.len() == 3);

    let mut trace = Trace::new(TraceLimits::new(3, RetainedBytes::ZERO));
    while let Some(delivery) = timeline.pop_next() {
        assert!(trace.try_push(*delivery.event()).is_ok());
    }
    let expected = [packet(2), packet(3), packet(1)];
    let mut replay = trace.replay();
    for packet in &expected {
        assert!(replay.observe(packet).is_ok());
    }
    assert!(replay.finish().is_ok());
}

#[test]
fn relay_preflights_count_without_mutation() {
    let plan = multipart_plan();
    let mut timeline = Timeline::new(
        TimelineId::new(5),
        TimelineLimits::new(2, RetainedBytes::ZERO),
    );
    let before = timeline.snapshot();
    let result = apply_plan_atomically(&mut timeline, plan);
    let Err(failure) = result else {
        return;
    };
    assert_eq!(failure.failure, ScheduleFailure::EventCapacity { limit: 2 });
    assert_eq!(failure.plan, multipart_plan());
    assert_eq!(timeline.snapshot(), before);
}

#[test]
fn relay_preserves_atomic_multi_outcome_admission() {
    let plan = overflowing_plan();
    let mut full_time = Timeline::empty_at(
        TimelineId::new(6),
        Moment::from_tick(u64::MAX),
        TimelineLimits::new(3, RetainedBytes::ZERO),
    );
    let before = full_time.snapshot();
    let result = apply_plan_atomically(&mut full_time, plan);
    let Err(failure) = result else {
        return;
    };
    assert_eq!(
        failure.failure,
        ScheduleFailure::TimeOverflow {
            current: Moment::from_tick(u64::MAX),
            delay: Span::from_ticks(1),
        }
    );
    assert_eq!(failure.plan, overflowing_plan());
    assert_eq!(full_time.snapshot(), before);
}

#[test]
fn relay_rolls_back_an_unexpected_later_failure() {
    let plan = Plan::new(Vec::from([
        Planned::new(Span::ZERO, packet(1)),
        Planned::new(Span::ZERO, packet(2)),
        Planned::new(Span::ZERO, packet(3)),
    ]));
    let mut timeline = Timeline::with_measure(
        TimelineId::new(8),
        TimelineLimits::new(3, RetainedBytes::ZERO),
        measure_packet_two,
    );
    let before = timeline.snapshot();
    let result = apply_plan_atomically(&mut timeline, plan);
    let Err(failure) = result else {
        return;
    };
    assert_eq!(
        failure.failure,
        ScheduleFailure::RetainedByteCapacity {
            limit: RetainedBytes::ZERO,
            current: RetainedBytes::ZERO,
            event: RetainedBytes::new(1),
        }
    );
    assert_eq!(
        failure.plan,
        Plan::new(Vec::from([
            Planned::new(Span::ZERO, packet(1)),
            Planned::new(Span::ZERO, packet(2)),
            Planned::new(Span::ZERO, packet(3)),
        ]))
    );
    assert_eq!(timeline.snapshot(), before);
}

const fn measure_packet_two(packet: &Packet) -> RetainedBytes {
    if packet.0[0] == 2 {
        RetainedBytes::new(1)
    } else {
        RetainedBytes::ZERO
    }
}

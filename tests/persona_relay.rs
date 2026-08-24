//! Relay composition from exact script through replayed delivery evidence.

use std::vec::Vec;

use criticality::{
    plan::{Plan, Planned},
    retained::{Retained, RetainedBytes},
    script::{ExactScript, ScriptLimits, ScriptStep},
    time::{Moment, Span},
    timeline::{EventToken, Timeline, TimelineId, TimelineLimits},
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
struct Packet(u8);

impl Retained for Packet {
    fn retained_bytes(&self) -> RetainedBytes {
        RetainedBytes::ZERO
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum Phase {
    Network,
}

#[derive(Debug)]
enum ApplyFailure {
    Capacity(Plan<Packet>),
    Admission(Packet),
}

fn multipart_plan() -> Plan<Packet> {
    Plan::new(Vec::from([
        Planned::new(Span::from_ticks(3), Packet(1)),
        Planned::new(Span::from_ticks(1), Packet(2)),
        Planned::new(Span::from_ticks(2), Packet(3)),
    ]))
}

fn apply_zero_retained_plan(
    timeline: &mut Timeline<Packet, Phase>,
    plan: Plan<Packet>,
) -> Result<Vec<EventToken<Phase>>, ApplyFailure> {
    let available = timeline.limits().pending_events() - timeline.len();
    if plan.len() > available {
        return Err(ApplyFailure::Capacity(plan));
    }

    let mut tokens = Vec::with_capacity(plan.len());
    for planned in plan.into_outcomes() {
        match timeline.schedule_planned_in(Phase::Network, planned) {
            Ok(token) => tokens.push(token),
            Err(error) => return Err(ApplyFailure::Admission(error.into_event())),
        }
    }
    Ok(tokens)
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
    let applied = apply_zero_retained_plan(&mut timeline, plan);
    assert!(applied.is_ok());
    let Ok(tokens) = applied else {
        return;
    };
    assert!(tokens.len() == 3);

    let mut trace = Trace::new(TraceLimits::new(3, RetainedBytes::ZERO));
    while let Some(delivery) = timeline.pop_next() {
        assert!(trace.try_push(*delivery.event()).is_ok());
    }
    let expected = [Packet(2), Packet(3), Packet(1)];
    let mut replay = trace.replay();
    for packet in &expected {
        assert!(replay.observe(packet).is_ok());
    }
    assert!(replay.finish().is_ok());
}

#[test]
fn relay_preflights_atomic_multi_outcome_admission() {
    let plan = multipart_plan();
    let mut timeline = Timeline::new(
        TimelineId::new(5),
        TimelineLimits::new(2, RetainedBytes::ZERO),
    );
    let result = apply_zero_retained_plan(&mut timeline, plan);
    assert!(result.is_err());
    let Err(failure) = result else {
        return;
    };
    let ApplyFailure::Capacity(rejected) = failure else {
        return;
    };
    assert!(rejected.as_slice() == multipart_plan().as_slice());
    assert!(timeline.is_empty());

    let overflow = Plan::single(Planned::new(Span::from_ticks(1), Packet(9)));
    let mut full_time = Timeline::empty_at(
        TimelineId::new(6),
        Moment::from_tick(u64::MAX),
        TimelineLimits::new(1, RetainedBytes::ZERO),
    );
    let result = apply_zero_retained_plan(&mut full_time, overflow);
    let Err(ApplyFailure::Admission(packet)) = result else {
        return;
    };
    assert!(packet == Packet(9));
    assert!(full_time.is_empty());
}

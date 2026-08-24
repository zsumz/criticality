//! Public planned-outcome timeline integration contracts.

use criticality::{
    plan::Planned,
    retained::{Retained, RetainedBytes},
    time::{Moment, Span},
    timeline::{Timeline, TimelineId, TimelineLimits},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Event(u8);

impl Retained for Event {
    fn retained_bytes(&self) -> RetainedBytes {
        RetainedBytes::ZERO
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum Phase {
    External,
}

#[test]
fn planned_outcomes_schedule_with_owned_delay_and_phase() {
    let mut timeline = Timeline::<Event, Phase>::new(
        TimelineId::new(1),
        TimelineLimits::new(1, RetainedBytes::ZERO),
    );
    let result =
        timeline.schedule_planned_in(Phase::External, Planned::new(Span::from_ticks(7), Event(1)));
    assert!(result.is_ok());
    let Ok(token) = result else {
        return;
    };
    assert!(token.timeline_id() == TimelineId::new(1));
    assert!(token.at() == Moment::from_tick(7));
    assert!(token.phase() == &Phase::External);

    let delivery = timeline.pop_next();
    assert!(delivery.is_some());
    let Some(delivery) = delivery else {
        return;
    };
    assert!(delivery.into_event() == Event(1));
    assert!(timeline.now() == Moment::from_tick(7));
}

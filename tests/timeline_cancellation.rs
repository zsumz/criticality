//! Public exact timeline cancellation contracts.

use criticality::{
    retained::{Retained, RetainedBytes},
    time::{Moment, Span},
    timeline::{Timeline, TimelineId, TimelineLimits},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Event {
    id: u8,
    bytes: RetainedBytes,
}

impl Retained for Event {
    fn retained_bytes(&self) -> RetainedBytes {
        self.bytes
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum Phase {
    External,
    Timer,
}

#[test]
fn cancellation_returns_ownership_and_releases_accounting() {
    let limits = TimelineLimits::new(2, RetainedBytes::new(8));
    let mut timeline = Timeline::<Event, Phase>::new(TimelineId::new(1), limits);
    let result = timeline.schedule_after_in(Span::from_ticks(5), Phase::Timer, event(7, 8));
    assert!(result.is_ok());
    let Ok(token) = result else {
        return;
    };

    assert!(timeline.cancel(token) == Some(event(7, 8)));
    assert!(timeline.snapshot().pending_events() == 0);
    assert!(timeline.snapshot().retained_bytes() == RetainedBytes::ZERO);
    assert!(timeline.now() == Moment::ORIGIN);
    assert!(timeline.cancel(token).is_none());
}

#[test]
fn foreign_tokens_cannot_cancel_local_events() {
    let limits = TimelineLimits::new(2, RetainedBytes::ZERO);
    let mut first = Timeline::<Event, Phase>::new(TimelineId::new(1), limits);
    let mut second = Timeline::<Event, Phase>::new(TimelineId::new(2), limits);
    let foreign = first.schedule_at_in(Moment::from_tick(3), Phase::Timer, event(1, 0));
    let local = second.schedule_at_in(Moment::from_tick(3), Phase::External, event(2, 0));
    assert!(foreign.is_ok());
    assert!(local.is_ok());
    let Ok(foreign) = foreign else {
        return;
    };
    let Ok(local) = local else {
        return;
    };

    assert!(second.cancel(foreign).is_none());
    assert!(second.len() == 1);
    assert!(second.cancel(local) == Some(event(2, 0)));
}

#[test]
fn stale_tokens_cannot_cancel_events_in_a_new_incarnation() {
    let limits = TimelineLimits::new(1, RetainedBytes::ZERO);
    let at = Moment::from_tick(3);
    let mut earlier = Timeline::<Event, Phase>::new(TimelineId::new(7), limits);
    let stale = earlier.schedule_at_in(at, Phase::Timer, event(1, 0));
    assert!(stale.is_ok());
    let Ok(stale) = stale else {
        return;
    };
    drop(earlier);

    let mut current =
        Timeline::<Event, Phase>::empty_at(TimelineId::new(8), Moment::ORIGIN, limits);
    let admitted = current.schedule_at_in(at, Phase::Timer, event(2, 0));
    assert!(admitted.is_ok());
    let Ok(admitted) = admitted else {
        return;
    };

    assert!(stale.id() == admitted.id());
    assert!(stale.at() == admitted.at());
    assert!(stale.phase() == admitted.phase());
    assert!(current.cancel(stale).is_none());
    assert!(current.cancel(admitted) == Some(event(2, 0)));
}

const fn event(id: u8, bytes: u64) -> Event {
    Event {
        id,
        bytes: RetainedBytes::new(bytes),
    }
}

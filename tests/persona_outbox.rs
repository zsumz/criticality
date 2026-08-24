//! Outbox consumer composition with consumer-owned supersession policy.

use criticality::{
    retained::{Retained, RetainedBytes},
    time::{Moment, Span},
    timeline::{EventToken, ScheduleFailure, Timeline, TimelineId, TimelineLimits},
    trace::{Trace, TraceLimits},
};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum Phase {
    Publish,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Publish {
    revision: u8,
}

impl Retained for Publish {
    fn retained_bytes(&self) -> RetainedBytes {
        RetainedBytes::ZERO
    }
}

#[derive(Debug, Default)]
struct Outbox {
    pending: Option<EventToken<Phase>>,
    published_revision: Option<u8>,
}

#[derive(Debug, Eq, PartialEq)]
enum StageFailure {
    ReplacementRejected {
        failure: ScheduleFailure,
        rejected: Publish,
        superseded: Option<Publish>,
    },
}

impl Outbox {
    fn stage(
        &mut self,
        timeline: &mut Timeline<Publish, Phase>,
        revision: u8,
        delay: Span,
    ) -> Result<Option<Publish>, StageFailure> {
        let replaced = self.pending.take().and_then(|token| timeline.cancel(token));
        match timeline.schedule_after_in(delay, Phase::Publish, Publish { revision }) {
            Ok(token) => {
                self.pending = Some(token);
                Ok(replaced)
            }
            Err(error) => Err(StageFailure::ReplacementRejected {
                failure: error.failure(),
                rejected: error.into_event(),
                superseded: replaced,
            }),
        }
    }

    fn confirm(&mut self, published: Publish) {
        self.pending = None;
        self.published_revision = Some(published.revision);
    }
}

#[test]
fn outbox_owns_supersession_and_publication_state() {
    let mut outbox = Outbox::default();
    let mut timeline = Timeline::empty_at(
        TimelineId::new(3),
        Moment::from_tick(7),
        TimelineLimits::new(1, RetainedBytes::ZERO),
    );
    let mut trace = Trace::new(TraceLimits::new(1, RetainedBytes::ZERO));

    let first = outbox.stage(&mut timeline, 1, Span::from_ticks(3));
    assert!(first == Ok(None));
    let replacement = outbox.stage(&mut timeline, 2, Span::from_ticks(3));
    assert!(replacement == Ok(Some(Publish { revision: 1 })));

    let delivery = timeline.pop_next();
    assert!(delivery.is_some());
    let Some(delivery) = delivery else {
        return;
    };
    let published = *delivery.event();
    outbox.confirm(published);
    assert!(trace.try_push(published).is_ok());

    assert!(outbox.published_revision == Some(2));
    assert!(timeline.now() == Moment::from_tick(10));
    assert!(trace.as_slice() == [Publish { revision: 2 }]);
}

#[test]
fn outbox_replacement_failure_preserves_both_values() {
    let mut outbox = Outbox::default();
    let mut timeline = Timeline::empty_at(
        TimelineId::new(7),
        Moment::from_tick(u64::MAX),
        TimelineLimits::new(1, RetainedBytes::ZERO),
    );
    assert_eq!(outbox.stage(&mut timeline, 1, Span::ZERO), Ok(None));

    let replacement = outbox.stage(&mut timeline, 2, Span::from_ticks(1));
    assert_eq!(
        replacement,
        Err(StageFailure::ReplacementRejected {
            failure: ScheduleFailure::TimeOverflow {
                current: Moment::from_tick(u64::MAX),
                delay: Span::from_ticks(1),
            },
            rejected: Publish { revision: 2 },
            superseded: Some(Publish { revision: 1 }),
        })
    );
    assert!(outbox.pending.is_none());
    assert!(timeline.is_empty());
}

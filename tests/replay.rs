//! Public exact replay position and first-divergence contracts.

use criticality::{
    ByteCount,
    trace::{ExactReplay, ReplayFailure, ReplayPosition, Trace, TraceLimits},
};

#[test]
fn exact_equality_advances_one_record_at_a_time() {
    let expected = [10_u8, 20, 30];
    let mut replay = ExactReplay::new(&expected);
    assert!(replay.position() == ReplayPosition::ORIGIN);
    assert!(replay.expected() == Some(&10));
    assert!(replay.remaining() == 3);

    assert!(replay.observe(&10).is_ok());
    assert!(replay.position() == ReplayPosition::new(1));
    assert!(replay.expected() == Some(&20));
    assert!(replay.observe(&20).is_ok());
    assert!(replay.observe(&30).is_ok());
    assert!(replay.is_complete());
    assert!(replay.finish().is_ok());
}

#[test]
fn mismatch_reports_first_position_without_consuming() {
    let expected = [10_u8, 20];
    let mut replay = ExactReplay::new(&expected);
    assert!(replay.observe(&10).is_ok());
    let failure = replay.observe(&99);
    assert!(
        failure
            == Err(ReplayFailure::Mismatch {
                position: ReplayPosition::new(1)
            })
    );
    assert!(replay.position() == ReplayPosition::new(1));
    assert!(replay.expected() == Some(&20));
    assert!(replay.remaining() == 1);
}

#[test]
fn exhaustion_and_remaining_records_are_explicit() {
    let expected = [4_u8, 5];
    let mut replay = ExactReplay::new(&expected);
    assert!(replay.observe(&4).is_ok());
    assert!(
        replay.finish()
            == Err(ReplayFailure::Remaining {
                position: ReplayPosition::new(1),
                remaining: 1
            })
    );
    assert!(replay.observe(&5).is_ok());
    assert!(
        replay.observe(&6)
            == Err(ReplayFailure::Exhausted {
                position: ReplayPosition::new(2)
            })
    );
    assert!(replay.position() == ReplayPosition::new(2));
}

#[test]
fn trace_replay_borrows_bounded_evidence() {
    let limits = TraceLimits::new(2, ByteCount::ZERO);
    let first = [10_u8; 9];
    let second = [20_u8; 9];
    let mut trace = Trace::with_measure(limits, measure_record);
    assert!(trace.try_push(first).is_ok());
    assert!(trace.try_push(second).is_ok());

    let mut replay = trace.replay();
    assert!(replay.expected() == trace.as_slice().first());
    assert!(replay.observe(&first).is_ok());
    assert!(replay.observe(&second).is_ok());
    assert!(replay.finish().is_ok());
}

const fn measure_record(_: &[u8; 9]) -> ByteCount {
    ByteCount::ZERO
}

//! Public exact FIFO script matching contracts.

use std::vec::Vec;

use bytebudget::{ByteCount, Retained};

use criticality::{
    plan::{Plan, Planned},
    script::{ExactScript, ScriptFailure, ScriptLimits, ScriptPosition, ScriptStep},
    time::Span,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Value {
    id: u8,
    bytes: ByteCount,
}

impl Retained for Value {
    fn retained_bytes(&self) -> ByteCount {
        self.bytes
    }
}

#[test]
fn mismatch_is_non_consuming_and_exhaustion_is_explicit() {
    let steps = Vec::from([ScriptStep::new(
        value(1, 2),
        Plan::single(Planned::new(Span::from_ticks(3), value(2, 4))),
    )]);
    let built = ExactScript::try_new(limits(2, 2, 16), steps);
    assert!(built.is_ok());
    let Ok(mut script) = built else {
        return;
    };
    assert!(script.len() == 1);
    assert!(script.position() == ScriptPosition::ORIGIN);
    assert!(script.expected() == Some(&value(1, 2)));
    assert!(script.retained_bytes() == ByteCount::new(6));

    assert!(
        script.respond(&value(9, 0))
            == Err(ScriptFailure::Mismatch {
                position: ScriptPosition::ORIGIN,
            })
    );
    assert!(script.position() == ScriptPosition::ORIGIN);
    assert!(script.len() == 1);
    assert!(script.retained_bytes() == ByteCount::new(6));

    let response = script.respond(&value(1, 2));
    assert!(response.is_ok());
    let Ok(response) = response else {
        return;
    };
    assert!(response.into_outcomes() == [Planned::new(Span::from_ticks(3), value(2, 4))]);
    assert!(script.is_empty());
    assert!(script.position() == ScriptPosition::new(1));
    assert!(script.retained_bytes() == ByteCount::ZERO);
    assert!(
        script.respond(&value(1, 2))
            == Err(ScriptFailure::Exhausted {
                position: ScriptPosition::new(1),
            })
    );
}

#[test]
fn explicit_measurement_supports_foreign_types() {
    let steps = Vec::from([ScriptStep::new(
        [0_u8; 16],
        Plan::single(Planned::new(Span::ZERO, [1_u8; 16])),
    )]);
    let built =
        ExactScript::try_with_measure(limits(1, 1, 32), steps, measure_array, measure_array);
    assert!(built.is_ok());
    let Ok(script) = built else {
        return;
    };
    assert!(script.retained_bytes() == ByteCount::new(32));
}

const fn value(id: u8, bytes: u64) -> Value {
    Value {
        id,
        bytes: ByteCount::new(bytes),
    }
}

const fn limits(steps: usize, outcomes: usize, bytes: u64) -> ScriptLimits {
    ScriptLimits::new(steps, outcomes, ByteCount::new(bytes))
}

fn measure_array(_: &[u8; 16]) -> ByteCount {
    ByteCount::new(16)
}

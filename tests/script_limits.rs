//! Public exact script construction-limit contracts.

use std::vec::Vec;

use bytebudget::{ByteCount, Retained};

use criticality::{
    plan::{Plan, Planned},
    script::{ExactScript, ScriptBuildFailure, ScriptLimits, ScriptStep},
    time::Span,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Value(ByteCount);

impl Retained for Value {
    fn retained_bytes(&self) -> ByteCount {
        self.0
    }
}

#[test]
fn step_and_outcome_limits_preserve_all_supplied_steps() {
    let step = ScriptStep::new(Value(ByteCount::ZERO), Plan::<Value>::empty());
    let steps = Vec::from([step.clone()]);
    let result = ExactScript::try_new(limits(0, 0, 0), steps);
    assert!(result.is_err());
    let Err(error) = result else {
        return;
    };
    assert!(
        error.failure()
            == ScriptBuildFailure::StepCapacity {
                limit: 0,
                actual: 1
            }
    );
    assert!(error.into_steps() == [step]);

    let step = ScriptStep::new(
        Value(ByteCount::ZERO),
        Plan::single(Planned::new(Span::ZERO, Value(ByteCount::ZERO))),
    );
    let result = ExactScript::try_new(limits(1, 0, 0), Vec::from([step.clone()]));
    assert!(result.is_err());
    let Err(error) = result else {
        return;
    };
    assert!(
        error.failure()
            == ScriptBuildFailure::OutcomeCapacity {
                limit: 0,
                actual: 1
            }
    );
    assert!(error.into_steps() == [step]);
}

#[test]
fn byte_capacity_and_overflow_are_distinct_and_ownership_preserving() {
    let step = ScriptStep::new(
        Value(ByteCount::new(2)),
        Plan::single(Planned::new(Span::ZERO, Value(ByteCount::new(3)))),
    );
    let result = ExactScript::try_new(limits(1, 1, 4), Vec::from([step.clone()]));
    assert!(result.is_err());
    let Err(error) = result else {
        return;
    };
    assert!(is_byte_capacity(error.failure()));
    assert!(error.into_steps() == [step]);

    let step = ScriptStep::new(
        Value(ByteCount::MAX),
        Plan::single(Planned::new(Span::ZERO, Value(ByteCount::new(1)))),
    );
    let result = ExactScript::try_new(limits(1, 1, u64::MAX), Vec::from([step.clone()]));
    assert!(result.is_err());
    let Err(error) = result else {
        return;
    };
    assert!(error.failure() == ScriptBuildFailure::RetainedByteOverflow);
    assert!(error.into_steps() == [step]);
}

const fn limits(steps: usize, outcomes: usize, bytes: u64) -> ScriptLimits {
    ScriptLimits::new(steps, outcomes, ByteCount::new(bytes))
}

const fn is_byte_capacity(failure: ScriptBuildFailure) -> bool {
    let ScriptBuildFailure::RetainedByteCapacity { .. } = failure else {
        return false;
    };
    true
}

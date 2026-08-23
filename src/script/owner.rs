//! Bounded exact FIFO matching with non-consuming mismatch.

use alloc::{collections::VecDeque, vec::Vec};

use crate::{
    plan::Plan,
    retained::{Retained, RetainedBytes},
};

use super::{ScriptBuildError, ScriptBuildFailure, ScriptFailure, ScriptLimits, ScriptStep};

/// Count- and byte-bounded exact finite capability script.
#[derive(Clone, Debug)]
pub struct ExactScript<Q, R> {
    limits: ScriptLimits,
    retained: RetainedBytes,
    steps: VecDeque<MeasuredStep<Q, R>>,
}

impl<Q: Retained, R: Retained> ExactScript<Q, R> {
    /// Retains all supplied steps or returns them unchanged on rejection.
    ///
    /// # Errors
    ///
    /// Returns every step when count or measured-byte admission fails.
    pub fn try_new(
        limits: ScriptLimits,
        steps: Vec<ScriptStep<Q, R>>,
    ) -> Result<Self, ScriptBuildError<Q, R>> {
        Self::try_with_measure(limits, steps, Q::retained_bytes, R::retained_bytes)
    }
}

impl<Q, R> ExactScript<Q, R> {
    /// Builds a script using explicit request and response measurement.
    ///
    /// # Errors
    ///
    /// Returns every step when count or measured-byte admission fails.
    pub fn try_with_measure(
        limits: ScriptLimits,
        steps: Vec<ScriptStep<Q, R>>,
        measure_request: fn(&Q) -> RetainedBytes,
        measure_response: fn(&R) -> RetainedBytes,
    ) -> Result<Self, ScriptBuildError<Q, R>> {
        if steps.len() > limits.steps() {
            let actual = steps.len();
            return Err(ScriptBuildError::new(
                steps,
                ScriptBuildFailure::StepCapacity {
                    limit: limits.steps(),
                    actual,
                },
            ));
        }
        let outcomes = match count_outcomes(&steps) {
            Ok(outcomes) => outcomes,
            Err(failure) => return Err(ScriptBuildError::new(steps, failure)),
        };
        if outcomes > limits.outcomes() {
            return Err(ScriptBuildError::new(
                steps,
                ScriptBuildFailure::OutcomeCapacity {
                    limit: limits.outcomes(),
                    actual: outcomes,
                },
            ));
        }
        let measurements = match measure_steps(&steps, measure_request, measure_response) {
            Ok(measurements) => measurements,
            Err(failure) => return Err(ScriptBuildError::new(steps, failure)),
        };
        let retained = measurements
            .last()
            .map_or(RetainedBytes::ZERO, |measurement| measurement.total);
        if retained > limits.retained_bytes() {
            return Err(ScriptBuildError::new(
                steps,
                ScriptBuildFailure::RetainedByteCapacity {
                    limit: limits.retained_bytes(),
                    actual: retained,
                },
            ));
        }
        let measured = steps
            .into_iter()
            .zip(measurements)
            .map(|(step, measurement)| MeasuredStep {
                step,
                retained: measurement.step,
            })
            .collect();
        Ok(Self {
            limits,
            retained,
            steps: measured,
        })
    }

    /// Returns configured hard ownership limits.
    #[must_use]
    pub const fn limits(&self) -> ScriptLimits {
        self.limits
    }

    /// Returns the remaining exact request count.
    #[must_use]
    pub fn len(&self) -> usize {
        self.steps.len()
    }

    /// Returns whether no scripted request remains.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }

    /// Returns variable bytes retained by all remaining steps and outcomes.
    #[must_use]
    pub const fn retained_bytes(&self) -> RetainedBytes {
        self.retained
    }

    /// Borrows the next exact expected request without consuming it.
    #[must_use]
    pub fn expected(&self) -> Option<&Q> {
        self.steps.front().map(|measured| measured.step.expected())
    }
}

impl<Q: PartialEq, R> ExactScript<Q, R> {
    /// Consumes the next response plan only when `request` matches exactly.
    ///
    /// # Errors
    ///
    /// Returns [`ScriptFailure::Exhausted`] when no step remains and
    /// [`ScriptFailure::Mismatch`] without consuming the next step otherwise.
    pub fn respond(&mut self, request: &Q) -> Result<Plan<R>, ScriptFailure> {
        let Some(next) = self.steps.front() else {
            return Err(ScriptFailure::Exhausted);
        };
        if next.step.expected() != request {
            return Err(ScriptFailure::Mismatch);
        }
        let Some(retained) = self.retained.checked_sub(next.retained) else {
            return Err(ScriptFailure::Exhausted);
        };
        let Some(measured) = self.steps.pop_front() else {
            return Err(ScriptFailure::Exhausted);
        };
        self.retained = retained;
        let (_, response) = measured.step.into_parts();
        Ok(response)
    }
}

#[derive(Clone, Debug)]
struct MeasuredStep<Q, R> {
    step: ScriptStep<Q, R>,
    retained: RetainedBytes,
}

#[derive(Clone, Copy, Debug)]
struct Measurement {
    step: RetainedBytes,
    total: RetainedBytes,
}

fn count_outcomes<Q, R>(steps: &[ScriptStep<Q, R>]) -> Result<usize, ScriptBuildFailure> {
    let mut total = 0usize;
    for step in steps {
        let Some(next) = total.checked_add(step.response().len()) else {
            return Err(ScriptBuildFailure::OutcomeCountOverflow);
        };
        total = next;
    }
    Ok(total)
}

fn measure_steps<Q, R>(
    steps: &[ScriptStep<Q, R>],
    measure_request: fn(&Q) -> RetainedBytes,
    measure_response: fn(&R) -> RetainedBytes,
) -> Result<Vec<Measurement>, ScriptBuildFailure> {
    let mut total = RetainedBytes::ZERO;
    let mut measurements = Vec::with_capacity(steps.len());
    for step in steps {
        let Some(measured) = measure_step(step, measure_request, measure_response) else {
            return Err(ScriptBuildFailure::RetainedByteOverflow);
        };
        let Some(next) = total.checked_add(measured) else {
            return Err(ScriptBuildFailure::RetainedByteOverflow);
        };
        total = next;
        measurements.push(Measurement {
            step: measured,
            total,
        });
    }
    Ok(measurements)
}

fn measure_step<Q, R>(
    step: &ScriptStep<Q, R>,
    measure_request: fn(&Q) -> RetainedBytes,
    measure_response: fn(&R) -> RetainedBytes,
) -> Option<RetainedBytes> {
    let mut total = measure_request(step.expected());
    for outcome in step.response().outcomes() {
        total = total.checked_add(measure_response(outcome.outcome()))?;
    }
    Some(total)
}

//! Ownership-preserving script construction and matching failures.

use alloc::vec::Vec;
use core::fmt;

use crate::retained::RetainedBytes;

use super::ScriptStep;

/// Why an exact finite script rejected its supplied steps.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScriptBuildFailure {
    /// The script contains more steps than its hard count limit.
    StepCapacity {
        /// Configured step-count limit.
        limit: usize,
        /// Supplied step count.
        actual: usize,
    },
    /// The script contains more outcomes than its hard count limit.
    OutcomeCapacity {
        /// Configured outcome-count limit.
        limit: usize,
        /// Supplied outcome count.
        actual: usize,
    },
    /// Response-outcome count accounting overflowed.
    OutcomeCountOverflow,
    /// Variable retained-byte accounting overflowed.
    RetainedByteOverflow,
    /// The script exceeds its variable retained-byte limit.
    RetainedByteCapacity {
        /// Configured retained-byte limit.
        limit: RetainedBytes,
        /// Bytes retained by the supplied script.
        actual: RetainedBytes,
    },
}

impl fmt::Display for ScriptBuildFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::StepCapacity { .. } => formatter.write_str("script step capacity exceeded"),
            Self::OutcomeCapacity { .. } => formatter.write_str("script outcome capacity exceeded"),
            Self::OutcomeCountOverflow => {
                formatter.write_str("script outcome-count accounting overflowed")
            }
            Self::RetainedByteOverflow => {
                formatter.write_str("script retained-byte accounting overflowed")
            }
            Self::RetainedByteCapacity { .. } => {
                formatter.write_str("script retained-byte capacity exceeded")
            }
        }
    }
}

impl core::error::Error for ScriptBuildFailure {}

/// Construction failure retaining ownership of every supplied step.
#[derive(Debug)]
pub struct ScriptBuildError<Q, R> {
    steps: Vec<ScriptStep<Q, R>>,
    failure: ScriptBuildFailure,
}

impl<Q, R> ScriptBuildError<Q, R> {
    pub(super) const fn new(steps: Vec<ScriptStep<Q, R>>, failure: ScriptBuildFailure) -> Self {
        Self { steps, failure }
    }

    /// Returns the construction failure.
    #[must_use]
    pub const fn failure(&self) -> ScriptBuildFailure {
        self.failure
    }

    /// Returns every rejected step in original order.
    #[must_use]
    pub fn into_steps(self) -> Vec<ScriptStep<Q, R>> {
        self.steps
    }
}

impl<Q, R> fmt::Display for ScriptBuildError<Q, R> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.failure.fmt(formatter)
    }
}

impl<Q: fmt::Debug, R: fmt::Debug> core::error::Error for ScriptBuildError<Q, R> {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        Some(&self.failure)
    }
}

/// Why an exact scripted request did not produce a response plan.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScriptFailure {
    /// No scripted request remains.
    Exhausted,
    /// The request does not equal the next expected value.
    Mismatch,
}

impl fmt::Display for ScriptFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Exhausted => formatter.write_str("script is exhausted"),
            Self::Mismatch => formatter.write_str("request does not match the next script step"),
        }
    }
}

impl core::error::Error for ScriptFailure {}

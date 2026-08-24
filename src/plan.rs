//! Finite delayed outcomes for consumer-owned capabilities.

use alloc::{boxed::Box, vec::Vec};

use crate::time::Span;

/// One owned outcome delivered after an abstract delay.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Planned<T> {
    delay: Span,
    outcome: T,
}

impl<T> Planned<T> {
    /// Creates one delayed outcome.
    #[must_use]
    pub const fn new(delay: Span, outcome: T) -> Self {
        Self { delay, outcome }
    }

    /// Returns the relative delivery delay.
    #[must_use]
    pub const fn delay(&self) -> Span {
        self.delay
    }

    /// Borrows the planned outcome.
    #[must_use]
    pub const fn outcome(&self) -> &T {
        &self.outcome
    }

    /// Returns the delay and owned outcome together.
    #[must_use]
    pub fn into_parts(self) -> (Span, T) {
        (self.delay, self.outcome)
    }

    /// Maps the outcome without changing its delay.
    #[must_use]
    pub fn map<U>(self, map: impl FnOnce(T) -> U) -> Planned<U> {
        Planned::new(self.delay, map(self.outcome))
    }
}

/// Finite zero-or-more delayed outcomes for one consumer-owned request.
///
/// Empty plans model drops. Multiple outcomes model duplication, multipart
/// completion, or intentionally reordered delivery through distinct delays.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Plan<T> {
    outcomes: Box<[Planned<T>]>,
}

impl<T> Plan<T> {
    /// Creates a plan and normalizes spare vector capacity into a boxed slice.
    #[must_use]
    pub fn new(outcomes: Vec<Planned<T>>) -> Self {
        Self {
            outcomes: outcomes.into_boxed_slice(),
        }
    }

    /// Creates a dropped-outcome plan.
    #[must_use]
    pub fn empty() -> Self {
        Self::default()
    }

    /// Creates a plan with exactly one delayed outcome.
    #[must_use]
    pub fn single(outcome: Planned<T>) -> Self {
        Self {
            outcomes: Box::new([outcome]),
        }
    }

    /// Returns the finite outcome count.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.outcomes.len()
    }

    /// Returns whether the plan models a dropped operation.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.outcomes.is_empty()
    }

    /// Borrows delayed outcomes in plan order.
    #[must_use]
    pub const fn as_slice(&self) -> &[Planned<T>] {
        &self.outcomes
    }

    /// Iterates over delayed outcomes in plan order.
    #[must_use]
    pub fn iter(&self) -> impl ExactSizeIterator<Item = &Planned<T>> {
        self.outcomes.iter()
    }

    /// Returns delayed outcomes in plan order.
    #[must_use]
    pub fn into_outcomes(self) -> Vec<Planned<T>> {
        self.outcomes.into_vec()
    }
}

impl<T> Default for Plan<T> {
    fn default() -> Self {
        Self {
            outcomes: Box::default(),
        }
    }
}

impl<T> FromIterator<Planned<T>> for Plan<T> {
    fn from_iter<I: IntoIterator<Item = Planned<T>>>(iter: I) -> Self {
        Self::new(iter.into_iter().collect())
    }
}

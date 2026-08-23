//! One exact expected request and its finite response plan.

use crate::plan::Plan;

/// One exact scripted request and its owned response plan.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ScriptStep<Q, R> {
    expected: Q,
    response: Plan<R>,
}

impl<Q, R> ScriptStep<Q, R> {
    /// Creates one exact script step.
    #[must_use]
    pub const fn new(expected: Q, response: Plan<R>) -> Self {
        Self { expected, response }
    }

    /// Borrows the exact expected request.
    #[must_use]
    pub const fn expected(&self) -> &Q {
        &self.expected
    }

    /// Borrows the finite response plan.
    #[must_use]
    pub const fn response(&self) -> &Plan<R> {
        &self.response
    }

    pub(super) fn into_parts(self) -> (Q, Plan<R>) {
        (self.expected, self.response)
    }
}

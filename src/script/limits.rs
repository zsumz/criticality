//! Hard count and retained-memory limits for exact scripts.

use bytebudget::ByteCount;

/// Hard ownership limits for one exact finite script.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ScriptLimits {
    steps: usize,
    outcomes: usize,
    retained_bytes: ByteCount,
}

impl ScriptLimits {
    /// Creates exact script limits.
    #[must_use]
    pub const fn new(steps: usize, outcomes: usize, retained_bytes: ByteCount) -> Self {
        Self {
            steps,
            outcomes,
            retained_bytes,
        }
    }

    /// Returns the maximum retained script-step count.
    #[must_use]
    pub const fn steps(self) -> usize {
        self.steps
    }

    /// Returns the maximum total response-outcome count.
    #[must_use]
    pub const fn outcomes(self) -> usize {
        self.outcomes
    }

    /// Returns the maximum variable retained bytes.
    #[must_use]
    pub const fn retained_bytes(self) -> ByteCount {
        self.retained_bytes
    }
}

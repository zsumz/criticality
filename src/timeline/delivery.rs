//! Owned event deliveries removed from a timeline.

use crate::time::Moment;

use super::EventToken;

/// One event delivered with its exact scheduling identity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Delivery<E, P = ()> {
    token: EventToken<P>,
    event: E,
}

impl<E, P> Delivery<E, P> {
    pub(super) const fn new(token: EventToken<P>, event: E) -> Self {
        Self { token, event }
    }

    /// Returns the delivery moment.
    #[must_use]
    pub const fn at(&self) -> Moment {
        self.token.at()
    }

    /// Borrows the consumer-defined delivery phase.
    #[must_use]
    pub const fn phase(&self) -> &P {
        self.token.phase()
    }

    /// Borrows the delivered event.
    #[must_use]
    pub const fn event(&self) -> &E {
        &self.event
    }

    /// Returns ownership of the delivered event.
    #[must_use]
    pub fn into_event(self) -> E {
        self.event
    }

    /// Returns the scheduling capability and event together.
    #[must_use]
    pub fn into_parts(self) -> (EventToken<P>, E) {
        (self.token, self.event)
    }
}

//! Bounded typed traces and equality-based exact replay.

mod error;
mod limits;
mod owner;
mod replay;

pub use error::{TraceError, TraceFailure};
pub use limits::TraceLimits;
pub use owner::Trace;
pub use replay::{ExactReplay, ReplayFailure, ReplayPosition};

#![no_std]
#![forbid(unsafe_code)]
//! Bounded deterministic simulation primitives for state machines and
//! effect-driven systems.
//!
//! Virtual time, retained-memory accounting, bounded phase-ordered event
//! delivery, finite delayed plans, exact scripts, versioned deterministic
//! entropy, bounded typed traces, and equality-based exact replay are
//! implemented.

extern crate alloc;

pub mod entropy;
pub mod plan;
pub mod retained;
pub mod script;
pub mod time;
pub mod timeline;
pub mod trace;

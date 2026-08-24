<p align="center">
  <img src="./criticality-logo.svg" alt="criticality" width="720">
</p>

<p align="center">
  <strong>Bounded deterministic simulation primitives for state machines and effect-driven systems.</strong>
</p>

<p align="center">
  Criticality provides explicit virtual time, event ordering, finite plans,
  exact scripts, deterministic entropy, and replay without owning your model
  or runtime.
</p>

<p align="center">
  <a href="#model">Model</a>
  <span> · </span>
  <a href="#primitives">Primitives</a>
  <span> · </span>
  <a href="#start">Start</a>
  <span> · </span>
  <a href="#qualification">Qualification</a>
  <span> · </span>
  <a href="#scope">Scope</a>
</p>

## Model

Criticality is a dependency-free `no_std + alloc` library. It gives a state
machine deterministic mechanisms while leaving domain events, transitions,
effects, scheduling policy, fairness, and failure policy with the consumer.

Every retained collection has explicit count and byte limits. Rejected values
return to the caller. Time advances only through checked virtual transitions,
and every replay divergence identifies the first unmatched position.

## Primitives

### Time and retention

`Moment`, `Span`, `Deadline`, and `VirtualClock` define a fixed-width virtual
time domain. `RetainedBytes` and `Retained` make variable memory part of
admission instead of an implicit property of a container.

### Timeline

`Timeline` orders owned events by virtual moment, consumer-defined phase, and
stable insertion identity. Scheduling and cancellation are bounded,
timeline-scoped, and ownership-preserving.

### Plans and scripts

`Plan` describes a finite ordered set of delayed outcomes. `ExactScript`
matches requests without consuming mismatches, reports exhaustion explicitly,
and bounds both step count and retained bytes.

### Entropy

`SplitMix64` provides portable versioned streams and unbiased bounded-index
selection. Independent stream identities keep unrelated draws from changing a
scenario's decisions.

### Traces and replay

`Trace` records typed observations behind count and retained-byte limits.
`ExactReplay` advances only on equality and reports the first divergence,
exhaustion, and remaining records.

## Start

```toml
[dependencies]
criticality = "=0.0.1-rc.1"
```

Schedule an event with explicit limits and virtual time:

```rust
use criticality::{
    retained::RetainedBytes,
    time::Moment,
    timeline::{Timeline, TimelineId, TimelineLimits},
};

let limits = TimelineLimits::new(8, RetainedBytes::ZERO);
let mut timeline = Timeline::<&str>::with_measure(
    TimelineId::new(1),
    limits,
    |_| RetainedBytes::ZERO,
);

assert!(timeline.schedule_at(Moment::from_tick(3), "retry").is_ok());
assert_eq!(timeline.pop_next().map(|item| item.into_event()), Some("retry"));
```

## Qualification

```sh
cargo +1.96.1 install zcheck --version 0.0.1 --locked
zcheck
```

The checked-in `zcheck.toml` is the complete qualification graph. It checks
formatting, the `no_std` library build, Clippy, tests, rustdoc, package contents,
source shape, zrail architecture, and clean diffs. There is no `scripts/check`;
scripts contain implementation logic while declarative commands stay in the
manifest.

Criticality requires Rust 1.88 or newer. zcheck builds with Rust 1.96.1 or
newer. `0.0.1-rc.1` is a release candidate.

## Scope

Criticality does not provide a state-machine framework, async runtime, general
executor, model explorer, protocol vocabulary, or production run loop. It owns
only reusable deterministic mechanisms whose limits and failure paths remain
visible to the caller.

## License

Apache-2.0. See [LICENSE](LICENSE).

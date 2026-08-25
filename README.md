<p align="center">
  <img src="./criticality-logo.svg" alt="criticality" width="720">
</p>

<p align="center">
  <strong>Bounded deterministic simulation primitives for state machines and effect-driven systems.</strong>
</p>

<p align="center">
  Explicit virtual time, event ordering, finite plans, exact scripts,
  deterministic entropy, and replay—without owning your model or runtime.
</p>

<p align="center">
  <a href="#model">Model</a>
  <span> · </span>
  <a href="#start">Start</a>
  <span> · </span>
  <a href="#primitives">Primitives</a>
  <span> · </span>
  <a href="#scope">Scope</a>
</p>

## Model

Criticality is a `no_std + alloc` library of deterministic mechanisms. Your
application keeps ownership of domain events, transitions, effects, scheduling
policy, fairness, and failure policy.

Incrementally admitting containers have explicit count and retained-byte
limits. Rejections preserve ownership, virtual-time transitions are checked,
and replay reports the first divergence.

## Start

```toml
[dependencies]
criticality = "=0.0.1-rc.3"
```

Schedule an event with explicit limits and virtual time:

```rust
use criticality::{
    ByteCount,
    time::Moment,
    timeline::{Timeline, TimelineId, TimelineLimits},
};

let limits = TimelineLimits::new(8, ByteCount::ZERO);
let mut timeline = Timeline::<&str>::with_measure(
    TimelineId::new(1),
    limits,
    |_| ByteCount::ZERO,
);

assert!(timeline.schedule_at(Moment::from_tick(3), "retry").is_ok());
assert_eq!(timeline.pop_next().map(|item| item.into_event()), Some("retry"));
```

Criticality re-exports `ByteCount` and `Retained`, so consumers only name
`criticality`. Its internal byte-budget owner is not part of the facade.

## Primitives

- <a id="time-and-retention"></a>[Time and retention](https://github.com/zsumz/criticality/blob/main/docs/concepts.md#time-and-retention)
- <a id="timeline"></a>[Timeline](https://github.com/zsumz/criticality/blob/main/docs/concepts.md#timeline)
- <a id="plans-and-scripts"></a>[Plans and scripts](https://github.com/zsumz/criticality/blob/main/docs/concepts.md#plans-and-scripts)
- <a id="entropy"></a>[Entropy](https://github.com/zsumz/criticality/blob/main/docs/concepts.md#entropy)
- <a id="traces-and-replay"></a>[Traces and replay](https://github.com/zsumz/criticality/blob/main/docs/concepts.md#traces-and-replay)
- <a id="consumer-evidence"></a>[Consumer composition](https://github.com/zsumz/criticality/blob/main/docs/concepts.md#consumer-composition)

## Qualification

Run `zcheck` for the complete local gate. See
[qualification](https://github.com/zsumz/criticality/blob/main/docs/qualification.md)
for setup, package proof, and toolchain policy.

Criticality requires Rust 1.88 or newer. `0.0.1-rc.3` is a release candidate.

## Scope

Criticality is not a state-machine framework, async runtime, general executor,
model explorer, protocol vocabulary, or production run loop. It owns only the
reusable deterministic mechanisms whose limits and failures remain visible.

## License

Apache-2.0. See [LICENSE](LICENSE).

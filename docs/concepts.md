# Concepts

Criticality supplies bounded, deterministic mechanisms. Consumers retain the
state machine, domain vocabulary, effects, run loop, and policy.

## Boundary

The crate is `no_std + alloc`. It re-exports bytebudget's `ByteCount` and
`Retained` because they appear in Criticality's public vocabulary. Consumers
therefore depend only on `criticality` and import those types from its root.

`ByteBudget` remains internal accounting machinery. Criticality does not expose
it because no public operation asks consumers to manipulate a budget directly.

Incrementally admitting containers enforce both count and retained-byte limits.
Rejected values return to the caller, so failure does not silently discard
ownership. Finite values such as plans remain bounded by their consumer.

## Time and retention

`Moment`, `Span`, `Deadline`, and `VirtualClock` define a fixed-width virtual
time domain. Time advances only through checked transitions.

`ByteCount` and `Retained` make variable memory an explicit part of admission.
Charge-bearing owners deliberately do not implement `Clone`: cloning would
create a new admission whose retained storage must be measured independently.

## Timeline

`Timeline` orders owned events by virtual moment, finite consumer-defined phase,
and stable insertion identity. Phases are `Copy + Ord`; variable retained data
belongs in the measured event.

Scheduling and cancellation are bounded, timeline-scoped, and
ownership-preserving. A `TimelineId` names one incarnation and must not be
reused while capabilities from an earlier incarnation may remain.
`Timeline::empty_at` creates a new empty timeline rather than restoring one.

## Plans and scripts

`Plan` describes a finite, consumer-bounded ordered set of delayed outcomes.
Criticality does not impose a batch-admission policy: consumers can admit
partially, preflight known limits, or roll back admitted events with tokens.

`ExactScript` matches requests without consuming mismatches. It reports
exhaustion and mismatches at exact positions, and bounds both step count and
retained bytes.

## Entropy

`SplitMix64` provides portable versioned streams and unbiased bounded-index
selection. Independent stream identities prevent unrelated draws from changing
a scenario's decisions.

## Traces and replay

`Trace` records typed observations behind count and retained-byte limits. It
keeps exact live accounting without exposing the budget that implements it.

`ExactReplay` borrows finite expected evidence, advances only on equality, and
reports the first divergence, exhaustion, and remaining records. A trace can
lend its already-bounded records to replay without duplicating them.

## Consumer composition

Public persona tests preserve retrying-client, dispatcher, outbox, relay, and
watchdog consumers. Each owns its state, transition policy, effects, and run
loop while composing Criticality's neutral mechanisms.

Those tests are the boundary evidence: reusable determinism belongs here;
domain and orchestration policy stay with the application until repeated
consumers demonstrate a shared requirement.

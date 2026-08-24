//! Public finite delayed-plan contracts.

use std::vec::Vec;

use criticality::{
    plan::{Plan, Planned},
    time::Span,
};

#[test]
fn planned_values_preserve_delay_ownership_and_mapping() {
    let planned = Planned::new(Span::from_ticks(3), String::from("first"));
    assert!(planned.delay() == Span::from_ticks(3));
    assert!(planned.outcome() == "first");

    let mapped = planned.map(|value| value.len());
    assert!(mapped == Planned::new(Span::from_ticks(3), 5));
    assert!(mapped.into_parts() == (Span::from_ticks(3), 5));
}

#[test]
fn finite_plans_preserve_exact_order_and_cardinality() {
    let outcomes = [
        Planned::new(Span::from_ticks(2), 20),
        Planned::new(Span::from_ticks(1), 10),
    ];
    let plan = Plan::new(Vec::from(outcomes.clone()));
    assert!(plan.len() == 2);
    assert!(!plan.is_empty());
    assert!(plan.as_slice() == outcomes);
    assert!(
        plan.iter().map(Planned::delay).collect::<Vec<_>>()
            == [Span::from_ticks(2), Span::from_ticks(1),]
    );
    assert!(plan.clone().into_outcomes() == outcomes);

    assert!(Plan::<u8>::empty().is_empty());
    assert!(Plan::<u8>::default().is_empty());
    let single = Plan::single(Planned::new(Span::ZERO, 7));
    assert!(single.into_outcomes() == [Planned::new(Span::ZERO, 7)]);
}

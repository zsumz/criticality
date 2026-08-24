//! Public virtual-time boundary tests.

use core::{fmt::Debug, hash::Hash};

use criticality::time::{ClockError, Deadline, Moment, Span, VirtualClock};

const CONST_MOMENT: Moment = Moment::from_tick(7);
const CONST_SPAN: Span = Span::from_ticks(5);
const CONST_ADVANCED: Option<Moment> = CONST_MOMENT.checked_add(CONST_SPAN);
const CONST_DEADLINE: Deadline = Deadline::at(Moment::from_tick(12));
const CONST_CLOCK: VirtualClock = VirtualClock::at(CONST_MOMENT);

fn require_structural_value<T: Clone + Copy + Debug + Eq + Hash + Ord>() {}

fn require_default<T: Default>() {}

fn require_error<T: core::error::Error + Send + Sync + 'static>() {}

#[test]
fn time_values_preserve_the_fixed_tick_domain() {
    require_structural_value::<Moment>();
    require_structural_value::<Span>();
    require_structural_value::<Deadline>();
    require_default::<Moment>();
    require_default::<Span>();
    require_default::<VirtualClock>();
    require_error::<ClockError>();

    assert!(CONST_ADVANCED == Some(Moment::from_tick(12)));
    assert!(CONST_DEADLINE.moment() == Moment::from_tick(12));
    assert!(CONST_CLOCK.now() == Moment::from_tick(7));
    assert!(Moment::ORIGIN.tick() == 0);
    assert!(Moment::default() == Moment::ORIGIN);
    assert!(Span::ZERO.ticks() == 0);
    assert!(Span::default() == Span::ZERO);
    assert!(VirtualClock::default() == VirtualClock::new());
    assert!(Moment::from_tick(7).checked_add(Span::from_ticks(5)) == Some(Moment::from_tick(12)));
    assert!(
        Moment::from_tick(u64::MAX)
            .checked_add(Span::from_ticks(1))
            .is_none()
    );
    assert!(
        Span::from_ticks(u64::MAX)
            .checked_add(Span::from_ticks(1))
            .is_none()
    );
}

#[test]
fn deadlines_are_absolute_and_clamp_remaining_time() {
    let deadline = Deadline::at(Moment::from_tick(10));

    assert!(deadline.moment() == Moment::from_tick(10));
    assert!(!deadline.is_elapsed_at(Moment::from_tick(9)));
    assert!(deadline.is_elapsed_at(Moment::from_tick(10)));
    assert!(deadline.is_elapsed_at(Moment::from_tick(11)));
    assert!(deadline.remaining_at(Moment::from_tick(7)) == Span::from_ticks(3));
    assert!(deadline.remaining_at(Moment::from_tick(10)) == Span::ZERO);
    assert!(deadline.remaining_at(Moment::from_tick(11)) == Span::ZERO);
}

#[test]
fn virtual_time_never_moves_backward() {
    let mut clock = VirtualClock::at(Moment::from_tick(10));
    let result = clock.advance_to(Moment::from_tick(9));

    assert!(
        result
            == Err(ClockError::MovesBackward {
                current: Moment::from_tick(10),
                requested: Moment::from_tick(9),
            })
    );
    assert!(clock.now() == Moment::from_tick(10));
    assert!(
        format!(
            "{}",
            ClockError::MovesBackward {
                current: Moment::from_tick(10),
                requested: Moment::from_tick(9),
            }
        ) == "virtual time cannot move backward"
    );
}

#[test]
fn virtual_time_overflow_preserves_the_clock() {
    let mut clock = VirtualClock::at(Moment::from_tick(u64::MAX));
    let result = clock.advance_by(Span::from_ticks(1));

    assert!(
        result
            == Err(ClockError::Overflow {
                current: Moment::from_tick(u64::MAX),
                span: Span::from_ticks(1),
            })
    );
    assert!(clock.now() == Moment::from_tick(u64::MAX));
}

#[test]
fn virtual_time_accepts_equal_and_forward_transitions() {
    let mut clock = VirtualClock::new();

    assert!(clock.now() == Moment::ORIGIN);
    assert!(clock.advance_to(Moment::ORIGIN).is_ok());
    assert!(clock.advance_by(Span::from_ticks(7)).is_ok());
    assert!(clock.now() == Moment::from_tick(7));
}

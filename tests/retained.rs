//! Public retained-memory accounting boundary tests.

use core::{fmt::Debug, hash::Hash};

use criticality::retained::{Retained, RetainedBytes, RetainedBytesOverflow};

const CONST_RETAINED: RetainedBytes = RetainedBytes::new(7);
const CONST_TOTAL: Option<RetainedBytes> = CONST_RETAINED.checked_add(RetainedBytes::new(5));

fn require_structural_value<T: Clone + Copy + Debug + Default + Eq + Hash + Ord>() {}

fn require_error<T: core::error::Error + Send + Sync + 'static>() {}

struct Payload {
    bytes: RetainedBytes,
}

impl Retained for Payload {
    fn retained_bytes(&self) -> RetainedBytes {
        self.bytes
    }
}

#[test]
fn retained_bytes_use_checked_fixed_width_accounting() {
    require_structural_value::<RetainedBytes>();
    require_error::<RetainedBytesOverflow>();

    assert!(CONST_TOTAL == Some(RetainedBytes::new(12)));
    assert!(RetainedBytes::ZERO.get() == 0);
    assert!(RetainedBytes::default() == RetainedBytes::ZERO);
    assert!(RetainedBytes::new(7).get() == 7);
    assert!(
        RetainedBytes::new(7).checked_add(RetainedBytes::new(5)) == Some(RetainedBytes::new(12))
    );
    assert!(
        RetainedBytes::new(u64::MAX)
            .checked_add(RetainedBytes::new(1))
            .is_none()
    );
    assert!(
        RetainedBytes::new(7).checked_sub(RetainedBytes::new(5)) == Some(RetainedBytes::new(2))
    );
    assert!(
        RetainedBytes::new(5)
            .checked_sub(RetainedBytes::new(7))
            .is_none()
    );
}

#[test]
fn retained_bytes_accept_lossless_integer_conversions() {
    assert!(RetainedBytes::from(7_u32) == RetainedBytes::new(7));
    assert!(RetainedBytes::from(9_u64) == RetainedBytes::new(9));
    assert!(RetainedBytes::try_from(11_usize) == Ok(RetainedBytes::new(11)));
}

#[test]
fn retained_measurement_is_consumer_owned_and_repeatable() {
    let payload = Payload {
        bytes: RetainedBytes::new(13),
    };

    assert!(payload.retained_bytes() == RetainedBytes::new(13));
    assert!(payload.retained_bytes() == RetainedBytes::new(13));
    assert!(().retained_bytes() == RetainedBytes::ZERO);
}

#[test]
fn retained_overflow_error_uses_core_error_contract() {
    assert!(
        format!("{RetainedBytesOverflow}")
            == "retained byte count exceeds the u64 accounting domain"
    );
}

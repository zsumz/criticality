//! Public retained-memory accounting boundary tests.

use core::{fmt::Debug, hash::Hash};

use bytebudget::{ByteCount, ByteCountOverflow, Retained};

const CONST_RETAINED: ByteCount = ByteCount::new(7);
const CONST_TOTAL: Option<ByteCount> = CONST_RETAINED.checked_add(ByteCount::new(5));

fn require_structural_value<T: Clone + Copy + Debug + Default + Eq + Hash + Ord>() {}

fn require_error<T: core::error::Error + Send + Sync + 'static>() {}

struct Payload {
    bytes: ByteCount,
}

impl Retained for Payload {
    fn retained_bytes(&self) -> ByteCount {
        self.bytes
    }
}

#[test]
fn byte_count_uses_checked_fixed_width_accounting() {
    require_structural_value::<ByteCount>();
    require_error::<ByteCountOverflow>();

    assert!(CONST_TOTAL == Some(ByteCount::new(12)));
    assert!(ByteCount::ZERO.get() == 0);
    assert!(ByteCount::ZERO.is_zero());
    assert!(ByteCount::MAX.get() == u64::MAX);
    assert!(ByteCount::default() == ByteCount::ZERO);
    assert!(ByteCount::new(7).get() == 7);
    assert!(ByteCount::new(7).checked_add(ByteCount::new(5)) == Some(ByteCount::new(12)));
    assert!(ByteCount::MAX.checked_add(ByteCount::new(1)).is_none());
    assert!(ByteCount::new(7).checked_sub(ByteCount::new(5)) == Some(ByteCount::new(2)));
    assert!(ByteCount::new(5).checked_sub(ByteCount::new(7)).is_none());
}

#[test]
fn byte_count_accepts_lossless_integer_conversions() {
    assert!(ByteCount::from(7_u32) == ByteCount::new(7));
    assert!(ByteCount::from(9_u64) == ByteCount::new(9));
    assert!(u64::from(ByteCount::new(9)) == 9);
    assert!(ByteCount::try_from(11_usize) == Ok(ByteCount::new(11)));
    assert!(usize::try_from(ByteCount::new(11)) == Ok(11));
}

#[test]
fn retained_measurement_is_consumer_owned_and_repeatable() {
    let payload = Payload {
        bytes: ByteCount::new(13),
    };

    assert!(payload.retained_bytes() == ByteCount::new(13));
    assert!(payload.retained_bytes() == ByteCount::new(13));
    assert!(().retained_bytes() == ByteCount::ZERO);
}

#[test]
fn byte_count_overflow_error_uses_core_error_contract() {
    assert!(format!("{ByteCountOverflow}") == "byte count does not fit the target integer type");
}

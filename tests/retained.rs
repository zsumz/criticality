//! Public retained-memory accounting boundary tests.

use criticality::{ByteCount, Retained};

const CONST_RETAINED: ByteCount = ByteCount::new(7);
const CONST_TOTAL: Option<ByteCount> = CONST_RETAINED.checked_add(ByteCount::new(5));

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
    assert!(CONST_TOTAL == Some(ByteCount::new(12)));
    assert!(ByteCount::ZERO.get() == 0);
    assert!(ByteCount::MAX.get() == u64::MAX);
    assert!(ByteCount::MAX.checked_add(ByteCount::new(1)).is_none());
    assert!(ByteCount::new(5).checked_sub(ByteCount::new(7)).is_none());
}

#[test]
fn byte_count_accepts_lossless_integer_conversions() {
    assert!(ByteCount::from(7_u32) == ByteCount::new(7));
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

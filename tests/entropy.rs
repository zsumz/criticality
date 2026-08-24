//! Portable deterministic entropy acceptance vectors.

use criticality::entropy::{EntropySeed, EntropyStreamId, SplitMix64};

fn stream(seed: u64, stream: u64) -> SplitMix64 {
    SplitMix64::new(EntropySeed::new(seed), EntropyStreamId::new(stream))
}

#[test]
fn splitmix64_has_portable_versioned_vectors() {
    let mut entropy = stream(0, 0);
    let actual = [
        entropy.next_u64(),
        entropy.next_u64(),
        entropy.next_u64(),
        entropy.next_u64(),
        entropy.next_u64(),
    ];
    let expected = [
        0x1df5_df97_578d_90c0,
        0xbb0e_8eb9_91d7_d0f7,
        0x274e_2155_3f69_0adc,
        0x0f6f_b523_f192_5196,
        0x10ca_8539_0bfc_4e35,
    ];

    let version = std::hint::black_box(SplitMix64::ALGORITHM_VERSION);
    assert!(version == 1);
    assert!(actual == expected);
}

#[test]
fn independent_streams_are_stable_under_unrelated_draws() {
    let seed = 0x0123_4567_89ab_cdef;
    let mut baseline = stream(seed, 7);
    let first = baseline.next_u64();
    let second = baseline.next_u64();

    let mut replay = stream(seed, 7);
    let mut unrelated = stream(seed, 8);
    assert!(replay.next_u64() == first);
    let _ = unrelated.next_u64();
    let _ = unrelated.next_u64();
    let _ = unrelated.next_u64();
    assert!(replay.next_u64() == second);
    assert!(replay.seed().get() == seed);
    assert!(replay.stream().get() == 7);
}

#[test]
fn bounded_index_mapping_has_portable_versioned_vectors() {
    let mut entropy = stream(0, 0);
    let actual = [
        entropy.next_index(3),
        entropy.next_index(3),
        entropy.next_index(3),
        entropy.next_index(3),
        entropy.next_index(3),
    ];
    let expected = [Ok(0), Ok(2), Ok(0), Ok(0), Ok(0)];

    let version = std::hint::black_box(SplitMix64::BOUNDED_INDEX_ALGORITHM_VERSION);
    assert!(version == 1);
    assert!(actual == expected);
}

#[test]
fn empty_bound_is_explicit_and_does_not_consume_entropy() {
    let mut entropy = stream(91, 4);
    let mut baseline = entropy;

    assert!(entropy.next_index(0).is_err());
    assert!(entropy.next_u64() == baseline.next_u64());
    assert!(entropy.next_index(1) == Ok(0));
}

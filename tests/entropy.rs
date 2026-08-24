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

#[test]
fn edge_bounds_and_forced_rejection_paths_are_exercised() {
    let mut entropy = stream(0, 0);
    let mut baseline = entropy;
    let bound = (1_u64 << 63) + 1;
    let selected = entropy.next_index(bound);
    assert!(selected.is_ok());
    let Ok(selected) = selected else {
        return;
    };
    assert!(selected < bound);

    let mut draws = 0_u8;
    while baseline != entropy && draws < 64 {
        let _ = baseline.next_u64();
        draws += 1;
    }
    assert_eq!(baseline, entropy);
    assert!(draws > 1, "the vector must force at least one rejection");

    assert_eq!(stream(1, 1).next_index(1), Ok(0));
    assert!(matches!(
        stream(1, 1).next_index(u64::MAX),
        Ok(index) if index < u64::MAX
    ));
}

#[test]
fn reduced_width_model_is_exhaustively_uniform() {
    for bound in 1_u8..=u8::MAX {
        let threshold = bound.wrapping_neg() % bound;
        let expected = (256_u16 - u16::from(threshold)) / u16::from(bound);
        let mut counts = [0_u16; 256];

        for draw in 0_u8..=u8::MAX {
            let [low, high] = (u16::from(draw) * u16::from(bound)).to_le_bytes();
            if low >= threshold {
                counts[usize::from(high)] += 1;
            }
        }

        for index in 0_u8..bound {
            assert_eq!(counts[usize::from(index)], expected);
        }
    }
}

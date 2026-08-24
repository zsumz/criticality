//! Versioned deterministic entropy with explicit seeds and independent streams.

use core::error::Error;
use core::fmt;

const GAMMA: u64 = 0x9e37_79b9_7f4a_7c15;
const STREAM_DOMAIN: u64 = 0xd1b5_4a32_d192_ed03;

/// An explicit root seed for a deterministic run.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct EntropySeed(u64);

impl EntropySeed {
    /// Creates a root seed from its portable representation.
    #[must_use]
    pub const fn new(seed: u64) -> Self {
        Self(seed)
    }

    /// Returns the portable representation.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

/// An explicit namespace for one independent entropy stream.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct EntropyStreamId(u64);

impl EntropyStreamId {
    /// Creates a stream identifier from its portable representation.
    #[must_use]
    pub const fn new(stream: u64) -> Self {
        Self(stream)
    }

    /// Returns the portable representation.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

/// A deterministic `SplitMix64` stream with a stable, versioned output contract.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct SplitMix64 {
    seed: EntropySeed,
    stream: EntropyStreamId,
    state: u64,
}

impl SplitMix64 {
    /// The algorithm version governing stream derivation and raw output.
    pub const ALGORITHM_VERSION: u16 = 1;

    /// The algorithm version governing bounded-index selection.
    pub const BOUNDED_INDEX_ALGORITHM_VERSION: u16 = 1;

    /// Creates an independent deterministic stream.
    #[must_use]
    pub fn new(seed: EntropySeed, stream: EntropyStreamId) -> Self {
        let namespace = mix(stream.get().wrapping_add(STREAM_DOMAIN));
        Self {
            seed,
            stream,
            state: seed.get() ^ namespace,
        }
    }

    /// Returns the root seed used to construct this stream.
    #[must_use]
    pub const fn seed(&self) -> EntropySeed {
        self.seed
    }

    /// Returns the stream namespace.
    #[must_use]
    pub const fn stream(&self) -> EntropyStreamId {
        self.stream
    }

    /// Produces the next portable 64-bit output.
    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(GAMMA);
        mix(self.state)
    }

    /// Selects an unbiased index from `0..bound` using rejection sampling.
    ///
    /// An empty bound is rejected without consuming entropy.
    ///
    /// # Errors
    ///
    /// Returns [`BoundedIndexError`] when `bound` is zero.
    pub fn next_index(&mut self, bound: u64) -> Result<u64, BoundedIndexError> {
        if bound == 0 {
            return Err(BoundedIndexError);
        }

        let threshold = bound.wrapping_neg() % bound;
        loop {
            let product = u128::from(self.next_u64()) * u128::from(bound);
            let bytes = product.to_le_bytes();
            let low = u64::from_le_bytes([
                bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
            ]);
            if low >= threshold {
                return Ok(u64::from_le_bytes([
                    bytes[8], bytes[9], bytes[10], bytes[11], bytes[12], bytes[13], bytes[14],
                    bytes[15],
                ]));
            }
        }
    }
}

/// The requested bounded-index domain was empty.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BoundedIndexError;

impl fmt::Display for BoundedIndexError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("bounded-index selection requires a nonzero bound")
    }
}

impl Error for BoundedIndexError {}

fn mix(mut value: u64) -> u64 {
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use primitive_types::H256;
use serde::{Deserialize, Serialize};

/// Simple k-hash Bloom filter backed by a byte vector.
///
/// * `bits` must be a power of two so we can apply a fast bit-mask modulo.
/// * We reuse the 4 u64 words of Keccak256 hash as 4 hash functions.
///   With `bits ≈ 4 × element_count` this yields a false-positive rate ≈ 2 %.
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct BloomFilter {
    bits: Vec<u8>,
    mask: usize,
    k: u8,
}

impl BloomFilter {
    /// Create a Bloom filter of given size. `bits` must be a power of two.
    pub fn new(bits: usize, k: u8) -> Self {
        assert!(bits.is_power_of_two(), "bits must be power of two");
        let bytes = (bits + 7) / 8;
        Self {
            bits: vec![0u8; bytes],
            mask: bits - 1,
            k,
        }
    }

    #[inline]
    fn set_bit(&mut self, idx: usize) {
        let byte = idx >> 3;
        let bit = idx & 7;
        self.bits[byte] |= 1 << bit;
    }

    #[inline]
    fn test_bit(&self, idx: usize) -> bool {
        let byte = idx >> 3;
        let bit = idx & 7;
        (self.bits[byte] & (1 << bit)) != 0
    }

    /// Insert a hash into the Bloom filter.
    pub fn insert(&mut self, hash: &H256) {
        let words: &[u64; 4] =
            unsafe { &*(hash.as_fixed_bytes() as *const [u8; 32] as *const [u64; 4]) };
        for i in 0..self.k {
            let w = words[i as usize % 4];
            let idx = (w as usize) & self.mask;
            self.set_bit(idx);
        }
    }

    /// Query whether a hash may exist (returns false if definitely not present).
    pub fn contains(&self, hash: &H256) -> bool {
        let words: &[u64; 4] =
            unsafe { &*(hash.as_fixed_bytes() as *const [u8; 32] as *const [u64; 4]) };
        for i in 0..self.k {
            let w = words[i as usize % 4];
            let idx = (w as usize) & self.mask;
            if !self.test_bit(idx) {
                return false;
            }
        }
        true
    }

    /// Serialize internal bitmap into bytes.
    pub fn to_bytes(&self) -> Vec<u8> {
        bcs::to_bytes(&self.bits).expect("BloomFilter to bcs should success")
        // bincode::serialize(&self.bits).expect("serialize bloom")
    }

    /// Deserialize from bytes produced by `to_bytes`.
    pub fn from_bytes(bytes: &[u8], k: u8) -> anyhow::Result<Self> {
        // Ok(bcs::from_bytes(bytes)?)
        // let bits_vec: Vec<u8> = bincode::deserialize(bytes)?;
        let bits_vec: Vec<u8> = bcs::from_bytes(bytes)?;
        let bits = bits_vec.len() * 8;
        let mask = bits - 1;
        Ok(Self {
            bits: bits_vec,
            mask,
            k,
        })
    }
}

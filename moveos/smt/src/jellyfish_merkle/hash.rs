// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use bitcoin_hashes::{hex::FromHex, sha256t_hash_newtype, HashEngine};
use bytes::Bytes;
use more_asserts::debug_assert_lt;
use once_cell::sync::Lazy;
use primitive_types::H256;
#[cfg(any(test, feature = "fuzzing"))]
use proptest::prelude::*;
use rand::{rngs::OsRng, Rng};
use std::fmt::{self, Debug};

pub(crate) use bitcoin_hashes::Hash;

sha256t_hash_newtype! {
    pub struct SMTNodeTag = hash_str("SMTNode");


    #[hash_newtype(forward)]
    pub struct SMTNodeHash(_);
}

#[cfg(any(test, feature = "fuzzing"))]
impl Arbitrary for SMTNodeHash {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        any::<[u8; 32]>().prop_map(SMTNodeHash::new).boxed()
    }
}

pub(crate) fn merkle_hash(left: SMTNodeHash, right: SMTNodeHash) -> SMTNodeHash {
    let mut value = left.to_vec();
    value.extend(right.to_vec());
    SMTNodeHash::tag_sha256(&value)
}

impl SMTNodeHash {
    /// The length of the hash in bits.
    pub const LEN_IN_BITS: usize = Self::LEN * 8;

    /// Create a new [`SMTNodeHash`] from a byte array.
    pub fn new(hash: [u8; SMTNodeHash::LEN]) -> Self {
        Self::from_byte_array(hash)
    }

    /// Computes branch hash given two hashes of the nodes underneath it.
    pub fn from_node_hashes(a: SMTNodeHash, b: SMTNodeHash) -> SMTNodeHash {
        Self::combine_node_hashes(a, b)
    }

    /// Computes branch hash given two hashes of the nodes
    fn combine_node_hashes(a: SMTNodeHash, b: SMTNodeHash) -> SMTNodeHash {
        let mut eng = SMTNodeHash::engine();
        eng.input(a.as_ref());
        eng.input(b.as_ref());
        SMTNodeHash::from_engine(eng)
    }

    /// Dumps into a vector.
    pub fn to_vec(self) -> Vec<u8> {
        self.to_byte_array().to_vec()
    }

    /// Creates a zero-initialized instance.
    pub fn zero() -> Self {
        Self::all_zeros()
    }

    /// Create a cryptographically random instance.
    pub fn random() -> Self {
        let mut rng = OsRng;
        let hash: [u8; SMTNodeHash::LEN] = rng.gen();
        SMTNodeHash::new(hash)
    }

    /// Creates a random instance with given rng. Useful in unit tests.
    pub fn random_with_rng<R: Rng>(rng: &mut R) -> Self {
        let hash: [u8; SMTNodeHash::LEN] = rng.gen();
        SMTNodeHash::new(hash)
    }

    /// Creates a new `SMTNodeHash` by tagging the given `data` with `SMTNode`.
    pub fn tag_sha256(data: &[u8]) -> Self {
        SMTNodeHash::hash(data)
    }

    /// Returns the `index`-th bit in the bytes.
    pub fn bit(&self, index: usize) -> bool {
        assert!(index < Self::LEN_IN_BITS);
        let pos = index / 8;
        let bit = 7 - index % 8;
        (self[pos] >> bit) & 1 != 0
    }

    /// Returns the `index`-th nibble in the bytes.
    pub fn nibble(&self, index: usize) -> u8 {
        assert!(index < Self::LEN * 2);
        let pos = index / 2;
        let shift = if index % 2 == 0 { 4 } else { 0 };
        (self[pos] >> shift) & 0x0f
    }

    /// Returns a `NodeHashBitIterator` over all the bits that represent this `SMTNodeHash`.
    pub fn iter_bits(&self) -> NodeHashBitIterator<'_> {
        NodeHashBitIterator::new(self)
    }

    /// Constructs a `SMTNodeHash` from an iterator of bits.
    pub fn from_bit_iter(
        iter: impl ExactSizeIterator<Item = bool>,
    ) -> Result<Self, HashParseError> {
        if iter.len() != Self::LEN_IN_BITS {
            return Err(HashParseError);
        }

        let mut buf = [0; Self::LEN];
        for (i, bit) in iter.enumerate() {
            if bit {
                buf[i / 8] |= 1 << (7 - i % 8);
            }
        }
        Ok(Self::new(buf))
    }

    /// Returns the length of common prefix of `self` and `other` in bits.
    pub fn common_prefix_bits_len(&self, other: SMTNodeHash) -> usize {
        self.iter_bits()
            .zip(other.iter_bits())
            .take_while(|(x, y)| x == y)
            .count()
    }

    /// Full hex representation of a given hash value.
    pub fn as_hex(&self) -> String {
        format!("{:x}", self)
    }

    /// Full hex representation of a given hash value with `0x` prefix.
    pub fn as_hex_literal(&self) -> String {
        format!("{:#x}", self)
    }

    /// Parse a given hex string to a hash value.
    pub fn from_hex(hex: &str) -> Result<Self, HashParseError> {
        <[u8; Self::LEN]>::from_hex(hex)
            .map_err(|_| HashParseError)
            .map(Self::new)
    }

    /// Create a hash value whose contents are just the given integer. Useful for
    /// generating basic mock hash values.
    ///
    /// Ex: SMTNodeHash::from_u64(0x1234) => SMTNodeHash([0, .., 0, 0x12, 0x34])
    #[cfg(any(test, feature = "fuzzing"))]
    pub fn from_u64(v: u64) -> Self {
        let mut hash = [0u8; Self::LEN];
        let bytes = v.to_be_bytes();
        hash[Self::LEN - bytes.len()..].copy_from_slice(&bytes[..]);
        Self::new(hash)
    }

    /// Parse a given hex string to a hash value
    pub fn from_hex_literal(literal: &str) -> Result<Self, HashParseError> {
        if literal.is_empty() {
            return Err(HashParseError);
        }
        let literal = literal.strip_prefix("0x").unwrap_or(literal);
        Self::from_hex(literal)
    }
}

impl Default for SMTNodeHash {
    fn default() -> Self {
        SMTNodeHash::zero()
    }
}

impl From<SMTNodeHash> for Bytes {
    fn from(value: SMTNodeHash) -> Bytes {
        Bytes::copy_from_slice(value.as_ref())
    }
}

impl From<[u8; SMTNodeHash::LEN]> for SMTNodeHash {
    fn from(hash: [u8; SMTNodeHash::LEN]) -> Self {
        SMTNodeHash::new(hash)
    }
}

impl From<SMTNodeHash> for [u8; SMTNodeHash::LEN] {
    fn from(hash_value: SMTNodeHash) -> Self {
        hash_value.to_byte_array()
    }
}

impl From<SMTNodeHash> for H256 {
    fn from(hash_value: SMTNodeHash) -> Self {
        H256(hash_value.to_byte_array())
    }
}

impl From<H256> for SMTNodeHash {
    fn from(h256: H256) -> Self {
        SMTNodeHash::new(h256.0)
    }
}

impl PartialEq<H256> for SMTNodeHash {
    fn eq(&self, other: &H256) -> bool {
        self.to_byte_array() == other.0
    }
}

/// Parse error when attempting to construct a SMTNodeHash
#[derive(Clone, Copy, Debug)]
pub struct HashParseError;

impl fmt::Display for HashParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "unable to parse SMTNodeHash")
    }
}

impl std::error::Error for HashParseError {}

/// An iterator over `SMTNodeHash` that generates one bit for each iteration.
pub struct NodeHashBitIterator<'a> {
    /// The reference to the bytes that represent the `SMTNodeHash`.
    hash_bytes: &'a [u8],
    pos: std::ops::Range<usize>,
    // invariant hash_bytes.len() == SMTNodeHash::LEN;
    // invariant pos.end == hash_bytes.len() * 8;
}

impl<'a> NodeHashBitIterator<'a> {
    /// Constructs a new `NodeHashBitIterator` using given `SMTNodeHash`.
    fn new(hash_value: &'a SMTNodeHash) -> Self {
        NodeHashBitIterator {
            hash_bytes: hash_value.as_ref(),
            pos: (0..SMTNodeHash::LEN_IN_BITS),
        }
    }

    /// Returns the `index`-th bit in the bytes.
    fn get_bit(&self, index: usize) -> bool {
        debug_assert_eq!(self.hash_bytes.len(), SMTNodeHash::LEN); // invariant
        debug_assert_lt!(index, SMTNodeHash::LEN_IN_BITS); // assumed precondition
        let pos = index / 8;
        let bit = 7 - index % 8;
        (self.hash_bytes[pos] >> bit) & 1 != 0
    }
}

impl<'a> std::iter::Iterator for NodeHashBitIterator<'a> {
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        self.pos.next().map(|x| self.get_bit(x))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.pos.size_hint()
    }
}

impl<'a> std::iter::DoubleEndedIterator for NodeHashBitIterator<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.pos.next_back().map(|x| self.get_bit(x))
    }
}

impl<'a> std::iter::ExactSizeIterator for NodeHashBitIterator<'a> {}

/// A type that implements `SMTHash` can be hashed by a cryptographic hash function and produce
/// a `SMTNodeHash`.
pub(crate) trait SMTHash {
    /// Hashes the object and produces a `SMTNodeHash`.
    fn merkle_hash(&self) -> SMTNodeHash;
}

pub(crate) fn create_literal_hash(word: &str) -> SMTNodeHash {
    let mut s = word.as_bytes().to_vec();
    assert!(s.len() <= SMTNodeHash::LEN);
    s.resize(SMTNodeHash::LEN, 0);
    SMTNodeHash::from_slice(&s).expect("create literal hash should success")
}

/// Placeholder hash of `SparseMerkleTree`.
pub(crate) static SPARSE_MERKLE_PLACEHOLDER_HASH_VALUE: Lazy<SMTNodeHash> =
    Lazy::new(|| create_literal_hash("SPARSE_MERKLE_PLACEHOLDER_HASH"));

pub static SPARSE_MERKLE_PLACEHOLDER_HASH: Lazy<H256> =
    Lazy::new(|| (*SPARSE_MERKLE_PLACEHOLDER_HASH_VALUE).into());

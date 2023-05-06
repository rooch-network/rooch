use bytes::Bytes;
use hex::FromHex;
use more_asserts::debug_assert_lt;
use once_cell::sync::Lazy;
#[cfg(any(test, feature = "fuzzing"))]
use proptest_derive::Arbitrary;
use rand::{rngs::OsRng, Rng};
use serde::{de, ser};
use std::{
    fmt::{self, Debug},
    str::FromStr,
};
use tiny_keccak::{Hasher, Sha3};

pub fn merkle_hash(left: HashValue, right: HashValue) -> HashValue {
    let mut value = left.to_vec();
    value.extend(right.to_vec());
    HashValue::sha3_256_of(&value)
}

/// Output value of our hash function. Intentionally opaque for safety and modularity.
#[derive(Clone, Copy, Eq, Hash, PartialEq, PartialOrd, Ord)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
pub struct HashValue {
    hash: [u8; HashValue::LENGTH],
}

impl HashValue {
    /// The length of the hash in bytes.
    pub const LENGTH: usize = 32;
    /// The length of the hash in bits.
    pub const LENGTH_IN_BITS: usize = Self::LENGTH * 8;

    /// Create a new [`HashValue`] from a byte array.
    pub fn new(hash: [u8; HashValue::LENGTH]) -> Self {
        HashValue { hash }
    }

    /// Create from a slice (e.g. retrieved from storage).
    pub fn from_slice<T: AsRef<[u8]>>(bytes: T) -> Result<Self, HashValueParseError> {
        <[u8; Self::LENGTH]>::try_from(bytes.as_ref())
            .map_err(|_| HashValueParseError)
            .map(Self::new)
    }

    /// Dumps into a vector.
    pub fn to_vec(&self) -> Vec<u8> {
        self.hash.to_vec()
    }

    /// Creates a zero-initialized instance.
    pub const fn zero() -> Self {
        HashValue {
            hash: [0; HashValue::LENGTH],
        }
    }

    /// Create a cryptographically random instance.
    pub fn random() -> Self {
        let mut rng = OsRng;
        let hash: [u8; HashValue::LENGTH] = rng.gen();
        HashValue { hash }
    }

    /// Creates a random instance with given rng. Useful in unit tests.
    pub fn random_with_rng<R: Rng>(rng: &mut R) -> Self {
        let hash: [u8; HashValue::LENGTH] = rng.gen();
        HashValue { hash }
    }

    /// Convenience function that computes a `HashValue` internally equal to
    /// the sha3_256 of a byte buffer. It will handle hasher creation, data
    /// feeding and finalization.
    ///
    /// Note this will not result in the `<T as CryptoHash>::hash()` for any
    /// reasonable struct T, as this computes a sha3 without any ornaments.
    pub fn sha3_256_of(buffer: &[u8]) -> Self {
        let mut sha3 = Sha3::v256();
        sha3.update(buffer);
        HashValue::from_keccak(sha3)
    }

    #[cfg(test)]
    pub fn from_iter_sha3<'a, I>(buffers: I) -> Self
    where
        I: IntoIterator<Item = &'a [u8]>,
    {
        let mut sha3 = Sha3::v256();
        for buffer in buffers {
            sha3.update(buffer);
        }
        HashValue::from_keccak(sha3)
    }

    /// Returns the mut reference array
    pub fn as_ref_mut(&mut self) -> &mut [u8] {
        &mut self.hash[..]
    }

    fn from_keccak(state: Sha3) -> Self {
        let mut hash = Self::zero();
        state.finalize(hash.as_ref_mut());
        hash
    }

    /// Returns the `index`-th bit in the bytes.
    pub fn bit(&self, index: usize) -> bool {
        assert!(index < Self::LENGTH_IN_BITS);
        let pos = index / 8;
        let bit = 7 - index % 8;
        (self.hash[pos] >> bit) & 1 != 0
    }

    /// Returns the `index`-th nibble in the bytes.
    pub fn nibble(&self, index: usize) -> u8 {
        assert!(index < Self::LENGTH * 2);
        let pos = index / 2;
        let shift = if index % 2 == 0 { 4 } else { 0 };
        (self.hash[pos] >> shift) & 0x0f
    }

    /// Returns a `HashValueBitIterator` over all the bits that represent this `HashValue`.
    pub fn iter_bits(&self) -> HashValueBitIterator<'_> {
        HashValueBitIterator::new(self)
    }

    /// Constructs a `HashValue` from an iterator of bits.
    pub fn from_bit_iter(
        iter: impl ExactSizeIterator<Item = bool>,
    ) -> Result<Self, HashValueParseError> {
        if iter.len() != Self::LENGTH_IN_BITS {
            return Err(HashValueParseError);
        }

        let mut buf = [0; Self::LENGTH];
        for (i, bit) in iter.enumerate() {
            if bit {
                buf[i / 8] |= 1 << (7 - i % 8);
            }
        }
        Ok(Self::new(buf))
    }

    /// Returns the length of common prefix of `self` and `other` in bits.
    pub fn common_prefix_bits_len(&self, other: HashValue) -> usize {
        self.iter_bits()
            .zip(other.iter_bits())
            .take_while(|(x, y)| x == y)
            .count()
    }

    /// Full hex representation of a given hash value.
    pub fn to_hex(&self) -> String {
        format!("{:x}", self)
    }

    /// Full hex representation of a given hash value with `0x` prefix.
    pub fn to_hex_literal(&self) -> String {
        format!("{:#x}", self)
    }

    /// Parse a given hex string to a hash value.
    pub fn from_hex<T: AsRef<[u8]>>(hex: T) -> Result<Self, HashValueParseError> {
        <[u8; Self::LENGTH]>::from_hex(hex)
            .map_err(|_| HashValueParseError)
            .map(Self::new)
    }

    /// Create a hash value whose contents are just the given integer. Useful for
    /// generating basic mock hash values.
    ///
    /// Ex: HashValue::from_u64(0x1234) => HashValue([0, .., 0, 0x12, 0x34])
    #[cfg(any(test, feature = "fuzzing"))]
    pub fn from_u64(v: u64) -> Self {
        let mut hash = [0u8; Self::LENGTH];
        let bytes = v.to_be_bytes();
        hash[Self::LENGTH - bytes.len()..].copy_from_slice(&bytes[..]);
        Self::new(hash)
    }

    /// Parse a given hex string to a hash value
    pub fn from_hex_literal(literal: &str) -> Result<Self, HashValueParseError> {
        if literal.is_empty() {
            return Err(HashValueParseError);
        }
        let literal = literal.strip_prefix("0x").unwrap_or(literal);
        let hex_len = literal.len();
        // If the string is too short, pad it
        if hex_len < Self::LENGTH * 2 {
            let mut hex_str = String::with_capacity(Self::LENGTH * 2);
            for _ in 0..Self::LENGTH * 2 - hex_len {
                hex_str.push('0');
            }
            hex_str.push_str(literal);
            Self::from_hex(hex_str)
        } else {
            Self::from_hex(&literal)
        }
    }
}

impl ser::Serialize for HashValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        if serializer.is_human_readable() {
            serializer.serialize_str(&self.to_string())
        } else {
            // In order to preserve the Serde data model and help analysis tools,
            // make sure to wrap our value in a container with the same name
            // as the original type.
            serializer
                .serialize_newtype_struct("HashValue", serde_bytes::Bytes::new(&self.hash[..]))
        }
    }
}

impl<'de> de::Deserialize<'de> for HashValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            let encoded_hash = <String>::deserialize(deserializer)?;
            HashValue::from_str(encoded_hash.as_str())
                .map_err(<D::Error as ::serde::de::Error>::custom)
        } else {
            // See comment in serialize.
            #[derive(::serde::Deserialize)]
            #[serde(rename = "HashValue")]
            struct Value<'a>(&'a [u8]);

            let value = Value::deserialize(deserializer)?;
            Self::from_slice(value.0).map_err(<D::Error as ::serde::de::Error>::custom)
        }
    }
}

impl Default for HashValue {
    fn default() -> Self {
        HashValue::zero()
    }
}

impl AsRef<[u8; HashValue::LENGTH]> for HashValue {
    fn as_ref(&self) -> &[u8; HashValue::LENGTH] {
        &self.hash
    }
}

impl std::ops::Deref for HashValue {
    type Target = [u8; Self::LENGTH];

    fn deref(&self) -> &Self::Target {
        &self.hash
    }
}

impl std::ops::Index<usize> for HashValue {
    type Output = u8;

    fn index(&self, s: usize) -> &u8 {
        self.hash.index(s)
    }
}

impl fmt::Binary for HashValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in &self.hash {
            write!(f, "{:08b}", byte)?;
        }
        Ok(())
    }
}

impl fmt::LowerHex for HashValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            write!(f, "0x")?;
        }
        for byte in &self.hash {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}

impl fmt::Debug for HashValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "HashValue({:#x})", self)
    }
}

impl fmt::Display for HashValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:#x}", self)
    }
}

impl From<HashValue> for Bytes {
    fn from(value: HashValue) -> Bytes {
        Bytes::copy_from_slice(value.hash.as_ref())
    }
}

impl FromStr for HashValue {
    type Err = HashValueParseError;

    fn from_str(s: &str) -> Result<Self, HashValueParseError> {
        HashValue::from_hex_literal(s)
    }
}

impl From<[u8; HashValue::LENGTH]> for HashValue {
    fn from(hash: [u8; HashValue::LENGTH]) -> Self {
        HashValue::new(hash)
    }
}

impl From<HashValue> for [u8; HashValue::LENGTH] {
    fn from(hash_value: HashValue) -> Self {
        hash_value.hash
    }
}
/// Parse error when attempting to construct a HashValue
#[derive(Clone, Copy, Debug)]
pub struct HashValueParseError;

impl fmt::Display for HashValueParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "unable to parse HashValue")
    }
}

impl std::error::Error for HashValueParseError {}

/// An iterator over `HashValue` that generates one bit for each iteration.
pub struct HashValueBitIterator<'a> {
    /// The reference to the bytes that represent the `HashValue`.
    hash_bytes: &'a [u8],
    pos: std::ops::Range<usize>,
    // invariant hash_bytes.len() == HashValue::LENGTH;
    // invariant pos.end == hash_bytes.len() * 8;
}

impl<'a> HashValueBitIterator<'a> {
    /// Constructs a new `HashValueBitIterator` using given `HashValue`.
    fn new(hash_value: &'a HashValue) -> Self {
        HashValueBitIterator {
            hash_bytes: hash_value.as_ref(),
            pos: (0..HashValue::LENGTH_IN_BITS),
        }
    }

    /// Returns the `index`-th bit in the bytes.
    fn get_bit(&self, index: usize) -> bool {
        debug_assert_eq!(self.hash_bytes.len(), HashValue::LENGTH); // invariant
        debug_assert_lt!(index, HashValue::LENGTH_IN_BITS); // assumed precondition
        let pos = index / 8;
        let bit = 7 - index % 8;
        (self.hash_bytes[pos] >> bit) & 1 != 0
    }
}

impl<'a> std::iter::Iterator for HashValueBitIterator<'a> {
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        self.pos.next().map(|x| self.get_bit(x))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.pos.size_hint()
    }
}

impl<'a> std::iter::DoubleEndedIterator for HashValueBitIterator<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.pos.next_back().map(|x| self.get_bit(x))
    }
}

impl<'a> std::iter::ExactSizeIterator for HashValueBitIterator<'a> {}

/// A type that implements `SMTHash` can be hashed by a cryptographic hash function and produce
/// a `HashValue`.
pub trait SMTHash {
    /// Hashes the object and produces a `HashValue`.
    fn merkle_hash(&self) -> HashValue;
}

pub fn create_literal_hash(word: &str) -> HashValue {
    let mut s = word.as_bytes().to_vec();
    assert!(s.len() <= HashValue::LENGTH);
    s.resize(HashValue::LENGTH, 0);
    HashValue::from_slice(&s).expect("Cannot fail")
}

/// Placeholder hash of `SparseMerkleTree`.
pub static SPARSE_MERKLE_PLACEHOLDER_HASH: Lazy<HashValue> =
    Lazy::new(|| create_literal_hash("SPARSE_MERKLE_PLACEHOLDER_HASH"));

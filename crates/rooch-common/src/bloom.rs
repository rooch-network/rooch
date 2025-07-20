use primitive_types::H256;
use serde::{Deserialize, Serialize};

/// Simple k-hash Bloom filter backed by a byte vector.
///
/// * `bits` 必须是 2 的幂以便快速取模；
/// * 使用 Keccak256 哈希的 4 个 u64 子段做 4 个哈希函数，误判率约 0.02 当 bits≈4×元素数。
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct BloomFilter {
    bits: Vec<u8>,
    mask: usize,
    k: u8,
}

impl BloomFilter {
    /// 创建一个大小为 `bits` 的 BloomFilter，bits 必须为 2 的幂。
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

    /// 将 `hash` 插入 Bloom 中。
    pub fn insert(&mut self, hash: &H256) {
        let words: &[u64; 4] =
            unsafe { &*(hash.as_fixed_bytes() as *const [u8; 32] as *const [u64; 4]) };
        for i in 0..self.k {
            let w = words[i as usize % 4];
            let idx = (w as usize) & self.mask;
            self.set_bit(idx);
        }
    }

    /// 判断 `hash` 是否可能存在。
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

    /// 序列化为字节。
    pub fn to_bytes(&self) -> Vec<u8> {
        bincode::serialize(&self.bits).expect("serialize bloom")
    }

    /// 从字节反序列化。
    pub fn from_bytes(bytes: &[u8], k: u8) -> anyhow::Result<Self> {
        let bits_vec: Vec<u8> = bincode::deserialize(bytes)?;
        let bits = bits_vec.len() * 8;
        let mask = bits - 1;
        Ok(Self {
            bits: bits_vec,
            mask,
            k,
        })
    }
}

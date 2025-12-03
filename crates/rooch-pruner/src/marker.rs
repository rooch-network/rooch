// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use moveos_common::bloom_filter::BloomFilter;
use moveos_types::h256::H256;
use parking_lot::Mutex;
use std::sync::atomic::{AtomicU64, AtomicU8, Ordering};
use std::sync::Arc;

/// Calculate optimal Bloom filter parameters for given node count and target false positive rate
///
/// Returns (bit_count, hash_functions) tuple
fn optimal_bloom_size(estimated_nodes: usize, target_fp_rate: f64) -> (usize, u8) {
    // Optimal bits per element: m/n = -ln(ε) / ln(2)^2
    // where ε is target false positive rate
    let bits_per_element = -(target_fp_rate.ln()) / (2.0_f64.ln().powi(2));

    // Optimal hash functions: k = -ln(ε) / ln(2)
    let optimal_hash_fns = (-target_fp_rate.ln() / 2.0_f64.ln()).ceil() as u8;

    // Calculate bit count, ensure minimum values
    let mut bit_count = ((estimated_nodes as f64) * bits_per_element).ceil() as usize;
    bit_count = bit_count.max(1024); // Minimum 1KB

    // Ensure bit_count is power of 2 for fast modulo operations
    let bit_count = bit_count.next_power_of_two();

    // Clamp hash functions to reasonable range
    let hash_fns = optimal_hash_fns.clamp(1, 16);

    // Validate hash functions are in reasonable range
    assert!((1..=16).contains(&hash_fns));

    (bit_count, hash_fns)
}

/// Trait for marking reachable nodes during garbage collection
///
/// This trait provides a unified interface for different marking strategies
/// while allowing the GC algorithm to work with any underlying storage mechanism.
pub trait NodeMarker: Send + Sync {
    /// Mark a node as reachable during the mark phase
    ///
    /// Returns whether this was the first time the node was marked
    fn mark(&self, node_hash: H256) -> Result<bool>;

    /// Check if a node has been marked (for deduplication)
    fn is_marked(&self, node_hash: &H256) -> bool;

    /// Get the total count of marked nodes
    fn marked_count(&self) -> u64;

    /// Reset the marker state for a new GC run
    fn reset(&self) -> Result<()>;

    /// Get the marker type for reporting
    fn marker_type(&self) -> &'static str;
}

/// Atomic Bloom filter-based marker for GC operations
///
/// Uses atomic operations to avoid lock contention during parallel GC.
/// False positives are safe in GC context (only retain extra nodes).
pub struct AtomicBloomFilterMarker {
    bits: Vec<AtomicU8>,
    mask: usize,
    k: u8,
    counter: Arc<AtomicU64>,
    bloom_bits: usize,
    bloom_hash_fns: u8,
}

impl AtomicBloomFilterMarker {
    /// Create a new AtomicBloomFilterMarker with specified parameters
    pub fn new(bloom_bits: usize, bloom_hash_fns: u8) -> Self {
        // Calculate bytes needed for the bloom filter
        let bytes = (bloom_bits + 7) / 8;
        let bits = (0..bytes)
            .map(|_| AtomicU8::new(0))
            .collect();

        Self {
            bits,
            mask: bloom_bits - 1,
            k: bloom_hash_fns,
            counter: Arc::new(AtomicU64::new(0)),
            bloom_bits,
            bloom_hash_fns,
        }
    }

    /// Create an AtomicBloomFilterMarker with optimal parameters for estimated node count and target false positive rate
    pub fn with_estimated_nodes(estimated_nodes: usize, target_fp_rate: f64) -> Self {
        let (bloom_bits, bloom_hash_fns) = optimal_bloom_size(estimated_nodes, target_fp_rate);
        Self::new(bloom_bits, bloom_hash_fns)
    }

    /// Get bloom filter statistics for monitoring
    pub fn bloom_stats(&self) -> (usize, u8) {
        (self.bloom_bits, self.bloom_hash_fns)
    }

    /// Estimate current false positive rate
    pub fn estimated_false_positive_rate(&self) -> f64 {
        let marked_count = self.marked_count();
        if marked_count == 0 {
            return 0.0;
        }

        // False positive rate formula: (1 - e^(-k*n/m))^k
        // where n = items inserted, m = bit count, k = hash functions
        let n = marked_count as f64;
        let m = self.bloom_bits as f64;
        let k = self.bloom_hash_fns as f64;

        let fp_rate = (1.0 - (-k * n / m).exp()).powi(k as i32);
        fp_rate.min(1.0) // Cap at 100%
    }

    /// Set a bit atomically using fetch_or
    #[inline]
    fn set_bit(&self, idx: usize) {
        let byte = idx >> 3;
        let bit = 1u8 << (idx & 7);
        self.bits[byte].fetch_or(bit, Ordering::Relaxed);
    }

    /// Test a bit atomically
    #[inline]
    fn test_bit(&self, idx: usize) -> bool {
        let byte = idx >> 3;
        let bit = 1u8 << (idx & 7);
        (self.bits[byte].load(Ordering::Relaxed) & bit) != 0
    }

    /// Insert a hash into the atomic Bloom filter
    pub fn insert(&self, hash: &H256) {
        let words: &[u64; 4] =
            unsafe { &*(hash.as_fixed_bytes() as *const [u8; 32] as *const [u64; 4]) };
        for i in 0..self.k {
            let w = words[i as usize % 4];
            let idx = (w as usize) & self.mask;
            self.set_bit(idx);
        }
    }

    /// Query whether a hash may exist (returns false if definitely not present)
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
}

impl NodeMarker for AtomicBloomFilterMarker {
    fn mark(&self, node_hash: H256) -> Result<bool> {
        // For Bloom Filter, we always attempt to mark and return whether it was newly inserted
        // We can't reliably check if it was already marked due to false positives
        // So we maintain a separate counter and always increment it for each mark call
        // This means mark() will always return true for Bloom Filter implementation

        self.insert(&node_hash);

        // Increment counter for every mark call
        self.counter.fetch_add(1, Ordering::Relaxed);

        // Bloom Filter always returns true since we can't detect duplicates reliably
        Ok(true)
    }

    fn is_marked(&self, node_hash: &H256) -> bool {
        self.contains(node_hash) // May have false positives, but safe for GC
    }

    fn marked_count(&self) -> u64 {
        self.counter.load(Ordering::Relaxed)
    }

    fn reset(&self) -> Result<()> {
        // Reset all bits to 0
        for atomic_byte in &self.bits {
            atomic_byte.store(0, Ordering::Relaxed);
        }

        // Reset counter
        self.counter.store(0, Ordering::Relaxed);

        Ok(())
    }

    fn marker_type(&self) -> &'static str {
        "AtomicBloomFilter"
    }
}

/// Bloom filter-based marker for GC operations
///
/// Uses a Bloom filter for memory-efficient node marking.
/// False positives are safe in GC context (only retain extra nodes).
pub struct BloomFilterMarker {
    bloom_filter: Arc<Mutex<BloomFilter>>,
    counter: Arc<AtomicU64>,
    bloom_bits: usize,
    bloom_hash_fns: u8,
}

impl BloomFilterMarker {
    /// Create a new BloomFilterMarker with specified parameters
    pub fn new(bloom_bits: usize, bloom_hash_fns: u8) -> Self {
        let bloom_filter = BloomFilter::new(bloom_bits, bloom_hash_fns);

        Self {
            bloom_filter: Arc::new(Mutex::new(bloom_filter)),
            counter: Arc::new(AtomicU64::new(0)),
            bloom_bits,
            bloom_hash_fns,
        }
    }

    /// Create a BloomFilterMarker with optimal parameters for estimated node count and target false positive rate
    pub fn with_estimated_nodes(estimated_nodes: usize, target_fp_rate: f64) -> Self {
        let (bloom_bits, bloom_hash_fns) = optimal_bloom_size(estimated_nodes, target_fp_rate);
        Self::new(bloom_bits, bloom_hash_fns)
    }

    /// Get bloom filter statistics for monitoring
    pub fn bloom_stats(&self) -> (usize, u8) {
        (self.bloom_bits, self.bloom_hash_fns)
    }

    /// Estimate current false positive rate
    pub fn estimated_false_positive_rate(&self) -> f64 {
        let marked_count = self.marked_count();
        if marked_count == 0 {
            return 0.0;
        }

        // False positive rate formula: (1 - e^(-k*n/m))^k
        // where n = items inserted, m = bit count, k = hash functions
        let n = marked_count as f64;
        let m = self.bloom_bits as f64;
        let k = self.bloom_hash_fns as f64;

        let fp_rate = (1.0 - (-k * n / m).exp()).powi(k as i32);
        fp_rate.min(1.0) // Cap at 100%
    }
}

impl NodeMarker for BloomFilterMarker {
    fn mark(&self, node_hash: H256) -> Result<bool> {
        // For Bloom Filter, we always attempt to mark and return whether it was newly inserted
        // We can't reliably check if it was already marked due to false positives
        // So we maintain a separate counter and always increment it for each mark call
        // This means mark() will always return true for Bloom Filter implementation

        let mut bloom = self.bloom_filter.lock();
        bloom.insert(&node_hash);
        drop(bloom);

        // Increment counter for every mark call
        self.counter.fetch_add(1, Ordering::Relaxed);

        // Bloom Filter always returns true since we can't detect duplicates reliably
        Ok(true)
    }

    fn is_marked(&self, node_hash: &H256) -> bool {
        let bloom = self.bloom_filter.lock();
        bloom.contains(node_hash) // May have false positives, but safe for GC
    }

    fn marked_count(&self) -> u64 {
        self.counter.load(Ordering::Relaxed)
    }

    fn reset(&self) -> Result<()> {
        // Reset bloom filter to new instance
        let mut bloom = self.bloom_filter.lock();
        *bloom = BloomFilter::new(self.bloom_bits, self.bloom_hash_fns);
        drop(bloom);

        // Reset counter
        self.counter.store(0, Ordering::Relaxed);

        Ok(())
    }

    fn marker_type(&self) -> &'static str {
        "BloomFilter"
    }
}

/// Create a marker instance with optimal parameters for the estimated node count
pub fn create_marker(estimated_nodes: usize, target_fp_rate: f64) -> Box<dyn NodeMarker> {
    Box::new(AtomicBloomFilterMarker::with_estimated_nodes(
        estimated_nodes,
        target_fp_rate,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optimal_bloom_size() {
        // Test various scenarios
        let (bits, fns) = optimal_bloom_size(1000, 0.01);
        assert!(bits >= 1024); // Minimum 1KB
        assert!(bits.is_power_of_two()); // Should be power of 2
        assert!((1..=16).contains(&fns)); // Reasonable hash function range

        // Larger dataset should use more bits
        let (bits_large, _) = optimal_bloom_size(100_000, 0.01);
        assert!(bits_large > bits);

        // Stricter FP rate should use more bits/hash functions
        let (bits_strict, fns_strict) = optimal_bloom_size(1000, 0.001);
        assert!(bits_strict >= bits);
        assert!(fns_strict >= fns);
    }

    #[test]
    fn test_bloom_filter_marker_basic() {
        let marker = BloomFilterMarker::new(1024, 4);
        let hash1 = H256::random();
        let hash2 = H256::random();

        // Test initial state
        assert!(!marker.is_marked(&hash1));
        assert_eq!(marker.marked_count(), 0);

        // Test marking
        assert!(marker.mark(hash1).unwrap());
        assert!(marker.is_marked(&hash1));
        assert!(!marker.is_marked(&hash2));
        assert_eq!(marker.marked_count(), 1);

        // Test duplicate marking (bloom filter may have false positives)
        let _was_newly_marked = marker.mark(hash1).unwrap();
        // Should be false (already marked), but bloom filter can have false positives
        // We just ensure it doesn't panic

        // Test reset
        marker.reset().unwrap();
        assert!(!marker.is_marked(&hash1));
        assert_eq!(marker.marked_count(), 0);
    }

    #[test]
    fn test_bloom_filter_marker_with_estimated_nodes() {
        let marker = BloomFilterMarker::with_estimated_nodes(100_000, 0.01);
        let hash1 = H256::random();
        let hash2 = H256::random();

        // Test basic operations
        assert!(marker.mark(hash1).unwrap());
        assert!(marker.is_marked(&hash1));
        assert!(!marker.is_marked(&hash2));
        assert_eq!(marker.marked_count(), 1);

        // Test false positive rate estimation
        let fp_rate = marker.estimated_false_positive_rate();
        assert!((0.0..=1.0).contains(&fp_rate));

        // Test reset
        marker.reset().unwrap();
        assert_eq!(marker.marked_count(), 0);
    }

    #[test]
    fn test_false_positive_rate_estimation() {
        let marker = BloomFilterMarker::new(1024, 4);

        // Empty marker should have 0 FP rate
        assert_eq!(marker.estimated_false_positive_rate(), 0.0);

        // Mark some nodes
        for _ in 0..10 {
            let hash = H256::random();
            marker.mark(hash).unwrap();
        }

        // Should have some estimated FP rate
        let fp_rate = marker.estimated_false_positive_rate();
        assert!(fp_rate > 0.0 && fp_rate <= 1.0);

        // Test that FP rate doesn't exceed 100%
        assert!(fp_rate <= 1.0);
    }

    #[test]
    fn test_create_marker() {
        let marker = create_marker(100_000, 0.01);
        assert_eq!(marker.marker_type(), "AtomicBloomFilter");

        // Test basic operations
        let hash1 = H256::random();
        assert!(marker.mark(hash1).unwrap());
        assert_eq!(marker.marked_count(), 1);
    }

    #[test]
    fn test_atomic_bloom_filter_marker_basic() {
        let marker = AtomicBloomFilterMarker::new(1024, 4);
        let hash1 = H256::random();
        let hash2 = H256::random();

        // Test initial state
        assert!(!marker.is_marked(&hash1));
        assert_eq!(marker.marked_count(), 0);

        // Test marking
        assert!(marker.mark(hash1).unwrap());
        assert!(marker.is_marked(&hash1));
        assert!(!marker.is_marked(&hash2));
        assert_eq!(marker.marked_count(), 1);

        // Test duplicate marking (bloom filter may have false positives)
        let _was_newly_marked = marker.mark(hash1).unwrap();
        // Should be false (already marked), but bloom filter can have false positives
        // We just ensure it doesn't panic

        // Test reset
        marker.reset().unwrap();
        assert!(!marker.is_marked(&hash1));
        assert_eq!(marker.marked_count(), 0);
    }

    #[test]
    fn test_atomic_bloom_filter_marker_with_estimated_nodes() {
        let marker = AtomicBloomFilterMarker::with_estimated_nodes(100_000, 0.01);
        let hash1 = H256::random();
        let hash2 = H256::random();

        // Test basic operations
        assert!(marker.mark(hash1).unwrap());
        assert!(marker.is_marked(&hash1));
        assert!(!marker.is_marked(&hash2));
        assert_eq!(marker.marked_count(), 1);

        // Test false positive rate estimation
        let fp_rate = marker.estimated_false_positive_rate();
        assert!((0.0..=1.0).contains(&fp_rate));

        // Test reset
        marker.reset().unwrap();
        assert_eq!(marker.marked_count(), 0);
    }

    #[test]
    fn test_atomic_bloom_filter_false_positive_rate_estimation() {
        let marker = AtomicBloomFilterMarker::new(1024, 4);

        // Empty marker should have 0 FP rate
        assert_eq!(marker.estimated_false_positive_rate(), 0.0);

        // Mark some nodes
        for _ in 0..10 {
            let hash = H256::random();
            marker.mark(hash).unwrap();
        }

        // Should have some estimated FP rate
        let fp_rate = marker.estimated_false_positive_rate();
        assert!(fp_rate > 0.0 && fp_rate <= 1.0);

        // Test that FP rate doesn't exceed 100%
        assert!(fp_rate <= 1.0);
    }
}

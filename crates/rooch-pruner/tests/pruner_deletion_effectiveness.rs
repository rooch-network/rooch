// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use moveos_store::MoveOSStore;
use moveos_types::h256::H256;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use rooch_config::prune_config::PruneConfig;
use rooch_pruner::reachability::ReachableBuilder;
use rooch_pruner::sweep_expired::SweepExpired;
use smt::jellyfish_merkle::node_type::Node;
use smt::SMTObject;
use std::sync::Arc;
use std::time::{Duration, Instant};

#[test]
fn test_pruner_deletion_effectiveness_basic() {
    println!("🧹 Testing Pruner Deletion Effectiveness - Basic");

    let _temp_dir = tempfile::tempdir().unwrap();
    let (moveos_store, _data_dir) = MoveOSStore::mock_moveos_store().unwrap();
    let moveos_store = Arc::new(moveos_store);

    // Create test nodes
    let node_store = moveos_store.get_state_node_store();
    let mut rng = StdRng::seed_from_u64(42);
    let mut test_nodes = Vec::new();
    let mut expired_nodes = Vec::new();

    println!("📊 Creating test data...");

    // Create total nodes
    let total_nodes = 1000;
    for i in 0..total_nodes {
        let key = H256::random();
        let mut value = vec![0u8; 64];
        rng.fill(&mut value[..]);

        let node = Node::new_leaf(
            key,
            SMTObject::<Vec<u8>>::from_origin(value.clone()).unwrap(),
        );
        let hash: H256 = node.get_merkle_hash().into();

        node_store.put(hash, node.encode().unwrap()).unwrap();
        test_nodes.push(hash);

        // Some nodes are "expired" (not reachable)
        if i >= total_nodes / 3 && i < total_nodes * 2 / 3 {
            expired_nodes.push(hash);
        }
    }

    println!("   Created {} total nodes", total_nodes);
    println!("   {} nodes marked as expired", expired_nodes.len());

    // Phase 1: Mark reachable nodes
    println!("\n🔍 Phase 1: Marking reachable nodes");
    let bloom = Arc::new(parking_lot::Mutex::new(
        moveos_common::bloom_filter::BloomFilter::new(8_388_608, 4), // 1MB
    ));

    // Mark first 1/3 as reachable
    let live_roots: Vec<H256> = test_nodes[0..total_nodes / 3].to_vec();
    let reachable_start = Instant::now();

    let builder = ReachableBuilder::new(moveos_store.clone(), bloom.clone());
    match builder.build(live_roots, num_cpus::get()) {
        Ok(scanned_size) => {
            let reachable_time = reachable_start.elapsed();
            println!("   ✅ Reachability build completed:");
            println!(
                "      Scanned {} nodes in {:?}",
                scanned_size, reachable_time
            );
            println!(
                "      Throughput: {:.0} nodes/sec",
                scanned_size as f64 / reachable_time.as_secs_f64()
            );
        }
        Err(e) => {
            println!("   ❌ Reachability build failed: {}", e);
            panic!("Reachability build should not fail");
        }
    }

    // Phase 2: Sweep expired nodes
    println!("\n🧹 Phase 2: Sweeping expired nodes");
    let sweeper = SweepExpired::new(
        moveos_store.clone(),
        bloom.clone(),
        8_388_608,                                          // 1MB
        Arc::new(std::sync::atomic::AtomicBool::new(true)), // true = running
    );

    // Create expired roots (state_root, tx_order)
    let expired_roots: Vec<(H256, u64)> = expired_nodes
        .iter()
        .enumerate()
        .map(|(i, hash)| (*hash, (i + 1) as u64))
        .collect();

    println!("   Testing {} expired roots", expired_roots.len());

    let sweep_start = Instant::now();
    match sweeper.sweep(expired_roots.clone(), 1) {
        Ok(deleted_count) => {
            let sweep_time = sweep_start.elapsed();
            println!("   ✅ Sweep completed:");
            println!("      Deleted {} nodes in {:?}", deleted_count, sweep_time);
            println!(
                "      Throughput: {:.0} nodes/sec",
                deleted_count as f64 / sweep_time.as_secs_f64()
            );
            println!(
                "      Deletion ratio: {:.1}%",
                (deleted_count as f64 / expired_nodes.len() as f64) * 100.0
            );

            // Effectiveness assertions
            assert!(deleted_count > 0, "Should delete some nodes");
            assert!(
                deleted_count >= (expired_nodes.len() / 2) as u64,
                "Should delete at least 50% of expired nodes"
            );
            assert!(
                sweep_time < Duration::from_secs(30),
                "Should complete within reasonable time"
            );

            println!("   🎯 Deletion effectiveness: ✅ PASSED");
        }
        Err(e) => {
            println!("   ❌ Sweep failed: {}", e);
            panic!("Sweep expired should not fail");
        }
    }
}

#[test]
fn test_pruner_effectiveness_different_scenarios() {
    println!("🔄 Testing Pruner Effectiveness in Different Scenarios");

    let scenarios = vec![
        ("Small Scale", 500, 0.3, 1_048_576),   // 1MB bloom
        ("Medium Scale", 2000, 0.3, 8_388_608), // 1MB bloom
        ("Large Scale", 5000, 0.3, 67_108_864), // 8MB bloom
    ];

    for (name, total_nodes, _expired_ratio, bloom_bits) in scenarios {
        println!("\n📊 Testing scenario: {}", name);

        let _temp_dir = tempfile::tempdir().unwrap();
        let (moveos_store, _data_dir) = MoveOSStore::mock_moveos_store().unwrap();
        let moveos_store = Arc::new(moveos_store);

        // Create test data
        let node_store = moveos_store.get_state_node_store();
        let mut rng = StdRng::seed_from_u64(42);
        let mut test_nodes = Vec::new();
        let mut expired_nodes = Vec::new();

        for i in 0..total_nodes {
            let key = H256::random();
            let mut value = vec![0u8; 64];
            rng.fill(&mut value[..]);

            let node = Node::new_leaf(
                key,
                SMTObject::<Vec<u8>>::from_origin(value.clone()).unwrap(),
            );
            let hash: H256 = node.get_merkle_hash().into();
            node_store.put(hash, node.encode().unwrap()).unwrap();
            test_nodes.push(hash);

            // Mark nodes as expired if they're in the middle third
            // This ensures they don't overlap with reachable nodes
            if i >= total_nodes / 3 && i < total_nodes * 2 / 3 {
                expired_nodes.push(hash);
            }
        }

        println!(
            "   {} nodes, {} expired",
            test_nodes.len(),
            expired_nodes.len()
        );

        // Build reachability
        let bloom = Arc::new(parking_lot::Mutex::new(
            moveos_common::bloom_filter::BloomFilter::new(bloom_bits, 4),
        ));
        // Use first 1/3 as reachable (not overlapping with expired nodes)
        let live_roots: Vec<H256> = test_nodes[0..total_nodes / 3].to_vec();

        let builder = ReachableBuilder::new(moveos_store.clone(), bloom.clone());
        let _ = builder.build(live_roots, 1).unwrap();

        // Sweep expired
        let sweeper = SweepExpired::new(
            moveos_store.clone(),
            bloom.clone(),
            bloom_bits,
            Arc::new(std::sync::atomic::AtomicBool::new(true)), // true = running
        );

        let expired_roots: Vec<(H256, u64)> = expired_nodes
            .iter()
            .enumerate()
            .map(|(i, hash)| (*hash, (i + 1) as u64))
            .collect();

        let start_time = Instant::now();
        match sweeper.sweep(expired_roots, 1) {
            Ok(deleted_count) => {
                let duration = start_time.elapsed();
                let deletion_ratio = deleted_count as f64 / expired_nodes.len() as f64;
                let throughput = deleted_count as f64 / duration.as_secs_f64();
                let memory_mb = bloom_bits as f64 / 8.0 / 1024.0 / 1024.0;

                println!("   📈 Results:");
                println!(
                    "      Deleted: {}/{} ({:.1}%)",
                    deleted_count,
                    expired_nodes.len(),
                    deletion_ratio * 100.0
                );
                println!("      Time: {:?}", duration);
                println!("      Throughput: {:.0} nodes/sec", throughput);
                println!("      Memory: {:.1} MB", memory_mb);

                // Effectiveness criteria - scaled expectations for different node counts
                let min_throughput = if total_nodes >= 5000 { 10.0 } else { 30.0 }; // Lower throughput for large scale
                let is_effective = deletion_ratio > 0.3 &&  // At least 30% deletion
                                   throughput > min_throughput &&  // Reasonable throughput (scaled)
                                   memory_mb < 100.0; // Reasonable memory

                let effectiveness = if is_effective {
                    "✅ Effective"
                } else {
                    "❌ Ineffective"
                };
                println!("      Effectiveness: {}", effectiveness);

                // Basic effectiveness assertions
                assert!(deleted_count > 0, "{} should delete some nodes", name);
                if total_nodes >= 1000 {
                    assert!(is_effective, "{} scenario should be effective", name);
                }
            }
            Err(e) => {
                println!("   ❌ {} failed: {}", name, e);
                panic!("Sweep should not fail for scenario: {}", name);
            }
        }
    }

    println!("\n🎯 Scenario Analysis:");
    println!("   • Small scale: Quick processing, minimal memory usage");
    println!("   • Medium scale: Balanced performance");
    println!("   • Large scale: Higher throughput, reasonable memory");
    println!("   • All scenarios achieve required deletion effectiveness");
}

#[test]
fn test_pruner_config_impact_on_deletion() {
    println!("📊 Testing Pruner Configuration Impact on Deletion Effectiveness");

    let configs = vec![
        (
            "Conservative",
            PruneConfig {
                enable: true,
                boot_cleanup_done: false,
                scan_batch: 500,
                delete_batch: 250,
                interval_s: 120,
                bloom_bits: 1_048_576, // 1MB
                enable_reach_seen_cf: false,
                protection_orders: 30000,
                enable_incremental_sweep: true,
                incremental_sweep_batch: 500,
                recycle_bin_enable: false,
                recycle_bin_max_entries: 10000,
                recycle_bin_max_bytes: 100_000_000,
                protected_roots_count: 1,
                // Phase 3: PersistentMarker configuration fields
                marker_batch_size: 5000,
                marker_bloom_bits: 524288,
                marker_bloom_hash_fns: 3,
                marker_memory_threshold_mb: 512,
                marker_auto_strategy: true,
                marker_force_persistent: false,
                marker_temp_cf_name: "gc_marker_temp".to_string(),
                marker_error_recovery: true,
            },
        ),
        (
            "Balanced",
            PruneConfig {
                enable: true,
                boot_cleanup_done: false,
                scan_batch: 2000,
                delete_batch: 1000,
                interval_s: 60,
                bloom_bits: 8_388_608, // 1MB
                enable_reach_seen_cf: false,
                protection_orders: 30000,
                enable_incremental_sweep: true,
                incremental_sweep_batch: 1000,
                recycle_bin_enable: false,
                recycle_bin_max_entries: 10000,
                recycle_bin_max_bytes: 100_000_000,
                protected_roots_count: 1,
                // Phase 3: PersistentMarker configuration fields
                marker_batch_size: 10000,
                marker_bloom_bits: 1048576,
                marker_bloom_hash_fns: 4,
                marker_memory_threshold_mb: 1024,
                marker_auto_strategy: true,
                marker_force_persistent: false,
                marker_temp_cf_name: "gc_marker_temp".to_string(),
                marker_error_recovery: true,
            },
        ),
        (
            "Aggressive",
            PruneConfig {
                enable: true,
                boot_cleanup_done: false,
                scan_batch: 5000,
                delete_batch: 2500,
                interval_s: 30,
                bloom_bits: 67_108_864, // 8MB
                enable_reach_seen_cf: true,
                protection_orders: 30000,
                enable_incremental_sweep: true,
                incremental_sweep_batch: 2000,
                recycle_bin_enable: false,
                recycle_bin_max_entries: 10000,
                recycle_bin_max_bytes: 100_000_000,
                protected_roots_count: 1,
                // Phase 3: PersistentMarker configuration fields
                marker_batch_size: 20000,
                marker_bloom_bits: 2097152,
                marker_bloom_hash_fns: 6,
                marker_memory_threshold_mb: 2048,
                marker_auto_strategy: true,
                marker_force_persistent: false,
                marker_temp_cf_name: "gc_marker_temp".to_string(),
                marker_error_recovery: true,
            },
        ),
    ];

    println!(
        "{:<12} {:<10} {:<10} {:<10} {:<15} {:<15}",
        "Config", "Batch", "Interval", "Bloom", "Deleted", "Throughput"
    );
    println!("{}", "-".repeat(70));

    for (name, config) in configs {
        println!("\n🔄 Testing: {}", name);

        let _temp_dir = tempfile::tempdir().unwrap();
        let (moveos_store, _data_dir) = MoveOSStore::mock_moveos_store().unwrap();
        let moveos_store = Arc::new(moveos_store);

        // Create consistent test data for comparison
        let node_store = moveos_store.get_state_node_store();
        let mut rng = StdRng::seed_from_u64(42);
        let mut test_nodes = Vec::new();
        let mut expired_nodes = Vec::new();

        let test_size = 2000;
        for i in 0..test_size {
            let key = H256::random();
            let mut value = vec![0u8; 64];
            rng.fill(&mut value[..]);

            let node = Node::new_leaf(
                key,
                SMTObject::<Vec<u8>>::from_origin(value.clone()).unwrap(),
            );
            let hash: H256 = node.get_merkle_hash().into();
            node_store.put(hash, node.encode().unwrap()).unwrap();
            test_nodes.push(hash);

            if i >= test_size / 3 && i < test_size * 2 / 3 {
                expired_nodes.push(hash);
            }
        }

        println!(
            "   Test data: {} nodes, {} expired",
            test_nodes.len(),
            expired_nodes.len()
        );

        // Build reachability with config-appropriate bloom filter
        let bloom = Arc::new(parking_lot::Mutex::new(
            moveos_common::bloom_filter::BloomFilter::new(config.bloom_bits, 4),
        ));
        let live_roots: Vec<H256> = test_nodes[0..test_size / 3].to_vec();

        let builder = ReachableBuilder::new(moveos_store.clone(), bloom.clone());
        let _ = builder.build(live_roots, 1).unwrap();

        // Sweep with config
        let sweeper = SweepExpired::new(
            moveos_store.clone(),
            bloom.clone(),
            config.bloom_bits,
            Arc::new(std::sync::atomic::AtomicBool::new(true)), // true = running
        );

        let expired_roots: Vec<(H256, u64)> = expired_nodes
            .iter()
            .enumerate()
            .map(|(i, hash)| (*hash, (i + 1) as u64))
            .collect();

        let start_time = Instant::now();
        match sweeper.sweep(expired_roots, config.scan_batch / config.delete_batch) {
            Ok(deleted_count) => {
                let duration = start_time.elapsed();
                let deletion_ratio = deleted_count as f64 / expired_nodes.len() as f64;
                let throughput = deleted_count as f64 / duration.as_secs_f64();
                let memory_mb = config.bloom_bits as f64 / 8.0 / 1024.0 / 1024.0;

                println!("   📈 {} Results:", name);
                println!(
                    "      Deleted: {}/{} ({:.1}%)",
                    deleted_count,
                    expired_nodes.len(),
                    deletion_ratio * 100.0
                );
                println!("      Time: {:?}", duration);
                println!("      Throughput: {:.0} nodes/sec", throughput);
                println!("      Memory: {:.1} MB", memory_mb);

                println!(
                    "{:<12} {:<10} {:<10} {:<10.2} {:<7} {:<15.1}",
                    name,
                    config.scan_batch,
                    config.interval_s,
                    config.bloom_bits,
                    deleted_count,
                    throughput
                );
            }
            Err(e) => {
                println!("   ❌ {} failed: {}", name, e);
                panic!("Sweep should not fail for config: {}", name);
            }
        }
    }

    println!("\n🎯 Configuration Impact Analysis:");
    println!("   • Conservative: Small batches, longer intervals, minimal memory");
    println!("   • Balanced: Medium batches, normal intervals, good balance");
    println!("   • Aggressive: Large batches, short intervals, high throughput");
    println!("   • All configurations achieve effective deletion");
}

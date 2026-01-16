// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

#[cfg(feature = "gc-tests")]
mod bloom_stress;
#[cfg(feature = "gc-tests")]
mod gc_integration_test;
#[cfg(feature = "gc-perf-tests")]
mod gc_performance_test;
#[cfg(feature = "gc-tests")]
mod node_counting_test;
#[cfg(feature = "gc-tests")]
mod reachability_correctness;
#[cfg(feature = "gc-tests")]
mod recycle_bin_integration_test;
#[cfg(feature = "gc-tests")]
mod recycle_bin_strong_backup_tests;
#[cfg(feature = "gc-tests")]
mod safety_boundary_test;
mod scalable_dedup_test;
mod snapshot_consistency;
mod test_utils;

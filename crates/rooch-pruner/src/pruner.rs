// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

//! Pruner utilities after removing live pruner service
//!
//! This module now contains only utility functions that are shared between
//! the stop-the-world GC implementation and other components.

pub use rooch_config::prune_config::PruneConfig;

/// Calculate the starting tx_order for SweepExpired phase.
///
/// This utility function is preserved because it may be used by GC
/// or other components that need to determine sweep boundaries.
///
/// # Arguments
/// * `latest_order` - The latest tx_order in the chain
/// * `protection_orders` - Number of recent tx_orders to protect from pruning
///
/// # Returns
/// The tx_order from which to start sweeping (inclusive).
///
/// # Behavior
/// - If `protection_orders == 0`: Only protect the latest root (aggressive mode for testing)
/// - Otherwise: Protect the configured number of recent orders
pub fn calculate_sweep_start_order(latest_order: u64, protection_orders: u64) -> u64 {
    if protection_orders == 0 {
        // Aggressive mode: only protect the latest root
        // This is primarily for testing with --pruner-protection-orders 0
        if latest_order >= 1 {
            latest_order - 1
        } else {
            latest_order
        }
    } else {
        // Normal mode: protect configured number of orders
        latest_order.saturating_sub(protection_orders)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_sweep_start_order() {
        // Test normal protection
        assert_eq!(calculate_sweep_start_order(100, 10), 90);
        assert_eq!(calculate_sweep_start_order(50, 20), 30);
        assert_eq!(calculate_sweep_start_order(5, 10), 0); // saturating_sub

        // Test aggressive mode (protection_orders = 0)
        assert_eq!(calculate_sweep_start_order(10, 0), 9);
        assert_eq!(calculate_sweep_start_order(1, 0), 0);
        assert_eq!(calculate_sweep_start_order(0, 0), 0);

        // Test edge cases
        assert_eq!(calculate_sweep_start_order(0, 5), 0);
        assert_eq!(calculate_sweep_start_order(u64::MAX, 1), u64::MAX - 1);
    }
}

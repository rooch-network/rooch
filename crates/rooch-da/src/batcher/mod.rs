// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use rooch_config::settings::ROOCH_BATCH_INTERVAL;

pub struct BatchMaker {
    pub tx_order_start: u64,
    pub tx_order_end: u64,
    pub last_updated: u64,
}

impl BatchMaker {
    pub fn new() -> Self {
        Self {
            tx_order_start: 0,
            tx_order_end: 0,
            last_updated: 0,
        }
    }

    // Append transaction to batch:
    // 1. If the batch is empty(batch_start_time is 0), set the start time and order
    // 2. If the batch is not empty, check if the transaction is in the interval:
    // 2.1 If the transaction is in the interval, update the end order
    // 2.2 If the transaction is not in the interval, return the batch and reset the batch range
    pub fn append_transaction(&mut self, tx_order: u64, tx_timestamp: u64) -> Option<(u64, u64)> {
        if self.last_updated == 0 {
            self.tx_order_start = tx_order;
            self.tx_order_end = tx_order;
            self.last_updated = tx_timestamp;
            return None;
        }

        if tx_timestamp < self.last_updated {
            // avoid overflow caused by tx_timestamp - self.last_updated (clock goes back)
            self.tx_order_end = tx_order;
            return None;
        }

        if tx_timestamp - self.last_updated < ROOCH_BATCH_INTERVAL {
            self.tx_order_end = tx_order;
            self.last_updated = tx_timestamp;
            return None;
        }

        let batch = (self.tx_order_start, self.tx_order_end);
        self.tx_order_start = tx_order;
        self.tx_order_end = tx_order;
        self.last_updated = tx_timestamp;
        Some(batch)
    }
}

// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use rooch_config::settings::ROOCH_BATCH_INTERVAL;

pub struct BatchMaker {
    pub tx_order_start: u64,
    pub tx_order_end: u64,
    pub start_timestamp: u64,
}

impl BatchMaker {
    pub fn new() -> Self {
        Self {
            tx_order_start: 0,
            tx_order_end: 0,
            start_timestamp: 0,
        }
    }

    // Append transaction to batch:
    // 1. If the batch is empty(batch_start_time is 0), set the start time and order
    // 2. If the batch is not empty, check if the transaction is in the interval:
    // 2.1 If the transaction is in the interval, update the end order
    // 2.2 If the transaction is not in the interval, return the batch and reset the batch range
    pub fn append_transaction(&mut self, tx_order: u64, tx_timestamp: u64) -> Option<(u64, u64)> {
        if self.start_timestamp == 0 {
            self.tx_order_start = tx_order;
            self.tx_order_end = tx_order;
            self.start_timestamp = tx_timestamp;
            return None;
        }

        if tx_timestamp < self.start_timestamp ||        // avoid overflow caused by tx_timestamp - self.last_updated (clock goes back)
            tx_timestamp - self.start_timestamp < ROOCH_BATCH_INTERVAL
        {
            self.tx_order_end = tx_order;
            return None;
        }

        let batch = (self.tx_order_start, self.tx_order_end);
        self.tx_order_start = tx_order;
        self.tx_order_end = tx_order;
        self.start_timestamp = tx_timestamp;
        Some(batch)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_append_transaction() {
        let mut batch_maker = BatchMaker::new();
        let tx_order = 1;
        let tx_timestamp = 1;
        assert_eq!(batch_maker.append_transaction(tx_order, tx_timestamp), None);

        let tx_order = 2;
        let tx_timestamp = 2;
        assert_eq!(batch_maker.append_transaction(tx_order, tx_timestamp), None);

        let tx_order = 3;
        let tx_timestamp = 3;
        assert_eq!(batch_maker.append_transaction(tx_order, tx_timestamp), None);

        let tx_order = 4;
        let tx_timestamp = 4;
        assert_eq!(batch_maker.append_transaction(tx_order, tx_timestamp), None);

        let tx_order = 5;
        let tx_timestamp = 1 + ROOCH_BATCH_INTERVAL;
        assert_eq!(
            batch_maker.append_transaction(tx_order, tx_timestamp),
            Some((1, 4))
        );

        let tx_order = 6;
        let tx_timestamp = 6; // clock goes back
        assert_eq!(batch_maker.append_transaction(tx_order, tx_timestamp), None);

        let tx_order = 7;
        let tx_timestamp = 7;
        assert_eq!(batch_maker.append_transaction(tx_order, tx_timestamp), None);

        let tx_order = 8;
        let tx_timestamp = 8;
        assert_eq!(batch_maker.append_transaction(tx_order, tx_timestamp), None);

        let tx_order = 9;
        let tx_timestamp = 1 + ROOCH_BATCH_INTERVAL + ROOCH_BATCH_INTERVAL;
        assert_eq!(
            batch_maker.append_transaction(tx_order, tx_timestamp),
            Some((5, 8))
        );
    }
}

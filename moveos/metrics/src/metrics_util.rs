// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

pub const LATENCY_SEC_BUCKETS: &[f64] = &[
    0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1., 2.5, 5., 10., 20., 30., 60., 90.,
];

#[derive(Debug, Clone)]
// A struct for sampling based on number of operations or duration.
// Sampling happens if the duration expires and after number of operations
pub struct SamplingInterval {
    // Sample once every time duration
    pub once_every_duration: Duration,
    // Sample once every number of operations
    pub after_num_ops: u64,
    // Counter for keeping track of previous sample
    pub counter: Arc<AtomicU64>,
}

impl Default for SamplingInterval {
    fn default() -> Self {
        // Enabled with 60 second interval
        SamplingInterval::new(Duration::from_secs(60), 0)
    }
}

impl SamplingInterval {
    pub fn new(once_every_duration: Duration, after_num_ops: u64) -> Self {
        let counter = Arc::new(AtomicU64::new(1));
        if !once_every_duration.is_zero() {
            let counter = counter.clone();
            tokio::task::spawn(async move {
                loop {
                    if counter.load(Ordering::SeqCst) > after_num_ops {
                        counter.store(0, Ordering::SeqCst);
                    }
                    tokio::time::sleep(once_every_duration).await;
                }
            });
        }
        SamplingInterval {
            once_every_duration,
            after_num_ops,
            counter,
        }
    }

    pub fn new_from_self(&self) -> SamplingInterval {
        SamplingInterval::new(self.once_every_duration, self.after_num_ops)
    }

    pub fn sample(&self) -> bool {
        if self.once_every_duration.is_zero() {
            self.counter.fetch_add(1, Ordering::Relaxed) % (self.after_num_ops + 1) == 0
        } else {
            self.counter.fetch_add(1, Ordering::Relaxed) == 0
        }
    }
}

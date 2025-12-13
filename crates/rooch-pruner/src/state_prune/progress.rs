// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Progress tracking for state prune operations
#[derive(Debug)]
pub struct ProgressTracker {
    /// Total items to process
    total_items: AtomicU64,

    /// Processed items
    processed_items: AtomicU64,

    /// Start time
    start_time: Instant,

    /// Last progress report time
    last_report: Arc<AtomicU64>,

    /// Progress reporting interval (seconds)
    report_interval: Duration,
}

impl ProgressTracker {
    /// Create new progress tracker
    pub fn new(report_interval_seconds: u64) -> Self {
        Self {
            total_items: AtomicU64::new(0),
            processed_items: AtomicU64::new(0),
            start_time: Instant::now(),
            last_report: Arc::new(AtomicU64::new(0)),
            report_interval: Duration::from_secs(report_interval_seconds),
        }
    }

    /// Set total items
    pub fn set_total(&self, total: u64) {
        self.total_items.store(total, Ordering::Relaxed);
    }

    /// Increment processed items count
    pub fn increment_processed(&self, count: u64) {
        self.processed_items.fetch_add(count, Ordering::Relaxed);
    }

    /// Get current progress percentage
    pub fn progress_percentage(&self) -> f64 {
        let total = self.total_items.load(Ordering::Relaxed);
        let processed = self.processed_items.load(Ordering::Relaxed);

        if total == 0 {
            0.0
        } else {
            (processed as f64 / total as f64) * 100.0
        }
    }

    /// Get elapsed time
    pub fn elapsed_time(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// Estimate remaining time
    pub fn estimated_remaining_time(&self) -> Option<Duration> {
        let total = self.total_items.load(Ordering::Relaxed);
        let processed = self.processed_items.load(Ordering::Relaxed);
        let elapsed = self.start_time.elapsed();

        if processed == 0 {
            None
        } else {
            let rate = processed as f64 / elapsed.as_secs_f64();
            let remaining = total.saturating_sub(processed);
            let remaining_secs = remaining as f64 / rate;
            Some(Duration::from_secs_f64(remaining_secs))
        }
    }

    /// Check if should report progress
    pub fn should_report(&self) -> bool {
        let now = self.start_time.elapsed().as_secs();
        let last = self.last_report.load(Ordering::Relaxed);
        now >= last + self.report_interval.as_secs()
    }

    /// Mark progress as reported
    pub fn mark_reported(&self) {
        let now = self.start_time.elapsed().as_secs();
        self.last_report.store(now, Ordering::Relaxed);
    }

    /// Get current progress report
    pub fn get_progress_report(&self) -> ProgressReport {
        ProgressReport {
            total_items: self.total_items.load(Ordering::Relaxed),
            processed_items: self.processed_items.load(Ordering::Relaxed),
            progress_percentage: self.progress_percentage(),
            elapsed_time: self.elapsed_time(),
            estimated_remaining_time: self.estimated_remaining_time(),
        }
    }
}

/// Progress report structure
#[derive(Debug, Clone)]
pub struct ProgressReport {
    pub total_items: u64,
    pub processed_items: u64,
    pub progress_percentage: f64,
    pub elapsed_time: Duration,
    pub estimated_remaining_time: Option<Duration>,
}

impl ProgressReport {
    /// Format progress report for logging
    pub fn format(&self) -> String {
        let progress_str = format!("{:.2}%", self.progress_percentage);
        let elapsed_str = format!("{:.2}s", self.elapsed_time.as_secs_f64());

        let remaining_str = if let Some(remaining) = self.estimated_remaining_time {
            format!("{:.2}s", remaining.as_secs_f64())
        } else {
            "unknown".to_string()
        };

        format!(
            "Progress: {} ({}/{}) items, Elapsed: {}, ETA: {}",
            progress_str, self.processed_items, self.total_items, elapsed_str, remaining_str
        )
    }

    /// Get items per second rate
    pub fn items_per_second(&self) -> f64 {
        if self.elapsed_time.as_secs() == 0 {
            0.0
        } else {
            self.processed_items as f64 / self.elapsed_time.as_secs_f64()
        }
    }

    /// Check if operation is complete
    pub fn is_complete(&self) -> bool {
        self.processed_items >= self.total_items && self.total_items > 0
    }
}

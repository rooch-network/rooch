// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

use anyhow::Result;
use coarsetime::Instant;
use metrics::{
    self, register, HistogramOpts, HistogramVec, Opts, PrometheusError, Registry, UIntCounterVec,
    UIntGauge,
};

#[derive(Clone)]
pub struct StoreMetrics {
    pub store_rw_total: UIntCounterVec,
    pub store_item_bytes: HistogramVec,
    pub store_time: HistogramVec,
    pub cache_items: UIntGauge,
}

impl StoreMetrics {
    pub fn register(registry: &Registry) -> Result<Self, PrometheusError> {
        let store_rw_total = register(
            UIntCounterVec::new(
                Opts::new("store_rw_total", "Counters of how many store read/write"),
                &["store_type", "key_type", "method", "result"],
            )?,
            registry,
        )?;
        let store_item_bytes = register(
            HistogramVec::new(
                HistogramOpts::new("store_item_bytes", "store write item size in bytes"),
                &["key_type"],
            )?,
            registry,
        )?;

        let store_time = register(
            HistogramVec::new(
                HistogramOpts::new(
                    "store_time",
                    "Histogram of store, measure store method time usage.",
                ),
                &["store_type", "key_type", "method"],
            )?,
            registry,
        )?;

        let cache_items = register(
            UIntGauge::with_opts(Opts::new("cache_items", "How many items in cache"))?,
            registry,
        )?;

        Ok(Self {
            store_rw_total,
            store_item_bytes,
            store_time,
            cache_items,
        })
    }
}

#[allow(clippy::upper_case_acronyms)]
pub enum ResultType {
    NONE,
    SOME,
    OK,
    ERROR,
}

impl ResultType {
    pub fn into_str(self) -> &'static str {
        match self {
            ResultType::NONE => "none",
            ResultType::ERROR => "error",
            ResultType::OK => "ok",
            ResultType::SOME => "some",
        }
    }
}

pub trait AsResultType {
    fn as_result_type(&self) -> ResultType;
}

impl AsResultType for Result<()> {
    fn as_result_type(&self) -> ResultType {
        match self {
            Ok(_) => ResultType::OK,
            Err(_) => ResultType::ERROR,
        }
    }
}

impl AsResultType for Result<bool> {
    fn as_result_type(&self) -> ResultType {
        match self {
            Ok(_) => ResultType::OK,
            Err(_) => ResultType::ERROR,
        }
    }
}

impl<T> AsResultType for Result<Option<T>> {
    fn as_result_type(&self) -> ResultType {
        match self {
            Ok(v) => match v {
                Some(_) => ResultType::SOME,
                None => ResultType::NONE,
            },
            Err(_) => ResultType::ERROR,
        }
    }
}

impl<T> AsResultType for Result<Vec<Option<T>>> {
    fn as_result_type(&self) -> ResultType {
        match self {
            Ok(_) => ResultType::OK,
            Err(_) => ResultType::ERROR,
        }
    }
}

pub struct MetricsRecord<'a> {
    store_type: &'a str,
    key_type: &'a str,
    method: &'a str,
    timer: Instant,
    metrics: Option<&'a StoreMetrics>,
}

impl<'a> MetricsRecord<'a> {
    pub fn new(
        store_type: &'a str,
        key_type: &'a str,
        method: &'a str,
        metrics: Option<&'a StoreMetrics>,
    ) -> Self {
        let timer = Instant::now();
        MetricsRecord {
            store_type,
            key_type,
            method,
            timer,
            metrics,
        }
    }

    pub fn record<R>(self, result: R) -> R
    where
        R: AsResultType,
    {
        let result_type = result.as_result_type();
        if let Some(metrics) = self.metrics {
            metrics
                .store_rw_total
                .with_label_values(&[
                    self.store_type,
                    self.key_type,
                    self.method,
                    result_type.into_str(),
                ])
                .inc();
            metrics
                .store_time
                .with_label_values(&[self.store_type, self.key_type, self.method])
                .observe(self.timer.elapsed().as_f64());
        }
        result
    }

    pub fn call<R, F>(self, f: F) -> R
    where
        F: FnOnce() -> R,
        R: AsResultType,
    {
        let r = f();
        self.record(r)
    }
}

//TODO implement a generic metrics macros.
pub fn record_metrics<'a>(
    store_type: &'a str,
    key_type: &'a str,
    method: &'a str,
    metrics: Option<&'a StoreMetrics>,
) -> MetricsRecord<'a> {
    MetricsRecord::new(store_type, key_type, method, metrics)
}

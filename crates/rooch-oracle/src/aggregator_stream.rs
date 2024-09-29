// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::datasource::OracleDecimalData;
use anyhow::Result;
use futures::stream::Stream;
use futures::task::{Context, Poll};
use move_core_types::u256::U256;
use pin_project::pin_project;
use std::collections::VecDeque;
use std::fmt;
use std::pin::Pin;
use std::str::FromStr;
use tracing::warn;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, clap::ValueEnum)]
pub enum AggregateStrategy {
    /// Calculate the average of the data
    #[default]
    Average,
    /// Calculate the median of the data
    Median,
    /// Calculate the mode of the data
    Mode,
}

impl FromStr for AggregateStrategy {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "average" => Ok(AggregateStrategy::Average),
            "median" => Ok(AggregateStrategy::Median),
            "mode" => Ok(AggregateStrategy::Mode),
            _ => Err(anyhow::anyhow!("Invalid aggregator strategy")),
        }
    }
}

impl fmt::Display for AggregateStrategy {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AggregateStrategy::Average => write!(f, "average"),
            AggregateStrategy::Median => write!(f, "median"),
            AggregateStrategy::Mode => write!(f, "mode"),
        }
    }
}

impl AggregateStrategy {
    pub fn aggregate(&self, data: Vec<OracleDecimalData>) -> OracleDecimalData {
        match self {
            AggregateStrategy::Average => {
                let mut sum: U256 = U256::zero();
                for d in data.iter() {
                    sum += d.value;
                }
                let avg = sum / U256::from(data.len() as u64);
                OracleDecimalData {
                    value: avg,
                    decimal: data[0].decimal,
                }
            }
            AggregateStrategy::Median => {
                let mut sorted_data = data.clone();
                sorted_data.sort_by(|a, b| a.value.cmp(&b.value));
                let mid = sorted_data.len() / 2;
                let median = if sorted_data.len() % 2 == 0 {
                    (sorted_data[mid].value + sorted_data[mid - 1].value) / U256::from(2u64)
                } else {
                    sorted_data[mid].value
                };
                OracleDecimalData {
                    value: median,
                    decimal: data[0].decimal,
                }
            }
            AggregateStrategy::Mode => {
                let mut freq_map = std::collections::HashMap::new();
                for d in data.iter() {
                    *freq_map.entry(d.value).or_insert(0) += 1;
                }
                let mode = freq_map.iter().max_by_key(|&(_, count)| count).unwrap().0;
                OracleDecimalData {
                    value: *mode,
                    decimal: data[0].decimal,
                }
            }
        }
    }
}

#[pin_project]
pub struct AggregatorStream<S> {
    #[pin]
    inner: S,
    strategy: AggregateStrategy,
    buffer: VecDeque<OracleDecimalData>,
}

impl<S> AggregatorStream<S>
where
    S: Stream<Item = Result<OracleDecimalData>>,
{
    pub fn new(inner: S, strategy: AggregateStrategy) -> Self {
        Self {
            inner,
            strategy,
            buffer: VecDeque::with_capacity(100),
        }
    }
}

impl<S> Stream for AggregatorStream<S>
where
    S: Stream<Item = Result<OracleDecimalData>>,
{
    type Item = OracleDecimalData;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();

        while let Poll::Ready(Some(item)) = this.inner.as_mut().poll_next(cx) {
            match item {
                Ok(data) => {
                    this.buffer.push_back(data);
                }
                Err(e) => {
                    warn!("Error in stream: {}", e);
                }
            }
        }

        if !this.buffer.is_empty() {
            let result = this.buffer.drain(..).collect();
            Poll::Ready(Some(this.strategy.aggregate(result)))
        } else {
            Poll::Pending
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::stream::StreamExt;

    #[test]
    fn test_agg_strategy() {
        let data = vec![
            OracleDecimalData {
                value: U256::from(100u64),
                decimal: 2,
            },
            OracleDecimalData {
                value: U256::from(200u64),
                decimal: 2,
            },
            OracleDecimalData {
                value: U256::from(300u64),
                decimal: 2,
            },
            OracleDecimalData {
                value: U256::from(400u64),
                decimal: 2,
            },
            OracleDecimalData {
                value: U256::from(500u64),
                decimal: 2,
            },
            OracleDecimalData {
                value: U256::from(100u64),
                decimal: 2,
            }, //two 100s
        ];

        let avg = AggregateStrategy::Average.aggregate(data.clone());
        assert_eq!(avg.value, U256::from(266u64));
        assert_eq!(avg.decimal, 2);

        let median = AggregateStrategy::Median.aggregate(data.clone());
        assert_eq!(median.value, U256::from(250u64));
        assert_eq!(median.decimal, 2);

        let mode = AggregateStrategy::Mode.aggregate(data.clone());
        assert_eq!(mode.value, U256::from(100u64));
        assert_eq!(mode.decimal, 2);
    }

    #[tokio::test]
    async fn test_agg_stream() {
        let data_stream = futures::stream::iter(vec![
            Ok(OracleDecimalData {
                value: U256::from(100u64),
                decimal: 2,
            }),
            Ok(OracleDecimalData {
                value: U256::from(200u64),
                decimal: 2,
            }),
            Ok(OracleDecimalData {
                value: U256::from(300u64),
                decimal: 2,
            }),
            Ok(OracleDecimalData {
                value: U256::from(400u64),
                decimal: 2,
            }),
            Ok(OracleDecimalData {
                value: U256::from(500u64),
                decimal: 2,
            }),
        ]);
        let mut agg_stream = AggregatorStream::new(data_stream, AggregateStrategy::Average);

        let result = agg_stream.next().await;
        assert_eq!(
            result,
            Some(OracleDecimalData {
                value: U256::from(300u64),
                decimal: 2
            })
        );
    }
}

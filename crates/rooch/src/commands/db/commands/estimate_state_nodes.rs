// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::CommandAction;
use crate::utils::open_rooch_db_readonly;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use clap::Parser;
use moveos_types::h256::H256;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use rooch_pruner::util::extract_child_nodes_strict;
use rooch_types::error::RoochResult;
use rustc_hash::FxHashSet;
use serde::Serialize;
use smt::{NodeReader, SPARSE_MERKLE_PLACEHOLDER_HASH};
use std::path::PathBuf;
use std::time::Instant;

#[derive(Debug, Clone, Serialize)]
struct SamplingRunReport {
    run: u32,
    seed: u64,
    estimate_nodes: u64,
    sampled_nodes: u64,
    queue_max: usize,
    elapsed_ms: u128,
    truncated: bool,
}

#[derive(Debug, Clone, Serialize)]
struct SamplingSummary {
    state_root: String,
    runs: u32,
    median_estimate_nodes: u64,
    min_estimate_nodes: u64,
    max_estimate_nodes: u64,
    mean_estimate_nodes: u64,
    truncated_runs: u32,
    config: SamplingConfig,
    reports: Vec<SamplingRunReport>,
}

#[derive(Debug, Clone, Serialize)]
struct SamplingConfig {
    full_depth: u16,
    mid_depth_span: u16,
    mid_prob: f64,
    deep_prob: f64,
    max_sampled_nodes: usize,
    skip_dedup: bool,
}

#[derive(Debug, Clone, Copy)]
struct SamplingState {
    estimate: f64,
    sampled_nodes: u64,
    queue_max: usize,
    truncated: bool,
}

#[derive(Debug, Clone, Copy)]
struct SampleTask {
    hash: H256,
    depth: u16,
    weight: f64,
}

/// Estimate node count for a state_root using stratified random sampling.
///
/// This is a fast estimator:
/// - `full_depth`: fully expand shallow levels
/// - `mid_prob`: sample middle levels
/// - `deep_prob`: sample deep levels
///
/// Run multiple times (`--runs`) and use the median for a stable estimate.
#[derive(Debug, Parser)]
pub struct EstimateStateNodesCommand {
    /// Base data directory for the blockchain data
    #[clap(long = "data-dir", short = 'd')]
    pub base_data_dir: Option<PathBuf>,

    /// Chain ID to specify which blockchain network
    #[clap(long, short = 'n')]
    pub chain_id: rooch_types::rooch_network::BuiltinChainID,

    /// Target state root hash (hex string, with or without 0x)
    #[clap(long)]
    pub state_root: String,

    /// Number of independent sampling runs
    #[clap(long, default_value_t = 7)]
    pub runs: u32,

    /// Fully expand all nodes up to this depth
    #[clap(long, default_value_t = 14)]
    pub full_depth: u16,

    /// Depth span after full_depth to use mid_prob sampling
    #[clap(long, default_value_t = 12)]
    pub mid_depth_span: u16,

    /// Sampling probability for middle-depth nodes
    #[clap(long, default_value_t = 0.35)]
    pub mid_prob: f64,

    /// Sampling probability for deep nodes
    #[clap(long, default_value_t = 0.15)]
    pub deep_prob: f64,

    /// Maximum sampled unique nodes per run (safety cap)
    #[clap(long, default_value_t = 2_000_000)]
    pub max_sampled_nodes: usize,

    /// Simulate `state-prune snapshot --skip-dedup` and count node visits (not unique hashes)
    #[clap(long)]
    pub skip_dedup: bool,

    /// Base RNG seed (deterministic). If omitted, uses system randomness.
    #[clap(long)]
    pub seed: Option<u64>,

    /// Output JSON instead of text
    #[clap(long)]
    pub json: bool,
}

#[async_trait]
impl CommandAction<String> for EstimateStateNodesCommand {
    async fn execute(self) -> RoochResult<String> {
        self.validate()?;
        let state_root = parse_state_root(&self.state_root)?;

        let (_root, rooch_db, _start_time) = open_rooch_db_readonly(
            self.base_data_dir,
            Some(rooch_types::rooch_network::RoochChainID::Builtin(
                self.chain_id,
            )),
        );
        let node_store = rooch_db.moveos_store.get_state_node_store();

        let base_seed = self.seed.unwrap_or_else(|| rand::thread_rng().gen::<u64>());
        let mut reports = Vec::with_capacity(self.runs as usize);

        for run in 0..self.runs {
            let run_seed = base_seed.wrapping_add(run as u64);
            let started = Instant::now();
            let run_state = sample_once(
                node_store,
                state_root,
                &SamplingConfig {
                    full_depth: self.full_depth,
                    mid_depth_span: self.mid_depth_span,
                    mid_prob: self.mid_prob,
                    deep_prob: self.deep_prob,
                    max_sampled_nodes: self.max_sampled_nodes,
                    skip_dedup: self.skip_dedup,
                },
                run_seed,
            )?;
            reports.push(SamplingRunReport {
                run: run + 1,
                seed: run_seed,
                estimate_nodes: run_state.estimate.round() as u64,
                sampled_nodes: run_state.sampled_nodes,
                queue_max: run_state.queue_max,
                elapsed_ms: started.elapsed().as_millis(),
                truncated: run_state.truncated,
            });
        }

        let mut estimates: Vec<u64> = reports.iter().map(|r| r.estimate_nodes).collect();
        estimates.sort_unstable();
        let median_estimate_nodes = percentile(&estimates, 50);
        let min_estimate_nodes = *estimates.first().unwrap_or(&0);
        let max_estimate_nodes = *estimates.last().unwrap_or(&0);
        let mean_estimate_nodes = if estimates.is_empty() {
            0
        } else {
            (estimates.iter().sum::<u64>() as f64 / estimates.len() as f64).round() as u64
        };
        let truncated_runs = reports.iter().filter(|r| r.truncated).count() as u32;

        let summary = SamplingSummary {
            state_root: format!("{:x}", state_root),
            runs: self.runs,
            median_estimate_nodes,
            min_estimate_nodes,
            max_estimate_nodes,
            mean_estimate_nodes,
            truncated_runs,
            config: SamplingConfig {
                full_depth: self.full_depth,
                mid_depth_span: self.mid_depth_span,
                mid_prob: self.mid_prob,
                deep_prob: self.deep_prob,
                max_sampled_nodes: self.max_sampled_nodes,
                skip_dedup: self.skip_dedup,
            },
            reports,
        };

        if self.json {
            return Ok(serde_json::to_string_pretty(&summary)?);
        }

        Ok(format_summary(&summary))
    }
}

impl EstimateStateNodesCommand {
    fn validate(&self) -> Result<()> {
        if self.runs == 0 {
            return Err(anyhow!("runs must be greater than 0"));
        }
        if self.max_sampled_nodes == 0 {
            return Err(anyhow!("max_sampled_nodes must be greater than 0"));
        }
        if !(0.0..=1.0).contains(&self.mid_prob) || self.mid_prob == 0.0 {
            return Err(anyhow!("mid_prob must be in (0, 1]"));
        }
        if !(0.0..=1.0).contains(&self.deep_prob) || self.deep_prob == 0.0 {
            return Err(anyhow!("deep_prob must be in (0, 1]"));
        }
        Ok(())
    }
}

fn parse_state_root(input: &str) -> Result<H256> {
    let trimmed = input.strip_prefix("0x").unwrap_or(input);
    let bytes = hex::decode(trimmed).map_err(|e| anyhow!("invalid state_root hex: {}", e))?;
    if bytes.len() != 32 {
        return Err(anyhow!(
            "invalid state_root length: expected 32 bytes, got {}",
            bytes.len()
        ));
    }
    Ok(H256::from_slice(&bytes))
}

fn sample_once<NR: NodeReader>(
    node_reader: &NR,
    root: H256,
    config: &SamplingConfig,
    seed: u64,
) -> Result<SamplingState> {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut seen: FxHashSet<H256> = FxHashSet::default();
    let mut stack = vec![SampleTask {
        hash: root,
        depth: 0,
        weight: 1.0,
    }];
    let mut estimate = 0f64;
    let mut queue_max = 1usize;
    let mut sampled_nodes = 0u64;
    let mut truncated = false;

    while let Some(task) = stack.pop() {
        if task.hash == *SPARSE_MERKLE_PLACEHOLDER_HASH {
            continue;
        }
        if !config.skip_dedup && !seen.insert(task.hash) {
            continue;
        }

        sampled_nodes += 1;
        estimate += task.weight;

        if sampled_nodes as usize >= config.max_sampled_nodes {
            truncated = true;
            break;
        }

        let bytes = node_reader.get(&task.hash)?.ok_or_else(|| {
            anyhow!(
                "missing state node during sampling: {:x} (depth={})",
                task.hash,
                task.depth
            )
        })?;
        let children = extract_child_nodes_strict(&bytes)?;
        let child_depth = task.depth.saturating_add(1);
        let p = sample_probability(child_depth, config);

        for child in children {
            if p >= 1.0 {
                stack.push(SampleTask {
                    hash: child,
                    depth: child_depth,
                    weight: task.weight,
                });
                continue;
            }
            if rng.gen::<f64>() <= p {
                stack.push(SampleTask {
                    hash: child,
                    depth: child_depth,
                    weight: task.weight / p,
                });
            }
        }

        if stack.len() > queue_max {
            queue_max = stack.len();
        }
    }

    Ok(SamplingState {
        estimate,
        sampled_nodes,
        queue_max,
        truncated,
    })
}

fn sample_probability(depth: u16, config: &SamplingConfig) -> f64 {
    if depth <= config.full_depth {
        return 1.0;
    }
    if depth <= config.full_depth.saturating_add(config.mid_depth_span) {
        return config.mid_prob;
    }
    config.deep_prob
}

fn percentile(values: &[u64], p: usize) -> u64 {
    if values.is_empty() {
        return 0;
    }
    let idx = ((values.len() - 1) * p) / 100;
    values[idx]
}

fn format_summary(summary: &SamplingSummary) -> String {
    use std::fmt::Write;

    let mut out = String::new();
    writeln!(
        out,
        "=== State Root Node Estimate (Stratified Sampling) ==="
    )
    .ok();
    writeln!(out, "state_root: {}", summary.state_root).ok();
    writeln!(out, "runs: {}", summary.runs).ok();
    writeln!(
        out,
        "median_estimate_nodes: {}",
        summary.median_estimate_nodes
    )
    .ok();
    writeln!(out, "mean_estimate_nodes: {}", summary.mean_estimate_nodes).ok();
    writeln!(out, "min_estimate_nodes: {}", summary.min_estimate_nodes).ok();
    writeln!(out, "max_estimate_nodes: {}", summary.max_estimate_nodes).ok();
    writeln!(out, "truncated_runs: {}", summary.truncated_runs).ok();
    writeln!(out).ok();
    writeln!(out, "config:").ok();
    writeln!(out, "  full_depth: {}", summary.config.full_depth).ok();
    writeln!(out, "  mid_depth_span: {}", summary.config.mid_depth_span).ok();
    writeln!(out, "  mid_prob: {:.4}", summary.config.mid_prob).ok();
    writeln!(out, "  deep_prob: {:.4}", summary.config.deep_prob).ok();
    writeln!(out, "  skip_dedup: {}", summary.config.skip_dedup).ok();
    writeln!(
        out,
        "  max_sampled_nodes: {}",
        summary.config.max_sampled_nodes
    )
    .ok();
    writeln!(out).ok();
    writeln!(out, "runs_detail:").ok();
    for report in &summary.reports {
        writeln!(
            out,
            "  run={} seed={} estimate={} sampled={} queue_max={} elapsed_ms={} truncated={}",
            report.run,
            report.seed,
            report.estimate_nodes,
            report.sampled_nodes,
            report.queue_max,
            report.elapsed_ms,
            report.truncated
        )
        .ok();
    }
    out
}

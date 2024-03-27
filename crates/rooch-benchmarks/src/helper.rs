// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::helper::PProfOut::{Flamegraph, Protobuf};
use criterion::Criterion;
use lazy_static::lazy_static;
use pprof::criterion::{Output, PProfProfiler};
use std::env;
use std::ffi::c_int;
use std::fmt::Display;
use std::str::FromStr;
use std::time::Duration;

#[derive(PartialEq, Eq)]
pub enum PProfOut {
    Protobuf,
    Flamegraph,
}

impl FromStr for PProfOut {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "proto" => Ok(Protobuf),
            "flamegraph" => Ok(Flamegraph),
            _ => Ok(Flamegraph),
        }
    }
}

impl Display for PProfOut {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Protobuf => write!(f, "proto"),
            Flamegraph => write!(f, "flamegraph"),
        }
    }
}

lazy_static! {
    pub static ref PPROF_OUT: PProfOut = {
        let pprof_out_str = env::var("PPROF_OUT").unwrap_or_else(|_| String::from("flamegraph"));
        pprof_out_str.parse::<PProfOut>().unwrap_or(Flamegraph)
    };
}

pub struct ProfileConfig {
    pub sample_size: usize,
    pub warm_up_time: Duration,
    pub frequency: c_int,
    pub measurement_time: Duration,
}

impl Default for ProfileConfig {
    fn default() -> Self {
        Self {
            sample_size: 10,
            warm_up_time: Duration::from_millis(10),
            frequency: 2000,
            measurement_time: Duration::from_millis(500),
        }
    }
}

pub fn profiled(config: Option<ProfileConfig>) -> Criterion {
    let cfg = config.unwrap_or_default();

    let profiler = match *PPROF_OUT {
        Protobuf => PProfProfiler::new(cfg.frequency, Output::Protobuf),
        Flamegraph => PProfProfiler::new(cfg.frequency, Output::Flamegraph(None)),
    };
    Criterion::default()
        .with_profiler(profiler)
        .warm_up_time(cfg.warm_up_time) // no need to warm this heavy operation
        .sample_size(cfg.sample_size)
        .measurement_time(cfg.measurement_time)
}

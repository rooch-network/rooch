// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::helper::PProfOut::{Flamegraph, Protobuf};
use lazy_static::lazy_static;
use std::env;
use std::fmt::Display;
use std::str::FromStr;

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
        pprof_out_str
            .parse::<PProfOut>()
            .unwrap_or(PProfOut::Flamegraph)
    };
}

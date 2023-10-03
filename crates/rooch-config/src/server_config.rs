// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::config::Config;
use serde::Deserialize;
use serde::Serialize;
use std::fmt::{Display, Formatter, Result, Write};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub block_propose_duration_in_seconds: u16,
}

impl ServerConfig {
    pub fn url(&self, https: bool) -> String {
        let schema = if https { "https" } else { "http" };

        format!("{}://{}:{}", schema, self.host, self.port)
    }

    pub fn new_with_port(port: u16) -> Self {
        Self {
            port,
            ..Default::default()
        }
    }
}

impl Config for ServerConfig {}

impl Display for ServerConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let mut writer = String::new();

        writeln!(writer, "host : {}", self.host)?;
        writeln!(writer, "port : {}", self.port)?;

        write!(f, "{}", writer)
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 50051,
            block_propose_duration_in_seconds: 5,
        }
    }
}

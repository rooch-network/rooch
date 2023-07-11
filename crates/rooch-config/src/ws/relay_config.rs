// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use serde::Deserialize;
use serde::Serialize;
use std::fmt::{Display, Formatter, Result, Write};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct RelayConfig {
    pub host: String,
    pub port: u16,
    pub remote_ip_header: Option<String>,
    pub ping_interval_seconds: u32,
}

impl RelayConfig {
    pub fn ws_url(&self, https: bool) -> String {
        let schema = if https { "wss" } else { "ws" };

        format!("{}://{}:{}", schema, self.host, self.port)
    }
}

impl Display for RelayConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let mut writer = String::new();

        writeln!(writer, "host : {}", self.host)?;
        writeln!(writer, "port : {}", self.port)?;

        write!(f, "{}", writer)
    }
}

impl Default for RelayConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_owned(),
            port: 8080,
            remote_ip_header: None,
            ping_interval_seconds: 300,
        }
    }
}

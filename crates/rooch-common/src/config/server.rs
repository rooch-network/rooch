// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter, Result, Write};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

impl ServerConfig {
    pub fn url(&self, https: bool) -> String {
        let schema = if https { "https" } else { "http" };

        format!("{}://{}:{}", schema, self.host, self.port)
    }
}

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
        }
    }
}

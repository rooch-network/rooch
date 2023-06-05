// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::{Client, ClientBuilder};
use anyhow::anyhow;
use rooch_config::Config;
use rooch_key::keystore::{AccountKeystore, Keystore};
use rooch_server::server_config::ServerConfig;
use rooch_types::address::RoochAddress;
use serde::Deserialize;
use serde::Serialize;
use serde_with::serde_as;
use std::fmt::{Display, Formatter, Write};

pub const DEFAULT_EXPIRATION_SECS: u64 = 30;

#[serde_as]
#[derive(Serialize, Deserialize)]
pub struct ClientConfig {
    pub keystore: Keystore,
    pub active_address: Option<RoochAddress>,
    pub envs: Vec<Env>,
    pub active_env: Option<String>,
}

impl ClientConfig {
    pub fn new(keystore: Keystore) -> Self {
        ClientConfig {
            keystore,
            active_address: None,
            envs: vec![],
            active_env: None,
        }
    }

    pub fn get_env(&self, alias: &Option<String>) -> Option<&Env> {
        if let Some(alias) = alias {
            self.envs.iter().find(|env| &env.alias == alias)
        } else {
            self.envs.first()
        }
    }

    pub fn get_active_env(&self) -> Result<&Env, anyhow::Error> {
        self.get_env(&self.active_env).ok_or_else(|| {
            anyhow!(
                "Environment configuration not found for env [{}]",
                self.active_env.as_deref().unwrap_or("None")
            )
        })
    }

    pub fn add_env(&mut self, env: Env) {
        if !self
            .envs
            .iter()
            .any(|other_env| other_env.alias == env.alias)
        {
            self.envs.push(env)
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Env {
    pub alias: String,
    pub rpc: String,
    pub ws: Option<String>,
}

impl Env {
    pub async fn create_rpc_client(
        &self,
        request_timeout: std::time::Duration,
        max_concurrent_requests: Option<u64>,
    ) -> Result<Client, anyhow::Error> {
        let mut builder = ClientBuilder::default();
        builder = builder.request_timeout(request_timeout);
        if let Some(ws_url) = &self.ws {
            builder = builder.ws_url(ws_url);
        }

        if let Some(max_concurrent_requests) = max_concurrent_requests {
            builder = builder.max_concurrent_requests(max_concurrent_requests as usize);
        }

        builder.build(&self.rpc).await
    }
}

impl Default for Env {
    fn default() -> Self {
        Env {
            alias: "default".to_string(),
            rpc: ServerConfig::default().url(false),
            ws: None,
        }
    }
}

impl Display for Env {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut writer = String::new();
        writeln!(writer, "Active environment : {}", self.alias)?;
        write!(writer, "RPC URL: {}", self.rpc)?;
        if let Some(ws) = &self.ws {
            writeln!(writer)?;
            write!(writer, "Websocket URL: {ws}")?;
        }
        write!(f, "{}", writer)
    }
}

impl Config for ClientConfig {}

impl Display for ClientConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut writer = String::new();

        writeln!(
            writer,
            "Managed addresses : {}",
            self.keystore.addresses().len()
        )?;
        write!(writer, "Active address: ")?;
        match self.active_address {
            Some(r) => writeln!(writer, "{}", r)?,
            None => writeln!(writer, "None")?,
        };
        writeln!(writer, "{}", self.keystore)?;
        write!(writer, "server: ")?;

        if let Ok(env) = self.get_active_env() {
            write!(writer, "{}", env)?;
        }
        match &self.active_env {
            Some(r) => writeln!(writer, "{}", r)?,
            None => writeln!(writer, "None")?,
        };
        write!(f, "{}", writer)
    }
}

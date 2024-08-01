// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeMap;

use crate::cli_types::CommandAction;
use async_trait::async_trait;
use clap::Parser;
use rooch_types::error::{RoochError, RoochResult};
use vergen_pretty::{vergen_pretty_env, PrettyBuilder};

/// Retrieves events based on their event handle.
#[derive(Debug, Parser)]
pub struct Version {
    /// Return command outputs in json format
    #[clap(long, default_value = "false")]
    json: bool,
}

#[async_trait]
impl CommandAction<String> for Version {
    async fn execute(self) -> RoochResult<String> {
        let build_envs = vergen_pretty_env!();

        if self.json {
            let build_envs = build_envs
                .into_iter()
                .filter_map(|(k, v)| v.map(|v| (k, v)))
                .collect::<BTreeMap<_, _>>();
            Ok(serde_json::to_string_pretty(&build_envs)?)
        } else {
            let mut buff = vec![];
            PrettyBuilder::default()
                .env(build_envs)
                .build()
                .map_err(|e| RoochError::UnexpectedError(e.to_string()))?
                .display(&mut buff)
                .map_err(|e| RoochError::UnexpectedError(e.to_string()))?;
            String::from_utf8(buff).map_err(|e| RoochError::UnexpectedError(e.to_string()))
        }
    }
}

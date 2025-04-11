// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// Copyright (c) The Move Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::CommandAction;
use async_trait::async_trait;
use clap::Parser;
use move_cli::{base::reroot_path, Move};
use rooch_types::error::RoochResult;
use serde_json::{json, Value};

/// Print address information.
#[derive(Parser)]
#[clap(name = "info")]
pub struct InfoCommand {
    #[clap(flatten)]
    move_args: Move,

    /// Return command outputs in json format
    #[clap(long, default_value = "false")]
    json: bool,
}

#[async_trait]
impl CommandAction<Option<Value>> for InfoCommand {
    async fn execute(self) -> RoochResult<Option<Value>> {
        let path = self.move_args.package_path;
        let config = self.move_args.build_config;
        let rerooted_path = reroot_path(path)?;

        let resolved_graph =
            config.resolution_graph_for_package(&rerooted_path, &mut std::io::stdout())?;

        if self.json {
            let json_result = json!({ "Result": "Success" });
            Ok(Some(json_result))
        } else {
            println!("{:?}", resolved_graph.graph);
            Ok(None)
        }
    }
}

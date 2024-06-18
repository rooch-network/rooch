// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// Copyright (c) The Move Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::CommandAction;
use crate::commands::move_cli::serialized_success;
use async_trait::async_trait;
use clap::Parser;
use move_cli::{base::coverage::Coverage, Move};
use rooch_types::error::RoochResult;
use serde_json::Value;

/// Inspect test coverage for this package. A previous test run with the `--coverage` flag must
/// have previously been run.
#[derive(Parser)]
#[clap(name = "coverage")]
pub struct CoverageCommand {
    #[clap(flatten)]
    pub coverage: Coverage,

    #[clap(flatten)]
    move_args: Move,

    /// Return command outputs in json format
    #[clap(long, default_value = "false")]
    json: bool,
}

#[async_trait]
impl CommandAction<Option<Value>> for CoverageCommand {
    async fn execute(self) -> RoochResult<Option<Value>> {
        let path = self.move_args.package_path;
        let config = self.move_args.build_config;
        self.coverage.execute(path, config)?;

        serialized_success(self.json)
    }
}

// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// Copyright (c) The Move Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::CommandAction;
use crate::commands::move_cli::print_serialized_success;
use async_trait::async_trait;
use clap::Parser;
use move_cli::{base::errmap::Errmap, base::reroot_path, Move};
use move_package::ModelConfig;
use rooch_types::error::{RoochError, RoochResult};
use serde_json::Value;

/// Generate error map for the package and its dependencies at `path` for use by the Move
/// explanation tool.
#[derive(Parser)]
#[clap(name = "errmap")]
pub struct ErrmapCommand {
    #[clap(flatten)]
    pub errmap: Errmap,

    #[clap(flatten)]
    move_args: Move,

    /// Return command outputs in json format
    #[clap(long, default_value = "false")]
    json: bool,
}

#[async_trait]
impl CommandAction<Option<Value>> for ErrmapCommand {
    async fn execute(self) -> RoochResult<Option<Value>> {
        let error_prefix: Option<String>;
        match self.errmap.error_prefix {
            Some(prefix) => {
                if prefix == "Error" {
                    error_prefix = Some("Error".to_owned());
                } else if prefix == "E" {
                    error_prefix = Some("E".to_owned());
                } else {
                    return Err(RoochError::CommandArgumentError(
                                "Invalid error prefix. Use --error-prefix \"E\" for move-stdlib, --error-prefix \"Error\" for moveos-stdlib and rooch-framework, etc.".to_owned(),
                            ));
                }
            }
            None => {
                return Err(RoochError::CommandArgumentError(
                            "Error prefix not provided. Use --error-prefix \"E\" for move-stdlib, --error-prefix \"Error\" for moveos-stdlib and rooch-framework, etc.".to_owned(),
                        ));
            }
        }

        let path = self.move_args.package_path;
        let config = self.move_args.build_config;

        let rerooted_path = reroot_path(path)?;
        let output_file = self.errmap.output_file;
        let mut errmap_options = move_errmapgen::ErrmapOptions::default();
        if let Some(err_prefix) = error_prefix {
            errmap_options.error_prefix = err_prefix;
        }
        errmap_options.output_file = output_file
            .with_extension(move_command_line_common::files::MOVE_ERROR_DESC_EXTENSION)
            .to_string_lossy()
            .to_string();
        let model = config.move_model_for_package(
            &rerooted_path,
            ModelConfig {
                all_files_as_targets: true,
                target_filter: None,
            },
        )?;
        let mut errmap_gen = move_errmapgen::ErrmapGen::new(&model, &errmap_options);
        errmap_gen.gen();
        errmap_gen.save_result();

        print_serialized_success(self.json)
    }
}

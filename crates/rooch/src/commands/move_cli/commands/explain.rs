// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::{CommandAction, WalletContextOptions};
use crate::commands::move_cli::serialized_success;
use async_trait::async_trait;
use clap::Parser;
use move_cli::Move;
use move_core_types::errmap::{ErrorDescription, ErrorMapping};
use move_core_types::language_storage::ModuleId;
use move_core_types::vm_status::AbortLocation;
use rooch_types::error::{RoochError, RoochResult};
use rooch_types::function_arg::ParsedModuleId;
use serde::{Deserialize, Serialize};
use serde_json::Value;

///Explain Move abort codes. Errors are defined as
///a global category + module-specific reason for the error.
#[derive(Parser)]
#[clap(name = "explain")]
pub struct ExplainCommand {
    #[clap(flatten)]
    context_options: WalletContextOptions,

    /// The location (module id) returned with a `MoveAbort` error
    #[clap(long = "location", short = 'l')]
    location: ParsedModuleId,
    /// The abort code returned with a `MoveAbort` error
    #[clap(long = "abort-code", short = 'a')]
    abort_code: u64,

    #[clap(flatten)]
    move_args: Move,

    /// Return command outputs in json format
    #[clap(long, default_value = "false")]
    json: bool,
}

#[async_trait]
impl CommandAction<Option<Value>> for ExplainCommand {
    async fn execute(self) -> RoochResult<Option<Value>> {
        let context = self.context_options.build()?;
        let address_mapping = context.address_mapping();
        let module_id = self.location.into_module_id(&address_mapping)?;
        let error_descriptions = &framework_release::error_descriptions::ERROR_DESCRIPTIONS;
        let error_mapping = error_descriptions.get(module_id.address());

        match error_mapping {
            Some(error_mapping) => {
                let explain_result = explain_move_abort(
                    AbortLocation::Module(module_id),
                    self.abort_code,
                    error_mapping,
                );
                println!("{}", explain_result)
            }
            None => {
                return Err(RoochError::from(anyhow::Error::msg(
                    "Error map data not found.",
                )));
            }
        }

        serialized_success(self.json)
    }
}

/// Given the module ID and the abort code raised from that module, returns the human-readable
/// explanation of that abort if possible.
pub fn get_explanation(
    module_id: &ModuleId,
    abort_code: u64,
    error_mapping: &ErrorMapping,
) -> Option<ErrorDescription> {
    let module_name = module_id.short_str_lossless();
    error_mapping.get_explanation(&module_name, abort_code)
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub struct MoveAbortExplain {
    pub reason_code: u64,
    pub reason_name: Option<String>,
    pub code_description: Option<String>,
}

impl std::fmt::Display for MoveAbortExplain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Reason Code: {}", self.reason_code)?;
        writeln!(
            f,
            "Reason Name: {}",
            self.reason_name.clone().unwrap_or("Unknown".to_string())
        )?;
        writeln!(
            f,
            "Code Description: {}",
            self.code_description
                .clone()
                .unwrap_or("Unknown".to_string())
        )?;
        Ok(())
    }
}

pub fn explain_move_abort(
    abort_location: AbortLocation,
    abort_code: u64,
    error_mapping: &ErrorMapping,
) -> MoveAbortExplain {
    let err_description = match abort_location {
        AbortLocation::Module(module_id) => get_explanation(&module_id, abort_code, error_mapping),
        AbortLocation::Script => None,
    };
    match err_description {
        Some(description) => MoveAbortExplain {
            reason_code: abort_code,
            reason_name: Some(description.code_name),
            code_description: Some(description.code_description),
        },
        None => MoveAbortExplain {
            reason_code: abort_code,
            reason_name: None,
            code_description: None,
        },
    }
}

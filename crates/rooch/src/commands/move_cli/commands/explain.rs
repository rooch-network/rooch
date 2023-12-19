// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use bcs_ext;
use clap::*;
use move_core_types::errmap::{ErrorDescription, ErrorMapping};
use move_core_types::language_storage::ModuleId;
use move_core_types::vm_status::AbortLocation;
use moveos_types::addresses::MOVEOS_STD_ADDRESS;
use moveos_types::addresses::MOVE_STD_ADDRESS;
use rooch_genesis::{
    move_std_error_descriptions, moveos_std_error_descriptions, rooch_framework_error_descriptions,
};
use rooch_types::addresses::ROOCH_FRAMEWORK_ADDRESS;
use rooch_types::function_arg::ParsedModuleId;
use serde::{Deserialize, Serialize};

use crate::cli_types::WalletContextOptions;

///Explain Move abort codes. Errors are defined as
///a global category + module-specific reason for the error.
#[derive(Parser)]
#[clap(name = "explain")]
pub struct Explain {
    #[clap(flatten)]
    context_options: WalletContextOptions,

    /// The location (module id) returned with a `MoveAbort` error
    #[clap(long = "location", short = 'l')]
    location: ParsedModuleId,
    /// The abort code returned with a `MoveAbort` error
    #[clap(long = "abort-code", short = 'a')]
    abort_code: u64,
}

impl Explain {
    pub async fn execute(self) -> anyhow::Result<()> {
        let context = self.context_options.build()?;
        let address_mapping = context.address_mapping();
        let module_id = self.location.into_module_id(&address_mapping)?;

        let error_description_bytes = {
            match *module_id.address() {
                MOVE_STD_ADDRESS => Some(move_std_error_descriptions()),
                MOVEOS_STD_ADDRESS => Some(moveos_std_error_descriptions()),
                ROOCH_FRAMEWORK_ADDRESS => Some(rooch_framework_error_descriptions()),
                _ => None,
            }
        };

        match error_description_bytes {
            Some(bytes) => {
                let explain_result =
                    explain_move_abort(AbortLocation::Module(module_id), self.abort_code, bytes);
                println!("{}", explain_result)
            }
            None => {
                return Err(anyhow::Error::msg("Error map data not found."));
            }
        }

        Ok(())
    }
}

/// Given the module ID and the abort code raised from that module, returns the human-readable
/// explanation of that abort if possible.
pub fn get_explanation(
    module_id: &ModuleId,
    abort_code: u64,
    data: &[u8],
) -> Option<ErrorDescription> {
    let error_descriptions: ErrorMapping =
        bcs_ext::from_bytes(data).expect("Decode err map failed");
    error_descriptions.get_explanation(module_id.to_string().as_str(), abort_code)
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
    data: &[u8],
) -> MoveAbortExplain {
    let err_description = match abort_location {
        AbortLocation::Module(module_id) => get_explanation(&module_id, abort_code, data),
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

// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use bcs_ext;
use clap::*;
use move_core_types::account_address::AccountAddress;
use move_core_types::errmap::{ErrorDescription, ErrorMapping};
use move_core_types::identifier::Identifier;
use move_core_types::language_storage::ModuleId;
use moveos_stdlib::{move_std_error_descriptions, moveos_std_error_descriptions};
use moveos_types::addresses::MOVEOS_STD_ADDRESS;
use moveos_types::addresses::MOVE_STD_ADDRESS;
use rooch_framework::rooch_framework_error_descriptions;
use rooch_types::addresses::ROOCH_FRAMEWORK_ADDRESS;

///Explain Move abort codes. Errors are defined as
///a global category + module-specific reason for the error.
#[derive(Parser)]
#[clap(name = "explain")]
pub struct Explain {
    /// The location (module id) returned with a `MoveAbort` error
    #[clap(long = "location", short = 'l')]
    location: String,
    /// The abort code returned with a `MoveAbort` error
    #[clap(long = "abort-code", short = 'a')]
    abort_code: u64,
}

impl Explain {
    pub async fn execute(self) -> anyhow::Result<()> {
        let mut location = self.location.split("::");
        let mut address_literal = location.next().expect("Could not find address").to_string();
        let module_name = location
            .next()
            .expect("Could not find module name")
            .to_string();

        if !address_literal.starts_with("0x") {
            address_literal = format!("0x{}", address_literal);
        }

        let module_id = ModuleId::new(
            AccountAddress::from_hex_literal(&address_literal)
                .expect("Unable to parse module address"),
            Identifier::new(module_name).expect("Invalid module name encountered"),
        );

        let error_description_bytes = {
            match *module_id.address() {
                MOVE_STD_ADDRESS => Some(move_std_error_descriptions()),
                MOVEOS_STD_ADDRESS => Some(moveos_std_error_descriptions()),
                ROOCH_FRAMEWORK_ADDRESS => Some(rooch_framework_error_descriptions()),
                _ => None,
            }
        };

        match error_description_bytes {
            Some(bytes) => match get_explanation(&module_id, self.abort_code, bytes) {
                None => println!(
                    "Unable to find a description for {}::{}",
                    self.location, self.abort_code
                ),
                Some(error_desc) => println!(
                    "Category:\n  Name: {}\n  Description: {}",
                    error_desc.code_name, error_desc.code_description,
                ),
            },
            None => {
                println!("Error map data not found.")
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
    error_descriptions.get_explanation(module_id, abort_code)
}

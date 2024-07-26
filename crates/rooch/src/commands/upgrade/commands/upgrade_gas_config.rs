// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use std::{collections::BTreeMap, str::FromStr};

use async_trait::async_trait;
use move_core_types::identifier::Identifier;
use move_core_types::language_storage::ModuleId;
use rpassword::prompt_password;

use framework_types::addresses::ROOCH_FRAMEWORK_ADDRESS;
use moveos_types::state::MoveState;
use moveos_types::transaction::MoveAction;
use moveos_types::{move_types::FunctionId, transaction::FunctionCall};
use rooch_framework::natives::gas_parameter::gas_member::ToOnChainGasSchedule;
use rooch_genesis::{FrameworksGasParameters, LATEST_GAS_SCHEDULE_VERSION};
use rooch_key::key_derive::verify_password;
use rooch_key::keystore::account_keystore::AccountKeystore;
use rooch_rpc_api::jsonrpc_types::{
    AnnotatedMoveStructView, AnnotatedMoveValueView::SpecificStruct,
    AnnotatedMoveValueView::Struct, AnnotatedMoveValueView::Vector, AnnotatedMoveValueView::U64,
};
use rooch_rpc_api::jsonrpc_types::{ExecuteTransactionResponseView, SpecificStructView};
use rooch_types::error::{RoochError, RoochResult};
use rooch_types::transaction::RoochTransaction;

use crate::cli_types::{CommandAction, TransactionOptions, WalletContextOptions};

/// Upgrade the onchain gas config
#[derive(Debug, clap::Parser)]
pub struct UpgradeGasConfigCommand {
    #[clap(flatten)]
    pub(crate) context_options: WalletContextOptions,

    #[clap(flatten)]
    tx_options: TransactionOptions,
}

#[async_trait]
impl CommandAction<ExecuteTransactionResponseView> for UpgradeGasConfigCommand {
    async fn execute(self) -> RoochResult<ExecuteTransactionResponseView> {
        let context = self.context_options.build()?;
        let gas_schedule_function_id = FunctionId::from_str("0x2::gas_schedule::gas_schedule")?;
        let function_call = FunctionCall::new(gas_schedule_function_id, vec![], vec![]);

        let client = context.get_client().await?;
        let gas_schedule_result = client
            .rooch
            .execute_view_function(function_call)
            .await
            .map_err(|e| RoochError::ViewFunctionError(e.to_string()));

        let gas_schedule_opt = match gas_schedule_result {
            Ok(gas_schedule_opt) => match gas_schedule_opt.return_values {
                Some(return_value_vec) => {
                    if !return_value_vec.is_empty() {
                        let return_value = return_value_vec.first().unwrap();
                        let decode_return_value = return_value.decoded_value.clone();
                        Some(decode_return_value)
                    } else {
                        None
                    }
                }
                None => None,
            },
            Err(_) => None,
        };

        let onchain_gas_schedule = match gas_schedule_opt {
            Some(gas_schedule) => match gas_schedule {
                Struct(gas_schedule_struct_) => Some(extract_gas_schedule(gas_schedule_struct_)),
                _ => None,
            },
            None => None,
        };

        let local_latest_gas_parameters = FrameworksGasParameters::latest();
        // let local_gas_config_version = LATEST_GAS_SCHEDULE_VERSION;

        match onchain_gas_schedule {
            None => {
                return Err(RoochError::OnchainGasScheduleIsEmpty);
            }
            Some((onchain_gas_schedule_version, onchain_gas_schedule_map)) => {
                let mut local_gas_entries = local_latest_gas_parameters
                    .vm_gas_params
                    .to_on_chain_gas_schedule();
                local_gas_entries.extend(
                    local_latest_gas_parameters
                        .rooch_framework_gas_params
                        .to_on_chain_gas_schedule(),
                );
                local_gas_entries.extend(
                    local_latest_gas_parameters
                        .bitcoin_move_gas_params
                        .to_on_chain_gas_schedule(),
                );

                if LATEST_GAS_SCHEDULE_VERSION < onchain_gas_schedule_version {
                    return Err(RoochError::InvalidLocalGasVersion(
                        LATEST_GAS_SCHEDULE_VERSION,
                        onchain_gas_schedule_version,
                    ));
                }

                let local_gas_schedule_map: BTreeMap<String, u64> =
                    local_gas_entries.into_iter().collect();

                if local_gas_schedule_map.len() < onchain_gas_schedule_map.len() {
                    return Err(RoochError::LessLocalGasScheduleLength);
                }

                for (gas_key, _) in onchain_gas_schedule_map.iter() {
                    match local_gas_schedule_map.get(gas_key) {
                        None => {
                            return Err(RoochError::LocalIncorrectGasSchedule);
                        }
                        Some(_) => {}
                    }
                }

                let mut modified_gas_entries = Vec::new();
                let mut added_gas_entries = Vec::new();

                for (gas_key, gas_value) in local_gas_schedule_map.iter() {
                    match onchain_gas_schedule_map.get(gas_key) {
                        None => added_gas_entries.push((gas_key.clone(), gas_value.clone())),
                        Some(onchain_gas_value) => {
                            if *onchain_gas_value != *gas_value {
                                modified_gas_entries.push((gas_key.clone(), gas_value.clone()))
                            }
                        }
                    }
                }

                if !added_gas_entries.is_empty() {
                    println!(
                        "Found {:} new gas entries that need to be upgraded:",
                        added_gas_entries.len()
                    );
                    for (gas_key, gas_value) in added_gas_entries.iter() {
                        println!("new gas: {:}, value: {:}", gas_key, gas_value);
                    }
                }

                if !modified_gas_entries.is_empty() {
                    println!(
                        "Found {:} modified gas entries that need to be upgraded:",
                        modified_gas_entries.len()
                    );
                    for (gas_key, gas_value) in modified_gas_entries.iter() {
                        println!("modified gas: {:}, value: {:}", gas_key, gas_value);
                    }
                }

                (onchain_gas_schedule_version, onchain_gas_schedule_map)
            }
        };

        let latest_gas_schedule = local_latest_gas_parameters.to_gas_schedule_config();
        let gas_schedule_bytes = latest_gas_schedule
            .to_move_value()
            .simple_serialize()
            .unwrap();

        let args = vec![bcs::to_bytes(&gas_schedule_bytes).unwrap()];

        let action = MoveAction::new_function_call(
            FunctionId::new(
                ModuleId::new(
                    ROOCH_FRAMEWORK_ADDRESS,
                    Identifier::new("upgrade".to_owned()).unwrap(),
                ),
                Identifier::new("upgrade_gas_schedule".to_owned()).unwrap(),
            ),
            vec![],
            args,
        );

        let sender = context.resolve_address(self.tx_options.sender)?.into();
        let max_gas_amount: Option<u64> = self.tx_options.max_gas_amount;

        match self.tx_options.authenticator {
            Some(authenticator) => {
                let tx_data = context
                    .build_tx_data(sender, action, max_gas_amount)
                    .await?;
                let tx = RoochTransaction::new(tx_data, authenticator.into());
                context.execute(tx).await
            }
            None => {
                if context.keystore.get_if_password_is_empty() {
                    context
                        .sign_and_execute(sender, action, None, max_gas_amount)
                        .await
                } else {
                    let password =
                        prompt_password("Enter the password to publish:").unwrap_or_default();
                    let is_verified = verify_password(
                        Some(password.clone()),
                        context.keystore.get_password_hash(),
                    )?;

                    if !is_verified {
                        return Err(RoochError::InvalidPasswordError(
                            "Password is invalid".to_owned(),
                        ));
                    }

                    context
                        .sign_and_execute(sender, action, Some(password), max_gas_amount)
                        .await
                }
            }
        }
    }
}

fn extract_gas_schedule(
    gas_schedule_struct_: AnnotatedMoveStructView,
) -> (u64, BTreeMap<String, u64>) {
    let AnnotatedMoveStructView {
        abilities: _,
        type_,
        value,
    } = gas_schedule_struct_;

    let struct_name = type_.0.to_string();

    let mut gas_entries_map = BTreeMap::new();
    let mut gas_config_version = 0;

    if struct_name == "0x2::gas_schedule::GasSchedule" {
        let key = Identifier::from_str("schedule_version").unwrap();
        let gas_config_version_value = value.get(&key).unwrap();

        if let U64(u64_val) = gas_config_version_value {
            gas_config_version = u64_val.0;
        }

        let key = Identifier::from_str("entries").unwrap();
        let gas_entries = value.get(&key).unwrap();

        match gas_entries {
            Vector(vector) => {
                for gas_item in vector.iter() {
                    match gas_item {
                        Struct(gas_entry_struct) => {
                            let AnnotatedMoveStructView {
                                abilities: _,
                                type_: _,
                                value,
                            } = gas_entry_struct;

                            let gas_entry_key =
                                value.get(&Identifier::from_str("key").unwrap()).unwrap();
                            let gas_entry_value =
                                value.get(&Identifier::from_str("val").unwrap()).unwrap();

                            if let SpecificStruct(special_struct) = gas_entry_key {
                                if let SpecificStructView::MoveString(move_string) = special_struct
                                {
                                    let gas_key = move_string.to_string();
                                    if let U64(u64_val) = gas_entry_value {
                                        let gas_val = u64_val.0;
                                        gas_entries_map.insert(gas_key, gas_val);
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    (gas_config_version, gas_entries_map)
}

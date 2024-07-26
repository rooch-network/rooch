// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeMap;

use async_trait::async_trait;
use move_core_types::identifier::Identifier;
use move_core_types::language_storage::ModuleId;

use framework_types::addresses::ROOCH_FRAMEWORK_ADDRESS;
use moveos_types::module_binding::MoveFunctionCaller;
use moveos_types::move_types::FunctionId;
use moveos_types::state::MoveState;
use moveos_types::transaction::MoveAction;
use rooch_framework::natives::gas_parameter::gas_member::ToOnChainGasSchedule;
use rooch_genesis::{FrameworksGasParameters, LATEST_GAS_SCHEDULE_VERSION};
use rooch_rpc_api::jsonrpc_types::ExecuteTransactionResponseView;
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
        let context = self.context_options.build_require_password()?;

        let client = context.get_client().await?;
        let gas_schedule_module =
            client.as_module_binding::<moveos_types::moveos_std::gas_schedule::GasScheduleModule>();
        let gas_schedule_opt = gas_schedule_module.gas_schedule();

        let onchain_gas_schedule = match gas_schedule_opt {
            Ok(gas_schedule) => {
                let mut entries_map = BTreeMap::new();
                let _: Vec<_> = gas_schedule
                    .entries
                    .iter()
                    .map(|gas_entry| entries_map.insert(gas_entry.key.to_string(), gas_entry.val))
                    .collect();
                Some((gas_schedule.schedule_version, entries_map))
            }
            _ => None,
        };

        let local_latest_gas_parameters = FrameworksGasParameters::latest();

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
                        None => added_gas_entries.push((gas_key.clone(), *gas_value)),
                        Some(onchain_gas_value) => {
                            if *onchain_gas_value != *gas_value {
                                modified_gas_entries.push((gas_key.clone(), *gas_value))
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
                context
                    .sign_and_execute(sender, action, context.get_password(), max_gas_amount)
                    .await
            }
        }
    }
}

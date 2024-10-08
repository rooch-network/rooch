// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use move_binary_format::binary_views::BinaryIndexedView;
use move_binary_format::errors::VMError;
use move_binary_format::file_format::FunctionDefinitionIndex;
use move_binary_format::CompiledModule;
use move_core_types::language_storage::ModuleId;
use move_core_types::vm_status::KeptVMStatus::Executed;
use move_vm_runtime::data_cache::TransactionCache;
use moveos::gas::table::{
    get_gas_schedule_entries, initial_cost_schedule, CostTable, MoveOSGasMeter,
};
use moveos::moveos::MoveOSConfig;
use moveos::vm::data_cache::MoveosDataCache;
use moveos::vm::moveos_vm::{MoveOSSession, MoveOSVM};
use moveos_common::types::ClassifiedGasMeter;
use moveos_gas_profiling::profiler::{new_gas_profiler, ProfileGasMeter};
use moveos_object_runtime::runtime::ObjectRuntime;
use moveos_types::h256::H256;
use moveos_types::move_std::option::MoveOption;
use moveos_types::moveos_std::gas_schedule::GasScheduleConfig;
use moveos_types::moveos_std::object::ObjectMeta;
use moveos_types::moveos_std::tx_context::TxContext;
use moveos_types::transaction::{
    MoveAction, RawTransactionOutput, VMErrorInfo, VerifiedMoveAction, VerifiedMoveOSTransaction,
};
use parking_lot::RwLock;
use rooch_genesis::FrameworksGasParameters;
use rooch_rpc_api::jsonrpc_types::{
    DryRunTransactionResponseView, KeptVMStatusView, RawTransactionOutputView, StrView,
};
use rooch_rpc_client::{Client, ClientResolver};
use rooch_types::address::{BitcoinAddress, MultiChainAddress};
use rooch_types::framework::auth_validator::{BuiltinAuthValidator, TxValidateResult};
use rooch_types::framework::system_pre_execute_functions;
use rooch_types::transaction::RoochTransactionData;
use std::rc::Rc;
use std::str::FromStr;

pub fn execute_tx_locally(
    state_root_bytes: Vec<u8>,
    client: Client,
    tx: RoochTransactionData,
) -> anyhow::Result<(TxContext, RawTransactionOutput, Option<VMErrorInfo>)> {
    let state_root = H256::from_slice(state_root_bytes.as_slice());
    let root_object_meta = ObjectMeta::root_metadata(state_root, 0);
    let client_resolver = ClientResolver::new(client, root_object_meta.clone());

    let (move_mv, object_runtime, client_resolver, action, cost_table) =
        prepare_execute_env(root_object_meta, &client_resolver, tx.clone());

    let mut gas_meter =
        MoveOSGasMeter::new(cost_table, GasScheduleConfig::CLI_DEFAULT_MAX_GAS_AMOUNT);
    gas_meter.charge_io_write(tx.tx_size()).unwrap();

    let mut moveos_session = MoveOSSession::new(
        move_mv.inner(),
        client_resolver,
        object_runtime,
        gas_meter,
        false,
    );

    let system_pre_execute_functions = system_pre_execute_functions();

    moveos_session
        .execute_function_call(system_pre_execute_functions, false)
        .expect("system_pre_execute_functions execution failed");

    match moveos_session.execute_move_action(action) {
        Ok(_) => {
            let (tx_context, raw_tx_output) = moveos_session
                .finish_with_extensions(Executed)
                .expect("finish_with_extensions failed");
            Ok((tx_context, raw_tx_output, None))
        }
        Err(vm_err) => {
            let error_status_code = vm_err.clone().into_vm_status().keep_or_discard();
            match error_status_code {
                Ok(kept_status) => {
                    let vm_error_opt = Some(VMErrorInfo {
                        error_message: vm_err.to_string(),
                        execution_state: extract_execution_state(
                            vm_err.clone(),
                            &moveos_session.runtime_session().data_cache,
                        )
                        .expect("extract_execution_state failed"),
                    });
                    let (tx_context, raw_tx_output) = moveos_session
                        .finish_with_extensions(kept_status)
                        .expect("finish_with_extensions failed");
                    Ok((tx_context, raw_tx_output, vm_error_opt))
                }
                Err(discarded_status) => {
                    panic!("execute_tx_locally panic {:?}", discarded_status)
                }
            }
        }
    }
}

pub fn execute_tx_locally_with_gas_profile(
    state_root_bytes: Vec<u8>,
    client: Client,
    tx: RoochTransactionData,
) -> anyhow::Result<(TxContext, RawTransactionOutput, Option<VMErrorInfo>)> {
    let state_root = H256::from_slice(state_root_bytes.as_slice());
    let root_object_meta = ObjectMeta::root_metadata(state_root, 0);
    let client_resolver = ClientResolver::new(client, root_object_meta.clone());

    let (move_mv, object_runtime, client_resolver, action, cost_table) =
        prepare_execute_env(root_object_meta, &client_resolver, tx.clone());

    let mut gas_meter =
        MoveOSGasMeter::new(cost_table, GasScheduleConfig::CLI_DEFAULT_MAX_GAS_AMOUNT);
    gas_meter.charge_io_write(tx.tx_size()).unwrap();

    let mut gas_profiler = new_gas_profiler(tx.clone().action, gas_meter);

    let mut moveos_session = MoveOSSession::new(
        move_mv.inner(),
        client_resolver,
        object_runtime,
        gas_profiler.clone(),
        false,
    );

    let system_pre_execute_functions = system_pre_execute_functions();

    moveos_session
        .execute_function_call(system_pre_execute_functions, false)
        .expect("system_pre_execute_functions execution failed");

    let result = match moveos_session.execute_move_action(action) {
        Ok(_) => {
            let (tx_context, raw_tx_output) = moveos_session
                .finish_with_extensions(Executed)
                .expect("finish_with_extensions failed");
            Ok((tx_context, raw_tx_output, None))
        }
        Err(vm_err) => {
            let error_status_code = vm_err.clone().into_vm_status().keep_or_discard();
            match error_status_code {
                Ok(kept_status) => {
                    let vm_error_opt = Some(VMErrorInfo {
                        error_message: vm_err.to_string(),
                        execution_state: extract_execution_state(
                            vm_err.clone(),
                            &moveos_session.runtime_session().data_cache,
                        )
                        .expect("extract_execution_state failed"),
                    });
                    let (tx_context, raw_tx_output) = moveos_session
                        .finish_with_extensions(kept_status)
                        .expect("finish_with_extensions failed");
                    Ok((tx_context, raw_tx_output, vm_error_opt))
                }
                Err(discarded_status) => {
                    panic!("execute_tx_locally panic {:?}", discarded_status)
                }
            }
        }
    };

    let gas_profiling_info = gas_profiler.finish();

    gas_profiling_info
        .generate_html_report(
            format!("./gas_profiling_{:?}", tx.tx_hash()),
            "Rooch Gas Profiling".to_string(),
        )
        .unwrap();

    result
}

pub fn prepare_execute_env(
    state_root: ObjectMeta,
    client_resolver: &ClientResolver,
    tx: RoochTransactionData,
) -> (
    MoveOSVM,
    Rc<RwLock<ObjectRuntime>>,
    &ClientResolver,
    VerifiedMoveAction,
    CostTable,
) {
    let gas_entries =
        get_gas_schedule_entries(client_resolver).expect("get_gas_schedule_entries failed");
    let cost_table = initial_cost_schedule(gas_entries);

    let verified_tx =
        convert_to_verified_tx(state_root.clone(), tx).expect("convert_to_verified_tx failed");

    let VerifiedMoveOSTransaction {
        root: _,
        ctx,
        action,
    } = verified_tx;

    let gas_parameters =
        FrameworksGasParameters::load_from_chain(client_resolver).expect("load_from_chain failed");

    let object_runtime = Rc::new(RwLock::new(ObjectRuntime::new(
        ctx,
        state_root,
        client_resolver,
    )));

    let vm = MoveOSVM::new(
        gas_parameters.all_natives(),
        MoveOSConfig::default().vm_config,
    )
    .expect("create MoveVM failed");

    (vm, object_runtime, client_resolver, action, cost_table)
}

fn convert_to_verified_tx(
    root: ObjectMeta,
    tx_data: RoochTransactionData,
) -> anyhow::Result<VerifiedMoveOSTransaction> {
    let mut tx_ctx = TxContext::new(
        tx_data.sender.into(),
        tx_data.sequence_number,
        tx_data.max_gas_amount,
        tx_data.tx_hash(),
        tx_data.tx_size(),
    );

    let mut bitcoin_address = BitcoinAddress::from_str("18cBEMRxXHqzWWCxZNtU91F5sbUNKhL5PX")?;

    let user_multi_chain_address: MultiChainAddress = tx_data.sender.into();
    if user_multi_chain_address.is_bitcoin_address() {
        bitcoin_address = user_multi_chain_address.try_into()?;
    }

    let dummy_result = TxValidateResult {
        auth_validator_id: BuiltinAuthValidator::Bitcoin.flag().into(),
        auth_validator: MoveOption::none(),
        session_key: MoveOption::none(),
        bitcoin_address,
    };

    tx_ctx.add(dummy_result)?;

    let verified_action = match tx_data.action {
        MoveAction::Script(script_call) => VerifiedMoveAction::Script { call: script_call },
        MoveAction::Function(function_call) => VerifiedMoveAction::Function {
            call: function_call,
            bypass_visibility: false,
        },
        MoveAction::ModuleBundle(module_bundle) => VerifiedMoveAction::ModuleBundle {
            module_bundle,
            init_function_modules: vec![],
        },
    };

    Ok(VerifiedMoveOSTransaction::new(
        root,
        tx_ctx,
        verified_action,
    ))
}

pub async fn dry_run_tx_locally(
    client: Client,
    tx: RoochTransactionData,
) -> anyhow::Result<Option<DryRunTransactionResponseView>> {
    let state_root = get_latest_state_root(&client).await?;
    let (_, raw_transaction_output, error_info_opt) = execute_tx_locally(state_root, client, tx)?;

    match error_info_opt {
        Some(vm_error_info) => {
            let raw_output_view = RawTransactionOutputView {
                status: KeptVMStatusView::from(raw_transaction_output.status),
                gas_used: StrView::from(raw_transaction_output.gas_used),
                is_upgrade: false,
            };
            Ok(Some(DryRunTransactionResponseView {
                raw_output: raw_output_view,
                vm_error_info,
            }))
        }
        None => Ok(None),
    }
}

async fn get_latest_state_root(client: &Client) -> anyhow::Result<Vec<u8>> {
    let status = client.rooch.status().await?;
    Ok(status
        .rooch_status
        .root_state
        .state_root
        .0
        .as_bytes()
        .to_vec())
}

fn extract_execution_state(
    vm_err: VMError,
    module_resolver: &MoveosDataCache<ClientResolver>,
) -> anyhow::Result<Vec<String>> {
    let mut execution_stack_trace = Vec::new();
    if let Some(exec_state) = vm_err.exec_state() {
        for execute_record in exec_state.stack_trace() {
            match execute_record {
                (Some(module_id), func_idx, code_offset) => {
                    let func_name = func_name_from_db(module_id, func_idx, module_resolver)?;
                    execution_stack_trace.push(format!(
                        "{}::{}.{}",
                        module_id.short_str_lossless(),
                        func_name,
                        code_offset
                    ));
                }
                (None, func_idx, code_offset) => {
                    execution_stack_trace.push(format!("{}::{}", func_idx, code_offset));
                }
            }
        }
    };

    Ok(execution_stack_trace)
}

fn func_name_from_db(
    module_id: &ModuleId,
    func_idx: &FunctionDefinitionIndex,
    module_resolver: &MoveosDataCache<ClientResolver>,
) -> anyhow::Result<String> {
    let module_bytes = module_resolver.load_module(module_id)?;
    let compiled_module = CompiledModule::deserialize(module_bytes.as_slice())?;
    let module_bin_view = BinaryIndexedView::Module(&compiled_module);
    let func_def = module_bin_view.function_def_at(*func_idx)?;
    Ok(module_bin_view
        .identifier_at(module_bin_view.function_handle_at(func_def.function).name)
        .to_string())
}

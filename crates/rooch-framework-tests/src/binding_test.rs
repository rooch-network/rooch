// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::{bail, Result};
use lazy_static::lazy_static;
use move_core_types::account_address::AccountAddress;
use move_core_types::vm_status::KeptVMStatus;
use moveos_config::DataDirPath;
use moveos_store::MoveOSStore;
use moveos_types::function_return_value::FunctionResult;
use moveos_types::module_binding::MoveFunctionCaller;
use moveos_types::moveos_std::object::ObjectEntity;
use moveos_types::moveos_std::tx_context::TxContext;
use moveos_types::transaction::{FunctionCall, VerifiedMoveOSTransaction};
use rooch_config::store_config::StoreConfig;
use rooch_executor::actor::reader_executor::ReaderExecutorActor;
use rooch_executor::actor::{executor::ExecutorActor, messages::ExecuteTransactionResult};
use rooch_framework::natives::default_gas_schedule;
use rooch_store::RoochStore;
use rooch_types::bitcoin::genesis::BitcoinGenesisContext;
use rooch_types::bitcoin::network::Network;
use rooch_types::{chain_id::RoochChainID, transaction::AbstractTransaction};
use std::env;
use std::sync::Arc;
use tempfile::TempDir;

lazy_static! {
    pub static ref DATA_DIR: DataDirPath = get_data_dir();
}

pub fn get_data_dir() -> DataDirPath {
    match env::var("ROOCH_TEST_DATA_DIR") {
        Ok(path_str) => {
            let temp_dir = TempDir::new_in(path_str)
                .expect("failed to create temp dir in provided data dir path");
            DataDirPath::TempPath(Arc::from(temp_dir))
        }
        Err(_) => moveos_config::temp_dir(),
    }
}

pub struct RustBindingTest {
    pub executor: ExecutorActor,
    pub reader_executor: ReaderExecutorActor,
}

impl RustBindingTest {
    pub fn new() -> Result<Self> {
        let (rooch_db_path, moveos_db_path) = (
            StoreConfig::get_mock_moveos_store_dir(&DATA_DIR),
            StoreConfig::get_mock_rooch_store_dir(&DATA_DIR),
        );
        if !rooch_db_path.exists() {
            std::fs::create_dir_all(rooch_db_path.clone())?;
        }
        if !moveos_db_path.exists() {
            std::fs::create_dir_all(moveos_db_path.clone())?;
        }

        let moveos_store = MoveOSStore::mock_moveos_store_with_data_dir(moveos_db_path.as_path())?;
        let rooch_store = RoochStore::mock_rooch_store(rooch_db_path.as_path())?;
        let sequencer = AccountAddress::ONE.into();
        let gas_schedule_blob = bcs::to_bytes(&default_gas_schedule())
            .expect("Failure serializing genesis gas schedule");
        let executor = ExecutorActor::new(
            RoochChainID::LOCAL.genesis_ctx(sequencer, gas_schedule_blob),
            BitcoinGenesisContext::new(Network::default().to_num()),
            moveos_store,
            rooch_store,
        )?;

        let reader_executor = ReaderExecutorActor::new(
            executor.genesis().clone(),
            executor.get_moveos_store(),
            executor.get_rooch_store(),
        )?;
        Ok(Self {
            executor,
            reader_executor,
        })
    }

    pub fn executor(&self) -> &ExecutorActor {
        &self.executor
    }

    //TODO let the module bundle to execute the function
    pub fn execute<T: AbstractTransaction>(&mut self, tx: T) -> Result<()> {
        let execute_result = self.execute_as_result(tx)?;
        if execute_result.transaction_info.status != KeptVMStatus::Executed {
            bail!(
                "tx should success, error: {:?}",
                execute_result.transaction_info.status
            );
        }
        Ok(())
    }

    pub fn execute_as_result<T: AbstractTransaction>(
        &mut self,
        tx: T,
    ) -> Result<ExecuteTransactionResult> {
        let verified_tx = self.executor.validate(tx)?;
        let result = self.executor.execute(verified_tx)?;
        let root = ObjectEntity::root_object(
            result.transaction_info.state_root,
            result.transaction_info.size,
        );
        self.reader_executor.refresh_state(root, false)?;
        Ok(result)
    }

    pub fn execute_verified_tx(&mut self, tx: VerifiedMoveOSTransaction) -> Result<()> {
        let execute_result = self.execute_verified_tx_as_result(tx)?;
        if execute_result.transaction_info.status != KeptVMStatus::Executed {
            bail!(
                "tx should success, error: {:?}",
                execute_result.transaction_info.status
            );
        }
        Ok(())
    }

    pub fn execute_verified_tx_as_result(
        &mut self,
        tx: VerifiedMoveOSTransaction,
    ) -> Result<ExecuteTransactionResult> {
        let result = self.executor.execute(tx)?;
        let root = ObjectEntity::root_object(
            result.transaction_info.state_root,
            result.transaction_info.size,
        );
        self.reader_executor.refresh_state(root, false)?;
        Ok(result)
    }
}

impl MoveFunctionCaller for RustBindingTest {
    fn call_function(
        &self,
        ctx: &TxContext,
        function_call: FunctionCall,
    ) -> Result<FunctionResult> {
        let result = self
            .reader_executor
            .moveos()
            .execute_readonly_function(ctx, function_call);
        Ok(result)
    }
}

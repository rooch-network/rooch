// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::{bail, Result};
use move_core_types::vm_status::KeptVMStatus;
use moveos_store::MoveOSStore;
use moveos_types::function_return_value::FunctionResult;
use moveos_types::module_binding::MoveFunctionCaller;
use moveos_types::moveos_std::tx_context::TxContext;
use moveos_types::transaction::FunctionCall;
use rooch_executor::actor::reader_executor::ReaderExecutorActor;
use rooch_executor::actor::{executor::ExecutorActor, messages::ExecuteTransactionResult};
use rooch_store::RoochStore;
use rooch_types::bitcoin::genesis::BitcoinGenesisContext;
use rooch_types::bitcoin::network::Network;
use rooch_types::{
    address::{RoochAddress, RoochSupportedAddress},
    chain_id::RoochChainID,
    transaction::AbstractTransaction,
};

pub struct RustBindingTest {
    pub executor: ExecutorActor,
    pub reader_executor: ReaderExecutorActor,
}

impl RustBindingTest {
    pub fn new() -> Result<Self> {
        let moveos_store = MoveOSStore::mock_moveos_store()?;
        let rooch_store = RoochStore::mock_rooch_store()?;
        let sequencer = RoochAddress::random();
        let executor = ExecutorActor::new(
            RoochChainID::LOCAL.genesis_ctx(sequencer),
            BitcoinGenesisContext::new(Network::default().to_num()),
            moveos_store.clone(),
            rooch_store.clone(),
        )?;
        let reader_executor =
            ReaderExecutorActor::new(executor.genesis().clone(), moveos_store, rooch_store)?;
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
        let verified_tx = self.reader_executor.validate(tx)?;
        self.executor.execute(verified_tx)
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

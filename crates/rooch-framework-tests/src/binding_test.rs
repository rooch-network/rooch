// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::{bail, Result};
use move_core_types::vm_status::KeptVMStatus;
use moveos_store::MoveOSStore;
use moveos_types::module_binding::{ModuleBinding, MoveFunctionCaller};
use rooch_executor::actor::{executor::ExecutorActor, messages::ExecuteTransactionResult};
use rooch_store::RoochStore;
use rooch_types::{
    address::{RoochAddress, RoochSupportedAddress},
    chain_id::RoochChainID,
    transaction::AbstractTransaction,
};

pub struct RustBindingTest {
    executor: ExecutorActor,
}

impl RustBindingTest {
    pub fn new() -> Result<Self> {
        let moveos_store = MoveOSStore::mock_moveos_store()?;
        let rooch_store = RoochStore::mock_rooch_store()?;
        let sequencer = RoochAddress::random();
        let executor = ExecutorActor::new(
            RoochChainID::LOCAL.genesis_ctx(sequencer),
            moveos_store,
            rooch_store,
        )?;
        Ok(Self { executor })
    }

    pub fn executor(&self) -> &ExecutorActor {
        &self.executor
    }

    pub fn as_module_bundle<'a, M: ModuleBinding<'a>>(&'a self) -> M {
        self.executor.moveos().as_module_binding::<M>()
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
        self.executor.execute(verified_tx)
    }
}

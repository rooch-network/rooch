// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::{bail, Result};
use move_core_types::vm_status::KeptVMStatus;
use moveos_store::MoveOSStore;
use moveos_types::module_binding::{ModuleBundle, MoveFunctionCaller};
use rooch_executor::actor::executor::ExecutorActor;
use rooch_store::RoochStore;
use rooch_types::transaction::AbstractTransaction;

pub struct RustBindingTest {
    executor: ExecutorActor,
}

impl RustBindingTest {
    pub fn new() -> Result<Self> {
        let moveos_store = MoveOSStore::mock_moveos_store()?;
        let rooch_store = RoochStore::mock_rooch_store();
        let executor = ExecutorActor::new(moveos_store, rooch_store)?;
        Ok(Self { executor })
    }

    pub fn as_module_bundle<'a, M: ModuleBundle<'a>>(&'a self) -> M {
        self.executor.moveos().as_module_bundle::<M>()
    }

    //TODO let the module bundle to execute the function
    pub fn execute<T: AbstractTransaction>(&mut self, tx: T) -> Result<()> {
        let verified_tx = self.executor.validate(tx)?;
        let execute_result = self.executor.execute(verified_tx)?;
        if execute_result.transaction_info.status != KeptVMStatus::Executed {
            bail!(
                "tx should success, error: {:?}",
                execute_result.transaction_info.status
            );
        }
        Ok(())
    }
}

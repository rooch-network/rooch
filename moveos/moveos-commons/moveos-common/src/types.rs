// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use move_binary_format::errors::PartialVMResult;
use move_core_types::gas_algebra::InternalGas;
use move_vm_types::gas::{GasMeter, UnmeteredGasMeter};

#[derive(Debug, Clone)]
pub struct GasStatement {
    pub execution_gas_used: InternalGas,
    pub storage_gas_used: InternalGas,
}

pub trait ClassifiedGasMeter {
    fn charge_execution(&mut self, gas_cost: u64) -> PartialVMResult<()>;
    // fn charge_io_read(&mut self);
    fn charge_io_write(&mut self, data_size: u64) -> PartialVMResult<()>;
    //fn charge_event(&mut self, events: &[TransactionEvent]) -> PartialVMResult<()>;
    //fn charge_change_set(&mut self, change_set: &StateChangeSet) -> PartialVMResult<()>;
    fn check_constrains(&self, max_gas_amount: u64) -> PartialVMResult<()>;
    fn gas_statement(&self) -> GasStatement;
}

impl ClassifiedGasMeter for UnmeteredGasMeter {
    fn charge_execution(&mut self, _gas_cost: u64) -> PartialVMResult<()> {
        Ok(())
    }

    fn charge_io_write(&mut self, _data_size: u64) -> PartialVMResult<()> {
        Ok(())
    }

    fn check_constrains(&self, _max_gas_amount: u64) -> PartialVMResult<()> {
        Ok(())
    }

    fn gas_statement(&self) -> GasStatement {
        GasStatement {
            execution_gas_used: InternalGas::from(0),
            storage_gas_used: InternalGas::from(0),
        }
    }
}

pub trait SwitchableGasMeter: GasMeter {
    fn stop_metering(&mut self);
    fn start_metering(&mut self);
    fn is_metering(&self) -> bool;
}

impl SwitchableGasMeter for UnmeteredGasMeter {
    fn stop_metering(&mut self) {}

    fn start_metering(&mut self) {}

    fn is_metering(&self) -> bool {
        false
    }
}

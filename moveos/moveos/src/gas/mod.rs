// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use move_vm_types::gas::{GasMeter, UnmeteredGasMeter};

mod gas_member;
mod native;
pub mod parameter;
mod parameters;
pub mod table;
mod r#abstract;

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

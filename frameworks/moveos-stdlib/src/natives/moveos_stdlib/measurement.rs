use crate::natives::helpers::{make_module_natives, make_native};
use log::info;
use move_binary_format::errors::PartialVMResult;
use move_core_types::gas_algebra::InternalGas;
use move_vm_runtime::native_functions::{NativeContext, NativeFunction};
use move_vm_types::loaded_data::runtime_types::Type;
use move_vm_types::natives::function::NativeResult;
use move_vm_types::pop_arg;
use move_vm_types::values::Value;
use smallvec::smallvec;
use std::collections::VecDeque;
use std::time::UNIX_EPOCH;

#[derive(Debug, Clone)]
pub struct MeasurementGas {
    pub base: InternalGas,
}

impl MeasurementGas {
    pub fn zero() -> Self {
        Self { base: 0.into() }
    }
}

fn inject_parameter(
    _gas_params: &MeasurementGas,
    context: &mut NativeContext,
    _ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    use std::time::SystemTime;

    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");

    let full_func_name_bytes = pop_arg!(args, Vec<u8>);
    let full_func_name = String::from_utf8_lossy(full_func_name_bytes.as_slice()).to_string();
    let timestamp = pop_arg!(args, u64);
    let gas_balance = pop_arg!(args, u64);

    let current_gas_balance = context.gas_balance();
    let current_timestamp_millis = since_the_epoch.as_millis() as u64;
    if timestamp > 0 && gas_balance > 0 {
        let gas_used_p1 = InternalGas::from(gas_balance)
            .checked_sub(current_gas_balance)
            .unwrap();

        if !gas_used_p1.is_zero() {
            if let Some(gas_used_p2) = gas_used_p1.checked_sub(14620.into()) {
                let time_used: u64 = current_timestamp_millis - timestamp;
                info!(
                    "gas_used: {:}, time_used {:?} -> {:?}",
                    gas_used_p2, time_used, full_func_name
                );
            }
        }
        Ok(NativeResult::Success {
            cost: InternalGas::zero(),
            ret_vals: smallvec![Value::u64(0), Value::u64(0)],
        })
    } else {
        Ok(NativeResult::Success {
            cost: InternalGas::zero(),
            ret_vals: smallvec![
                Value::u64(current_gas_balance.into()),
                Value::u64(current_timestamp_millis)
            ],
        })
    }
}

pub fn make_all(gas_params: MeasurementGas) -> impl Iterator<Item = (String, NativeFunction)> {
    let natives = [(
        "inject_parameter",
        make_native(gas_params, inject_parameter),
    )];

    make_module_natives(natives)
}

// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use std::fmt;
use std::sync::{Arc, Mutex};
use wasmer::wasmparser::Operator;
use wasmer::{
    LocalFunctionIndex, MiddlewareError, MiddlewareReaderState, ModuleMiddleware,
    RuntimeError
};
use wasmer_types::FunctionIndex;
use tracing::debug;

#[derive(Debug)]
pub struct GasMeter {
    gas_limit: u64,
    gas_used: u64,
    charge_function_index: Option<FunctionIndex>
}

impl GasMeter {
    pub fn new(gas_limit: u64) -> Self {
        Self {
            gas_limit,
            gas_used: 0,
            charge_function_index: None,
        }
    }

    pub fn charge(&mut self, amount: u64) -> Result<(), RuntimeError> {
        if self.gas_used + amount > self.gas_limit {
            Err(RuntimeError::new("GAS limit exceeded"))
        } else {
            self.gas_used += amount;
            Ok(())
        }
    }
}

fn default_cost_function(operator: &Operator) -> u64 {
    match operator {
        Operator::LocalGet { .. } => 1,
        Operator::LocalSet { .. } => 1,
        Operator::I32Const { .. } => 1,
        Operator::I32Add { .. } => 2,
        Operator::I32Sub { .. } => 2,
        Operator::I32Mul { .. } => 3,
        Operator::Call { .. } => 5,
        Operator::Loop { .. } => 1,
        Operator::If { .. } => 1,
        Operator::Else { .. } => 1,
        Operator::End { .. } => 1,
        _ => 0,
    }
}

pub struct GasMiddleware {
    gas_meter: Arc<Mutex<GasMeter>>,
    cost_function: Arc<dyn Fn(&Operator) -> u64 + Send + Sync>,
}

impl GasMiddleware {
    pub fn new(gas_meter: Arc<Mutex<GasMeter>>, cost_function: Option<Arc<dyn Fn(&Operator) -> u64 + Send + Sync>>) -> Self {
        Self {
            gas_meter: gas_meter.clone(),
            cost_function: cost_function.unwrap_or_else(|| Arc::new(default_cost_function)),
        }
    }
}

impl fmt::Debug for GasMiddleware {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GasMiddleware")
            .field("gas_meter", &self.gas_meter)
            .finish()
    }
}

impl ModuleMiddleware for GasMiddleware {
    fn generate_function_middleware(
        &self,
        _index: LocalFunctionIndex,
    ) -> Box<dyn wasmer::FunctionMiddleware> {
        Box::new(GasFunctionMiddleware {
            gas_meter: self.gas_meter.clone(),
            accumulated_cost: 0,
            cost_function: self.cost_function.clone(),
        })
    }

    fn transform_module_info(&self, module_info: &mut wasmer_types::ModuleInfo) {
        // Get the index of the charge function
        if let Some(index) = module_info.imports.iter().find_map(|(name, import_index)| {
            if name.module == "env" && name.field == "charge" {
                if let wasmer_types::ImportIndex::Function(index) = import_index {
                    return Some(*index);
                }
            }
            
            None
        }) {
            debug!("transform_module_info -> charge_func_index:{:?}", &index);
            let mut gas_meter = self.gas_meter.lock().unwrap();
            gas_meter.charge_function_index = Some(index);
        } else {
            panic!("charge function not found in imports");
        }
    }
}

struct GasFunctionMiddleware {
    gas_meter: Arc<Mutex<GasMeter>>,
    accumulated_cost: u64,
    cost_function: Arc<dyn Fn(&Operator) -> u64 + Send + Sync>,
}

impl fmt::Debug for GasFunctionMiddleware {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GasFunctionMiddleware")
            .field("gas_meter", &self.gas_meter)
            .field("accumulated_cost", &self.accumulated_cost)
            .finish()
    }
}

impl wasmer::FunctionMiddleware for GasFunctionMiddleware {
    fn feed<'a>(
        &mut self,
        operator: Operator<'a>,
        state: &mut MiddlewareReaderState<'a>,
    ) -> Result<(), MiddlewareError> {
        debug!("feed: op: {:?}", &operator);

        // Use cost_function to evaluate the cost of the instruction
        self.accumulated_cost += (self.cost_function)(&operator);

        // Perform batch charging at critical points
        match operator {
            Operator::Loop { .. }
            | Operator::End
            | Operator::Else
            | Operator::Br { .. }
            | Operator::BrTable { .. }
            | Operator::BrIf { .. }
            | Operator::Call { .. }
            | Operator::CallIndirect { .. }
            | Operator::Return => {
                if self.accumulated_cost > 0 {
                    let gas_meter = self.gas_meter.lock().unwrap();

                    state.extend(&[
                        Operator::I32Const { value: self.accumulated_cost as i32 },
                        Operator::Call { function_index: gas_meter.charge_function_index.unwrap().as_u32() },
                    ]);

                    self.accumulated_cost = 0;
                }
            }
            _ => {}
        }
        state.push_operator(operator);

        Ok(())
    }
}

// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use std::fmt;
use std::sync::{Arc, Mutex};
use tracing::debug;
use wasmer::wasmparser::Operator;
use wasmer::{
    LocalFunctionIndex, MiddlewareError, MiddlewareReaderState, ModuleMiddleware, RuntimeError,
    Type,
};
use wasmer_types::{
    entity::PrimaryMap, ExportIndex, FunctionIndex, FunctionType, ImportIndex, ImportKey,
};

type CostFunction = dyn Fn(&Operator) -> u64 + Send + Sync;

#[derive(Debug)]
pub struct GasMeter {
    gas_limit: u64,
    gas_used: u64,
    charge_function_index: Option<FunctionIndex>,
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
    cost_function: Arc<CostFunction>,
}

impl GasMiddleware {
    pub fn new(gas_meter: Arc<Mutex<GasMeter>>, cost_function: Option<Arc<CostFunction>>) -> Self {
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
        // Insert the signature for the charge function
        let charge_signature = FunctionType::new(vec![Type::I64], vec![]);
        let charge_signature_index = module_info.signatures.push(charge_signature);

        // Insert the charge function after existing imported functions
        let charge_function_index =
            FunctionIndex::from_u32(module_info.num_imported_functions as u32);

        // Insert the charge function name
        module_info
            .function_names
            .insert(charge_function_index, "charge".to_string());

        // Insert the charge function import declaration
        let charge_import_key = ImportKey {
            module: "env".to_string(),
            field: "charge".to_string(),
            import_idx: charge_function_index.as_u32(),
        };

        // Insert the charge function import declaration at the end
        module_info.imports.insert(
            charge_import_key,
            ImportIndex::Function(charge_function_index),
        );

        // Adjust the function indices to make room for the charge function after imported functions
        let mut new_functions = PrimaryMap::with_capacity(module_info.functions.len() + 1);
        for (index, sig_index) in module_info.functions.iter() {
            if index.as_u32() == module_info.num_imported_functions as u32 {
                new_functions.push(charge_signature_index);
            }
            new_functions.push(*sig_index);
        }
        if module_info.functions.len() == module_info.num_imported_functions {
            new_functions.push(charge_signature_index);
        }
        module_info.functions = new_functions;

        // Update all function references to account for the new function
        if let Some(start_function) = module_info.start_function {
            if start_function.as_u32() >= charge_function_index.as_u32() {
                module_info.start_function =
                    Some(FunctionIndex::from_u32(start_function.as_u32() + 1));
            }
        }

        for (_, elem_indices) in module_info.passive_elements.iter_mut() {
            for elem_index in elem_indices.iter_mut() {
                if elem_index.as_u32() >= charge_function_index.as_u32() {
                    *elem_index = FunctionIndex::from_u32(elem_index.as_u32() + 1);
                }
            }
        }
        for (_, export_index) in module_info.exports.iter_mut() {
            if let ExportIndex::Function(func_index) = export_index {
                if func_index.as_u32() >= charge_function_index.as_u32() {
                    *func_index = FunctionIndex::from_u32(func_index.as_u32() + 1);
                }
            }
        }

        for table_initializer in module_info.table_initializers.iter_mut() {
            for func_index in table_initializer.elements.iter_mut() {
                if func_index.as_u32() >= charge_function_index.as_u32() {
                    *func_index = FunctionIndex::from_u32(func_index.as_u32() + 1);
                }
            }
        }

        module_info.num_imported_functions += 1;

        let mut gas_meter = self.gas_meter.lock().unwrap();
        gas_meter.charge_function_index = Some(charge_function_index);

        debug!("transform_module_info: {:?}", &module_info);
    }
}

struct GasFunctionMiddleware {
    gas_meter: Arc<Mutex<GasMeter>>,
    accumulated_cost: u64,
    cost_function: Arc<CostFunction>,
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
        // Use cost_function to evaluate the cost of the instruction
        self.accumulated_cost += (self.cost_function)(&operator);

        // Perform batch charging before critical points
        match operator {
            Operator::End
            | Operator::Br { .. }
            | Operator::BrTable { .. }
            | Operator::Call { .. }
            | Operator::CallIndirect { .. }
            | Operator::Return => {
                if self.accumulated_cost > 0 {
                    let gas_meter = self.gas_meter.lock().unwrap();

                    state.extend(&[
                        Operator::I64Const {
                            value: self.accumulated_cost as i64,
                        },
                        Operator::Call {
                            function_index: gas_meter.charge_function_index.unwrap().as_u32(),
                        },
                    ]);

                    self.accumulated_cost = 0;
                }
            }
            _ => {}
        }

        // Update function call indices if necessary
        match operator {
            Operator::Call { function_index } => {
                let gas_meter = self.gas_meter.lock().unwrap();
                if function_index >= gas_meter.charge_function_index.unwrap().as_u32() {
                    state.push_operator(Operator::Call {
                        function_index: function_index + 1,
                    });
                } else {
                    state.push_operator(operator.clone());
                }
            }
            Operator::CallIndirect {
                table_index,
                type_index,
                table_byte,
            } => {
                let gas_meter = self.gas_meter.lock().unwrap();
                if table_index >= gas_meter.charge_function_index.unwrap().as_u32() {
                    state.push_operator(Operator::CallIndirect {
                        table_index: table_index + 1,
                        type_index,
                        table_byte,
                    });
                } else {
                    state.push_operator(operator.clone());
                }
            }
            _ => {
                state.push_operator(operator.clone());
            }
        }

        // Perform batch charging after critical points
        match operator {
            Operator::Loop { .. } | Operator::BrIf { .. } | Operator::Else => {
                if self.accumulated_cost > 0 {
                    let gas_meter = self.gas_meter.lock().unwrap();

                    state.extend(&[
                        Operator::I64Const {
                            value: self.accumulated_cost as i64,
                        },
                        Operator::Call {
                            function_index: gas_meter.charge_function_index.unwrap().as_u32(),
                        },
                    ]);

                    self.accumulated_cost = 0;
                }
            }
            _ => {}
        }

        Ok(())
    }
}

// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use std::fmt;
use wasmer::{LocalFunctionIndex, MiddlewareError, MiddlewareReaderState, ModuleMiddleware};
use wasmer::wasmparser::Operator;

pub struct ProhibitOpsMiddleware;

impl ProhibitOpsMiddleware {
    pub fn new() -> Self {
        Self
    }
}

impl fmt::Debug for ProhibitOpsMiddleware {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ProhibitOpsMiddleware").finish()
    }
}

impl ModuleMiddleware for ProhibitOpsMiddleware {
    fn generate_function_middleware(
        &self,
        _index: LocalFunctionIndex,
    ) -> Box<dyn wasmer::FunctionMiddleware> {
        Box::new(ProhibitOpsFunctionMiddleware)
    }
}

struct ProhibitOpsFunctionMiddleware;

impl fmt::Debug for ProhibitOpsFunctionMiddleware {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ProhibitOpsFunctionMiddleware").finish()
    }
}

impl wasmer::FunctionMiddleware for ProhibitOpsFunctionMiddleware {
    fn feed<'a>(
        &mut self,
        operator: Operator<'a>,
        _state: &mut MiddlewareReaderState<'a>,
    ) -> Result<(), MiddlewareError> {
        match operator {
            Operator::CallIndirect { .. } | Operator::ReturnCallIndirect { .. } => {
                // Return an error to prohibit CallIndirect and ReturnCallIndirect
                return Err(MiddlewareError::new("prohibited", "CallIndirect and ReturnCallIndirect are prohibited"));
            }
            _ => {}
        }

        Ok(())
    }
}
// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use wasmer::{Instance, Module, Store, imports, Function, Value, RuntimeError, Trap, TrapCode};
use wasmer_compiler::{CompilerConfig, Middleware, MiddlewareChain, ModuleMiddleware, FunctionBody, Operator};
use wasmer_engine::EngineBuilder;
use std::sync::{Arc, Mutex};

struct BlockBreakpointMiddleware {}

impl ModuleMiddleware for BlockBreakpointMiddleware {
    fn transform_function(
        &self,
        _local_function_index: usize,
        function_body: &mut FunctionBody,
    ) {
        // 在函数体的开始处插入中断指令
        function_body.code.insert(0, Operator::Unreachable);

        // 在代码块、循环、条件跳转和函数调用的开始处插入中断指令
        for (i, op) in function_body.code.iter().enumerate() {
            match op {
                Operator::Block { .. } | Operator::Loop { .. } | Operator::If { .. } |
                Operator::Br { .. } | Operator::BrIf { .. } | Operator::Call { .. } |
                Operator::CallIndirect { .. } => {
                    function_body.code.insert(i + 1, Operator::Unreachable);
                }
                _ => {}
            }
        }
    }
}

pub struct BlockBreakpointMiddlewareGenerator {

}

impl Middleware for BlockBreakpointMiddlewareGenerator {
  fn generate(&self) -> Box<dyn ModuleMiddleware> {
      Box::new(BlockBreakpointMiddleware {})
  }
}


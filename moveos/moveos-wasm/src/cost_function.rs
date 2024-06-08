// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use wasmer::wasmparser::Operator;

pub fn cost_function(operator: &Operator) -> u64 {
    match operator {
        Operator::Call { .. }
        | Operator::ReturnCall { .. }
        | Operator::ReturnCallIndirect { .. } => 5,
        Operator::I64DivS | Operator::I64DivU | Operator::I64RemS | Operator::I64RemU => 4,
        Operator::F64Add | Operator::F64Sub | Operator::F64Mul | Operator::F64Div => 4,
        Operator::I32DivS | Operator::I32DivU | Operator::I32RemS | Operator::I32RemU => 3,
        Operator::I64Add | Operator::I64Sub | Operator::I64Mul => 3,
        Operator::F32Add | Operator::F32Sub | Operator::F32Mul | Operator::F32Div => 3,
        Operator::MemoryGrow { .. } | Operator::MemorySize { .. } => 3,
        Operator::I32Add | Operator::I32Sub | Operator::I32Mul => 2,
        Operator::Br { .. } | Operator::BrIf { .. } | Operator::BrTable { .. } => 2,
        Operator::I32Load { .. }
        | Operator::I64Load { .. }
        | Operator::F32Load { .. }
        | Operator::F64Load { .. } => 2,
        Operator::I32Store { .. }
        | Operator::I64Store { .. }
        | Operator::F32Store { .. }
        | Operator::F64Store { .. } => 2,
        Operator::I32Load8S { .. }
        | Operator::I32Load8U { .. }
        | Operator::I32Load16S { .. }
        | Operator::I32Load16U { .. } => 2,
        Operator::I64Load8S { .. }
        | Operator::I64Load8U { .. }
        | Operator::I64Load16S { .. }
        | Operator::I64Load16U { .. } => 2,
        Operator::I64Load32S { .. } | Operator::I64Load32U { .. } => 2,
        Operator::I32Store8 { .. }
        | Operator::I32Store16 { .. }
        | Operator::I64Store8 { .. }
        | Operator::I64Store16 { .. } => 2,
        Operator::I64Store32 { .. } => 2,
        Operator::GlobalGet { .. } | Operator::GlobalSet { .. } => 2,
        _ => 1,
    }
}

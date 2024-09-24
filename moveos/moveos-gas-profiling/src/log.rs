// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use move_binary_format::file_format::CodeOffset;
use move_binary_format::file_format_common::Opcodes;
use move_core_types::account_address::AccountAddress;
use move_core_types::gas_algebra::InternalGas;
use move_core_types::identifier::Identifier;
use move_core_types::language_storage::{ModuleId, TypeTag};
use smallvec::{smallvec, SmallVec};

/// An event occurred during the execution of a function, along with the
/// gas cost associated with it, if any.
#[derive(Debug, Clone)]
pub enum ExecutionGasEvent {
    /// A special event indicating that the program counter has moved to
    /// a specific offset. This is emitted by the branch instructions
    /// and is crucial for reconstructing the control flow.
    Loc(CodeOffset),
    Bytecode {
        op: Opcodes,
        cost: InternalGas,
    },
    Call(CallFrame),
    CallNative {
        module_id: ModuleId,
        fn_name: Identifier,
        ty_args: Vec<TypeTag>,
        cost: InternalGas,
    },
    LoadResource {
        addr: AccountAddress,
        ty: TypeTag,
        cost: InternalGas,
    },
    CreateTy {
        cost: InternalGas,
    },
}

/// An enum representing the name of a call frame.
/// Could be either a script or a function.
#[derive(Debug, Clone)]
pub enum FrameName {
    Script,
    Function {
        module_id: ModuleId,
        name: Identifier,
        ty_args: Vec<TypeTag>,
    },
}

/// A struct containing information about a function call, including the name of the
/// function and all gas events that happened during the call.
#[derive(Debug, Clone)]
pub struct CallFrame {
    pub name: FrameName,
    pub events: Vec<ExecutionGasEvent>,
}

impl CallFrame {
    pub fn new_function(module_id: ModuleId, name: Identifier, ty_args: Vec<TypeTag>) -> Self {
        Self {
            name: FrameName::Function {
                module_id,
                name,
                ty_args,
            },
            events: vec![],
        }
    }

    pub fn new_script() -> Self {
        Self {
            name: FrameName::Script,
            events: vec![],
        }
    }
}

#[derive(Debug, Clone)]
pub struct ExecutionAndIOCosts {
    pub total: InternalGas,
    pub call_graph: CallFrame,
}

#[derive(Debug, Clone)]
pub struct TransactionGasLog {
    pub exec_io: ExecutionAndIOCosts,
    pub storage: InternalGas,
}

pub struct GasEventIter<'a> {
    stack: SmallVec<[(&'a CallFrame, usize); 16]>,
}

impl<'a> Iterator for GasEventIter<'a> {
    type Item = &'a ExecutionGasEvent;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.stack.last_mut() {
                None => return None,
                Some((frame, pc)) => {
                    if *pc >= frame.events.len() {
                        self.stack.pop();
                        continue;
                    }

                    let event = &frame.events[*pc];
                    *pc += 1;
                    if let ExecutionGasEvent::Call(child_frame) = event {
                        self.stack.push((child_frame, 0))
                    }
                    return Some(event);
                }
            }
        }
    }
}

impl ExecutionAndIOCosts {
    #[allow(clippy::needless_lifetimes)]
    pub fn gas_events<'a>(&'a self) -> GasEventIter<'a> {
        GasEventIter {
            stack: smallvec![(&self.call_graph, 0)],
        }
    }
}

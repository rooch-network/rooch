// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::log::{CallFrame, ExecutionAndIOCosts, ExecutionGasEvent, FrameName};
use crate::render::Render;
use move_core_types::gas_algebra::InternalGas;
use std::ops::AddAssign;

/// Represents a node in a general tree structure where each node is tagged with
/// some text & a numerical value.
#[derive(Clone)]
pub struct Node<N> {
    pub text: String,
    pub val: N,
    pub children: Vec<Node<N>>,
}

#[derive(Clone)]
pub struct TypeErasedExecutionAndIoCosts {
    pub total: InternalGas,
    pub tree: Node<InternalGas>,
}

impl<N> Node<N> {
    pub fn new(name: impl Into<String>, data: impl Into<N>) -> Self {
        Self {
            text: name.into(),
            val: data.into(),
            children: vec![],
        }
    }

    pub fn new_with_children(
        name: impl Into<String>,
        data: impl Into<N>,
        children: impl IntoIterator<Item = Self>,
    ) -> Self {
        Self {
            text: name.into(),
            val: data.into(),
            children: children.into_iter().collect(),
        }
    }

    pub fn preorder_traversel(&self, mut f: impl FnMut(usize, &str, &N)) {
        let mut stack = vec![(self, 0)];

        while let Some((node, depth)) = stack.pop() {
            f(depth, &node.text, &node.val);
            stack.extend(node.children.iter().map(|child| (child, depth + 1)).rev());
        }
    }
}

impl CallFrame {
    fn to_erased(&self) -> Node<InternalGas> {
        let name = match &self.name {
            FrameName::Script => "script".to_string(),
            FrameName::Function {
                module_id,
                name,
                ty_args,
            } => {
                format!(
                    "{}",
                    Render(&(module_id, name.as_ident_str(), ty_args.as_slice()))
                )
            }
        };

        let children = self
            .events
            .iter()
            .map(|event| event.to_erased())
            .collect::<Vec<_>>();

        Node::new_with_children(name, 0, children)
    }
}

impl ExecutionAndIOCosts {
    /// Convert the gas log into a type-erased representation.
    pub fn to_erased(&self) -> TypeErasedExecutionAndIoCosts {
        let nodes = vec![self.call_graph.to_erased()];

        TypeErasedExecutionAndIoCosts {
            total: self.total,
            tree: Node::new_with_children("execution & IO (gas unit, full trace)", 0, nodes),
        }
    }
}

impl<N> Node<N>
where
    N: AddAssign<N> + Copy,
{
    pub fn include_child_costs(&mut self) {
        for child in &mut self.children {
            child.include_child_costs();
            self.val += child.val;
        }
    }
}

impl ExecutionGasEvent {
    fn to_erased(&self) -> Node<InternalGas> {
        use ExecutionGasEvent::*;

        match self {
            Loc(offset) => Node::new(format!("@{}", offset), 0),
            Bytecode { op, cost } => Node::new(format!("{:?}", op).to_ascii_lowercase(), *cost),
            Call(frame) => frame.to_erased(),
            CallNative {
                module_id,
                fn_name,
                ty_args,
                cost,
            } => Node::new(
                format!(
                    "{}",
                    Render(&(module_id, fn_name.as_ident_str(), ty_args.as_slice()))
                ),
                *cost,
            ),
            LoadResource { addr, ty, cost } => {
                Node::new(format!("load<{}::{}>", Render(addr), ty), *cost)
            }
            CreateTy { cost } => Node::new("create_ty", *cost),
        }
    }
}

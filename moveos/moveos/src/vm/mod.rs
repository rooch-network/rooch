// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use move_core_types::resolver::MoveResolver;
use moveos_types::table::TableResolver;

pub mod dependency_order;
pub mod move_vm_ext;
pub mod tx_argument_resolver;
pub mod vm_status_explainer;

pub trait MoveResolverExt: MoveResolver + TableResolver {}

impl<T> MoveResolverExt for T where T: MoveResolver + TableResolver {}

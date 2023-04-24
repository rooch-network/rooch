// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use move_core_types::resolver::MoveResolver;
use moveos_stdlib::natives::moveos_stdlib::raw_table::TableResolver;


pub mod move_vm_ext;
pub mod tx_argument_resolver;

pub trait MoveResolverExt: MoveResolver + TableResolver {}

impl<T> MoveResolverExt for T where T: MoveResolver + TableResolver {}

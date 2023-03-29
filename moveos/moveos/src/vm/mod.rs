// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use move_core_types::resolver::MoveResolver;
use move_table_extension::TableResolver;
use moveos_stdlib::natives::mos_stdlib::object_extension::ObjectResolver;

pub mod move_vm_ext;
pub mod tx_argument_resolver;

pub trait MoveResolverExt: MoveResolver + TableResolver + ObjectResolver {}

impl<T> MoveResolverExt for T where T: MoveResolver + TableResolver + ObjectResolver {}

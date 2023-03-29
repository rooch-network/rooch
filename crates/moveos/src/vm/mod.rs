// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use framework::natives::mos_stdlib::object_extension::ObjectResolver;
use move_core_types::resolver::MoveResolver;
use move_table_extension::TableResolver;

pub mod move_vm_ext;
pub mod tx_argument_resolver;

pub trait MoveResolverExt: MoveResolver + TableResolver + ObjectResolver {}

impl<T> MoveResolverExt for T where T: MoveResolver + TableResolver + ObjectResolver {}

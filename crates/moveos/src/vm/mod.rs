// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use move_core_types::resolver::MoveResolver;
use move_table_extension::TableResolver;

pub mod move_vm_ext;

pub trait MoveResolverExt: MoveResolver + TableResolver {}

impl<T> MoveResolverExt for T where T: MoveResolver + TableResolver {}

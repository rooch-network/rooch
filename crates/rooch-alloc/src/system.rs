// Licensed under SPDX-License-Identifier: Apache-2.0
// Copyright 2021 TiKV Project Authors

pub use crate::default::*;

pub type Allocator = std::alloc::System;
pub const fn allocator() -> Allocator {
    std::alloc::System
}

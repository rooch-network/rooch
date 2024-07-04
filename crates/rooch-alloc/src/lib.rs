// Copyright 2021 TiKV Project Authors
// SPDX-License-Identifier: Apache-2.0

#[cfg(feature = "jemalloc")]
#[macro_use]
extern crate lazy_static;

pub mod error;

#[cfg(not(all(unix, not(fuzzing), feature = "jemalloc")))]
mod default;

pub type AllocStats = Vec<(&'static str, usize)>;

// Allocators
#[cfg(all(unix, not(fuzzing), feature = "jemalloc"))]
#[path = "jemalloc.rs"]
mod imp;

#[cfg(not(all(unix, not(fuzzing), any(feature = "jemalloc"))))]
#[path = "system.rs"]
mod imp;

pub use crate::imp::*;

#[global_allocator]
static ALLOC: imp::Allocator = imp::allocator();

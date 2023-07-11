// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// Copyright (c) 2021, Facebook, Inc. and its affiliates
// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0
#![warn(
    future_incompatible,
    nonstandard_style,
    rust_2018_idioms,
    rust_2021_compatibility
)]

pub mod traits;
pub use traits::Map;
pub mod rocks;
pub use rocks::RawStoreError;
pub mod macros;
pub mod metrics;
pub mod test_db;

pub type StoreError = rocks::RawStoreError;

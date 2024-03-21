// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::jsonrpc_types::StrView;
use bitcoin::hashes::sha256d;
use schemars::JsonSchema;

pub type SHA256DView = StrView<sha256d::Hash>;

/// Attempted to create a hash from an invalid length slice.
#[derive(Debug, Copy, Clone, PartialEq, Eq, JsonSchema)]
pub struct FromSliceError {
    expected: StrView<usize>,
    got: StrView<usize>,
}

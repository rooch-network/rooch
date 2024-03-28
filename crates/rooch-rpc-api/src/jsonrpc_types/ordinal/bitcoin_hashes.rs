// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::jsonrpc_types::StrView;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Attempted to create a hash from an invalid length slice.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct FromSliceError {
    expected: StrView<usize>,
    got: StrView<usize>,
}

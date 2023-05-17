// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Custom error type for MoveOS.
#[derive(Eq, PartialEq, Clone, Debug, Serialize, Deserialize, Error, Hash)]
pub enum MoveOSError {
    #[error("Invalid length error:")]
    InvalidlengthError(),
    #[error("Move VM Module Deserialization error: ")]
    VMModuleDeserializationError,
}

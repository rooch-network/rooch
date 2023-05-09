// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Custom error type for Rooch.
#[derive(Eq, PartialEq, Clone, Debug, Serialize, Deserialize, Error, Hash)]
pub enum RoochError {
    #[error("base64 decode error: {0}")]
    Base64DecodeError(String),
    #[error("Invalid length error:")]
    InvalidlengthError(),
    #[error("Signature key generation error: {0}")]
    SignatureKeyGenError(String),
    #[error("Key Conversion Error: {0}")]
    KeyConversionError(String),
}

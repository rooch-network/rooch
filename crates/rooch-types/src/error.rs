// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};
use thiserror::Error;

// TODO: merge CliResult, CliError
pub type RoochResult<T> = Result<T, RoochError>;

/// Custom error type for Rooch.
#[derive(Eq, PartialEq, Clone, Debug, Serialize, Deserialize, Error, Hash)]
pub enum RoochError {
    #[error("base64 decode error: {0}")]
    Base64DecodeError(String),
    #[error("Invalid length error:")]
    InvalidlengthError(),

    // Cryptography errors.
    #[error("Signature key generation error: {0}")]
    SignatureKeyGenError(String),
    #[error("Key Conversion Error: {0}")]
    KeyConversionError(String),

    // Signature verification
    #[error("Signature is not valid: {}", error)]
    InvalidSignature { error: String },
    #[error("Value was not signed by the correct sender: {}", error)]
    IncorrectSigner { error: String },

    #[error("Use of disabled feature: {:?}", error)]
    UnsupportedFeatureError { error: String },
}

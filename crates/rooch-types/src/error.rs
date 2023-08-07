// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};
use std::io;
use thiserror::Error;

pub type RoochResult<T> = Result<T, RoochError>;

/// Custom error type for Rooch.
#[derive(Eq, PartialEq, Clone, Debug, Serialize, Deserialize, Error, Hash)]
pub enum RoochError {
    /// config
    #[error("Unable to find config {0}, have you run `rooch init`?")]
    ConfigNotFoundError(String),
    #[error("Unable to load config: {0}, Reason: {1}.")]
    ConfigLoadError(String, String),

    /// common
    #[error("Aborted command")]
    AbortedError,
    #[error("Invalid arguments: {0}")]
    CommandArgumentError(String),

    /// move
    #[error("Move compilation failed: {0}")]
    MoveCompilationError(String),
    #[error("Move unit tests failed")]
    MoveTestError,
    #[error("Move Prover failed: {0}")]
    MoveProverError(String),
    #[error("Unable to parse '{0}': error: {1}")]
    UnableToParse(&'static str, String),
    #[error("Unable to read file '{0}', error: {1}")]
    UnableToReadFile(String, String),
    #[error("Error: {0}")]
    UnexpectedError(String),

    #[error("Simulation failed with status: {0}")]
    SimulationError(String),

    #[error("Coverage failed with status: {0}")]
    CoverageError(String),
    #[error("BCS failed with status: {0}")]
    BcsError(String),
    #[error("IO error: {0}")]
    IOError(String),
    #[error("Sign message error: {0}")]
    SignMessageError(String),
    #[error("Transaction error: {0}")]
    TransactionError(String),
    #[error("View function error: {0}")]
    ViewFunctionError(String),
    #[error("Import account error: {0}")]
    ImportAccountError(String),
    #[error("Switch account error: {0}")]
    SwitchAccountError(String),
    #[error("Update account error: {0}")]
    UpdateAccountError(String),
    #[error("Generate key error: {0}")]
    GenerateKeyError(String),

    //#[error("base64 decode error: {0}")]
    //Base64DecodeError(String),
    #[error("Invalid length error:")]
    InvalidlengthError(),

    // Cryptography errors.
    #[error("Signature key generation error: {0}")]
    SignatureKeyGenError(String),
    #[error("Key Conversion Error: {0}")]
    KeyConversionError(String),

    #[error("Switch env error: {0}")]
    SwitchEnvError(String),
    #[error("Remove env error: {0}")]
    RemoveEnvError(String),

    // Signature verification
    #[error("Signature is not valid: {}", error)]
    InvalidSignature { error: String },
    #[error("Value was not signed by the correct sender: {}", error)]
    IncorrectSigner { error: String },

    #[error("Use of disabled feature: {:?}", error)]
    UnsupportedFeatureError { error: String },
}

impl From<anyhow::Error> for RoochError {
    fn from(e: anyhow::Error) -> Self {
        RoochError::UnexpectedError(e.to_string())
    }
}

impl From<bcs::Error> for RoochError {
    fn from(e: bcs::Error) -> Self {
        RoochError::BcsError(e.to_string())
    }
}

impl From<io::Error> for RoochError {
    fn from(e: io::Error) -> Self {
        RoochError::IOError(e.to_string())
    }
}

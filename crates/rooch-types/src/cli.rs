// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use std::io;

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// A common result to be handled in main
pub type CliResult<T> = Result<T, CliError>;

// TODO: TBD, move moveos-cli to rooch, use RoochError
/// CLI Errors for reporting through telemetry and outputs
#[derive(Eq, PartialEq, Clone, Debug, Serialize, Deserialize, Error)]
pub enum CliError {
    #[error("Aborted command")]
    AbortedError,
    #[error("Invalid arguments: {0}")]
    CommandArgumentError(String),
    #[error("Unable to load config: {0}, Reason: {1}.")]
    ConfigLoadError(String, String),
    #[error("Unable to find config {0}, have you run `rooch init`?")]
    ConfigNotFoundError(String),
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
}

impl From<anyhow::Error> for CliError {
    fn from(e: anyhow::Error) -> Self {
        CliError::UnexpectedError(e.to_string())
    }
}

impl From<bcs::Error> for CliError {
    fn from(e: bcs::Error) -> Self {
        CliError::BcsError(e.to_string())
    }
}

impl From<io::Error> for CliError {
    fn from(e: io::Error) -> Self {
        CliError::IOError(e.to_string())
    }
}

#[async_trait]
pub trait CommandAction<T: Serialize + Send>: Sized + Send {
    /// Executes the command, returning a command specific type
    async fn execute(self) -> CliResult<T>;

    /// Executes the command, and serializes it to the common JSON output type
    async fn execute_serialized(self) -> CliResult<String> {
        match self.execute().await {
            Ok(result) => Ok(serde_json::to_string_pretty(&result).unwrap()),
            Err(e) => Err(e),
        }
    }
}

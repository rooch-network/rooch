// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// A common result to be handled in main
pub type CliResult<T> = Result<T, CliError>;

/// CLI Errors for reporting through telemetry and outputs
#[derive(Eq, PartialEq, Clone, Debug, Serialize, Deserialize, Error)]
pub enum CliError {
    #[error("Aborted command")]
    AbortedError,
    #[error("Invalid arguments: {0}")]
    CommandArgumentError(String),
    #[error("Unable to load config: {0} {1}")]
    ConfigLoadError(String, String),
    #[error("Unable to find config {0}, have you run `aptos init`?")]
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

// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// Define the errors
use jsonrpsee::core::error::Error as JsonError;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("file not found")]
    NotFound,

    #[error("config file not found, \nHelp: You can add fixtures/config.yml to the ROOCH_CONFIG environment variable,\n      or copy fixtures/config.yml to the current directory")]
    ConfigReadError,

    #[error("config parse error")]
    ConfigParseError,

    #[error("actor reference {0} error")]
    ActorRefError(#[from] coerce::actor::ActorRefErr),

    #[error("unknown error")]
    Unknown,
}

impl PartialEq for Error {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::NotFound, Self::NotFound) => true,
            (Self::Unknown, Self::Unknown) => true,
            (Self::ActorRefError(e1), Self::ActorRefError(e2)) => e1 == e2,
            _ => false,
        }
    }
}

impl From<Error> for tonic::Status {
    fn from(e: Error) -> Self {
        match e {
            Error::ConfigReadError | Error::ConfigParseError => {
                tonic::Status::internal(e.to_string())
            }
            Error::NotFound => {
                tonic::Status::not_found("No config file found by the given condition")
            }
            Error::ActorRefError(e) => tonic::Status::internal(e.to_string()),
            Error::Unknown => tonic::Status::unknown("Unknown error"),
        }
    }
}

impl From<Error> for JsonError {
    fn from(e: Error) -> Self {
        match e {
            Error::ConfigReadError | Error::ConfigParseError => JsonError::Custom(e.to_string()),
            Error::NotFound => {
                JsonError::Custom("No config file found by the given condition".to_string())
            }
            Error::ActorRefError(e) => JsonError::Custom(e.to_string()),
            Error::Unknown => JsonError::Custom("Unknown error".to_string()),
        }
    }
}

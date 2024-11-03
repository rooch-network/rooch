// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum FaucetError {
    #[error("Failed to parse transaction response {0}")]
    ParseTransactionResponseError(String),

    #[error("Faucet does not have enough balance")]
    InsuffientBalance,

    #[error("`{0}` is not supported")]
    NotSupport(String),

    #[error("Address `{0}` is not valid")]
    InvalidAddress(String),

    #[error("Timed out waiting for a coin from the gas coin pool")]
    NoRGasAvailable,

    #[error("Wallet Error: `{0}`")]
    Wallet(String),

    #[error("Coin Transfer Failed `{0}`")]
    Transfer(String),

    #[error("Request consumer queue closed.")]
    ChannelClosed,

    #[error("Coin amounts sent are incorrect:`{0}`")]
    CoinAmountTransferredIncorrect(String),

    #[error("{0}")]
    Custom(String),
}

impl FaucetError {
    pub(crate) fn custom(e: impl ToString) -> Self {
        FaucetError::Custom(e.to_string())
    }
}

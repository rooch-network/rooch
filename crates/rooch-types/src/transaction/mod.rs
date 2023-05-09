// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use serde::{Deserialize, Serialize};

pub mod authenticator;
pub mod ethereum;
pub mod rooch;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum TransactionType {
    Rooch,
    Ethereum,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct RawTransaction {
    pub transaction_type: TransactionType,
    pub raw: Vec<u8>,
}

pub trait AbstractTransaction {
    /// The transaction sender authenticator type.
    /// Usually it is a signature.
    type Authenticator;
    type Hash;

    fn transaction_type(&self) -> TransactionType;

    fn decode(bytes: &[u8]) -> Result<Self>
    where
        Self: std::marker::Sized;
    fn encode(&self) -> Vec<u8>;

    fn hash(&self) -> Self::Hash;
    fn authenticator(&self) -> Self::Authenticator;

    // Verify the Authenticator
    fn verify(&self) -> bool;
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum TypedTransaction {
    Rooch(rooch::RoochTransaction),
    Ethereum(ethereum::EthereumTransaction),
}

// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use moveos_types::serde::Readable;
use serde::{Deserialize, Serialize};
use serde_with::hex::Hex;
use serde_with::serde_as;
use std::{fmt::Display, str::FromStr};

/// This type is used to wrap the authentication key bytes.
/// It is not care about the authentication validator.
/// If you want binding the authentication validator, you can use the `AuthenticationKey<ValidatorType>` type in `rooch_types::framework`.
#[serde_as]
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct AuthenticationKey(#[serde_as(as = "Readable<Hex,_>")] Vec<u8>);

impl Display for AuthenticationKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{}", hex::encode(&self.0))
    }
}

impl FromStr for AuthenticationKey {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(hex::decode(s.strip_prefix("0x").unwrap_or(s))?))
    }
}

impl From<AuthenticationKey> for Vec<u8> {
    fn from(authentication_key: AuthenticationKey) -> Self {
        authentication_key.0
    }
}

impl AuthenticationKey {
    pub fn new(authentication_key: Vec<u8>) -> Self {
        Self(authentication_key)
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl AsRef<[u8]> for AuthenticationKey {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

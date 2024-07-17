// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use bech32::{decode, Hrp};
use bitcoin::{
    hex::{Case, DisplayHex},
    secp256k1::SecretKey,
};
use once_cell::sync::Lazy;

use crate::{crypto::SignatureScheme, error::RoochError};

pub static ROOCH_SECRET_KEY_HRP: Lazy<Hrp> =
    Lazy::new(|| Hrp::parse("roochsecretkey").expect("roochsecretkey is a valid HRP"));

/// Rooch Key length in bech32 string length: 14 hrp + 60 data
pub const LENGTH_SK_BECH32: usize = 74;

// Parsed Rooch Key, either a bech32 encoded private key or a raw material key
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ParsedSecretKey(SecretKey);

impl ParsedSecretKey {
    pub fn into_inner(self) -> SecretKey {
        self.0
    }

    pub fn parse(s: &str) -> anyhow::Result<Self, anyhow::Error> {
        if s.starts_with(ROOCH_SECRET_KEY_HRP.as_str()) && s.len() == LENGTH_SK_BECH32 {
            let (hrp, data) = decode(s)?;
            if hrp != *ROOCH_SECRET_KEY_HRP {
                return Err(anyhow::Error::new(RoochError::CommandArgumentError(
                    format!("Hrp [{:?}] check failed", hrp.to_string()),
                )));
            };
            if data.len() != 33 {
                return Err(anyhow::Error::new(RoochError::CommandArgumentError(
                    format!(
                        "Private key [{:?}] length check failed",
                        data.to_hex_string(Case::Lower)
                    ),
                )));
            };
            if data[0] != SignatureScheme::Secp256k1.flag() {
                return Err(anyhow::Error::new(RoochError::CommandArgumentError(
                    format!("Flag [{:?}] check failed", data[0]),
                )));
            };
            Ok(Self(SecretKey::from_slice(&data[1..])?))
        } else {
            match SecretKey::from_slice(s.as_bytes()) {
                Ok(a) => Ok(Self(a)),
                Err(_) => Err(anyhow::Error::new(RoochError::CommandArgumentError(
                    "Parse from a raw material key failed".to_owned(),
                ))),
            }
        }
    }
}

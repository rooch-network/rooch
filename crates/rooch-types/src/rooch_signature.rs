// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::{
    crypto::{Ed25519RoochSignature, RoochSignature, RoochSignatureInner, Secp256k1RoochSignature},
    error::RoochError,
};
use bitcoin::secp256k1::{schnorr::Signature, Message, PublicKey};
use fastcrypto::traits::ToFromBytes;
use serde::{Deserialize, Serialize};
use std::{result::Result, str::FromStr};

// Enums for defined signatures
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InnerSignature {
    Ed25519Signature(Ed25519RoochSignature),
    Secp256k1Signature(Secp256k1RoochSignature),
    SchnorrSignature(Signature),
}

impl InnerSignature {
    pub fn verify(self, value: &[u8], public_key: Option<String>) -> Result<(), RoochError> {
        match self {
            Self::Ed25519Signature(sig) => sig.verify(value),
            Self::Secp256k1Signature(sig) => sig.verify(value),
            Self::SchnorrSignature(sig) => {
                let public_key = public_key.unwrap_or_else(|| panic!("Unable to parse public key"));
                let x_only_public_key = PublicKey::from_str(
                    public_key.strip_prefix("0x").unwrap_or(public_key.as_str()),
                )
                .map_err(|_| RoochError::KeyConversionError("Invalid public key".to_owned()))?
                .x_only_public_key()
                .0;
                let message = Message::from_digest_slice(value)
                    .map_err(|_| RoochError::InvalidlengthError())?;
                sig.verify(&message, &x_only_public_key)
                    .map_err(|e| RoochError::InvalidSignature {
                        error: format!("Invalid schnorr signature {:?}", e),
                    })
            }
        }
    }
}

// Parsed Rooch Signature, either Ed25519RoochSignature or Secp256k1RoochSignature, or SchnorrSignature
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ParsedSignature(InnerSignature);

impl ParsedSignature {
    pub fn into_inner(self) -> InnerSignature {
        self.0
    }

    pub fn from_signature(signature: InnerSignature) -> Self {
        Self(signature)
    }

    pub fn parse(s: &str) -> anyhow::Result<Self, anyhow::Error> {
        let signature_bytes = hex::decode(s)?;
        // either ed25519 or secp256k1 rooch signature, or schnorr signature
        let signature = if signature_bytes.len() == Ed25519RoochSignature::LENGTH {
            Self::from_signature(InnerSignature::Ed25519Signature(
                Ed25519RoochSignature::from_bytes(&signature_bytes)?,
            ))
        } else if signature_bytes.len() == Secp256k1RoochSignature::LENGTH {
            Self::from_signature(InnerSignature::Secp256k1Signature(
                Secp256k1RoochSignature::from_bytes(&signature_bytes)?,
            ))
        } else {
            Self::from_signature(InnerSignature::SchnorrSignature(Signature::from_slice(
                &signature_bytes,
            )?))
        };
        Ok(signature)
    }
}

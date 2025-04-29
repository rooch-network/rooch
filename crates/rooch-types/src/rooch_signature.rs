// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::{
    crypto::{Ed25519RoochSignature, RoochSignature, RoochSignatureInner, Secp256k1RoochSignature},
    error::RoochError,
};
use anyhow::{Error, Result};
use bitcoin::{
    key::constants::SCHNORR_SIGNATURE_SIZE,
    secp256k1::{schnorr::Signature, Message, PublicKey},
};
use fastcrypto::traits::ToFromBytes;
use std::str::FromStr;

// Parsed Rooch Signature, either Ed25519RoochSignature or Secp256k1RoochSignature, or SchnorrSignature
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ParsedSignature(Vec<u8>);

impl ParsedSignature {
    pub fn into_inner(self) -> Vec<u8> {
        self.0
    }

    pub fn from_signature(signature: Vec<u8>) -> Self {
        Self(signature)
    }

    pub fn parse(s: &str) -> Result<Self, Error> {
        let signature_bytes = hex::decode(s)?;
        // either ed25519 or secp256k1 rooch signature, or schnorr signature
        let signature = match signature_bytes.len() {
            Ed25519RoochSignature::LENGTH => Self::from_signature(
                Ed25519RoochSignature::from_bytes(&signature_bytes)?
                    .as_bytes()
                    .to_vec(),
            ),
            Secp256k1RoochSignature::LENGTH => Self::from_signature(
                Secp256k1RoochSignature::from_bytes(&signature_bytes)?
                    .as_bytes()
                    .to_vec(),
            ),
            SCHNORR_SIGNATURE_SIZE => Self::from_signature(
                Signature::from_slice(&signature_bytes)?
                    .serialize()
                    .to_vec(),
            ),
            _ => Err(RoochError::InvalidlengthError())?,
        };
        Ok(signature)
    }

    pub fn verify(self, value: &[u8], public_key: Option<String>) -> Result<(), Error> {
        let signature_bytes = self.into_inner();
        match signature_bytes.len() {
            Ed25519RoochSignature::LENGTH => {
                Ed25519RoochSignature::from_bytes(signature_bytes.as_slice())?
                    .verify(value)
                    .map_err(|e| {
                        Error::new(RoochError::InvalidSignature {
                            error: format!("Invalid ed25519 signature {:?}", e),
                        })
                    })
            }
            Secp256k1RoochSignature::LENGTH => {
                Secp256k1RoochSignature::from_bytes(signature_bytes.as_slice())?
                    .verify(value)
                    .map_err(|e| {
                        Error::new(RoochError::InvalidSignature {
                            error: format!("Invalid secp256k1 ecdsa signature {:?}", e),
                        })
                    })
            }
            SCHNORR_SIGNATURE_SIZE => {
                let public_key = public_key.unwrap_or_else(|| panic!("Unable to parse public key"));
                let x_only_public_key = PublicKey::from_str(
                    public_key.strip_prefix("0x").unwrap_or(public_key.as_str()),
                )
                .map_err(|_| RoochError::KeyConversionError("Invalid public key".to_owned()))?
                .x_only_public_key()
                .0;
                let message = Message::from_digest_slice(value)
                    .map_err(|_| RoochError::InvalidlengthError())?;
                let signature = Signature::from_slice(signature_bytes.as_slice())?;
                signature.verify(&message, &x_only_public_key).map_err(|e| {
                    Error::new(RoochError::InvalidSignature {
                        error: format!("Invalid secp256k1 schnorr signature {:?}", e),
                    })
                })
            }
            _ => Err(RoochError::InvalidlengthError())?,
        }
    }
}

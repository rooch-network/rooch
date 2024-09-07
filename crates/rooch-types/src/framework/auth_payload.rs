// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::{
    crypto::{RoochSignature, Signature, SignatureScheme},
    transaction::RoochTransactionData,
};
use anyhow::{ensure, Result};
use bitcoin::consensus::{Decodable, Encodable};
use fastcrypto::{
    hash::Sha256,
    secp256k1::{Secp256k1PublicKey, Secp256k1Signature},
    traits::ToFromBytes,
};
use framework_types::addresses::ROOCH_FRAMEWORK_ADDRESS;
use move_core_types::{account_address::AccountAddress, ident_str, identifier::IdentStr};
use moveos_types::{
    h256::{sha2_256_of, H256},
    state::{MoveStructState, MoveStructType},
};
use serde::{Deserialize, Serialize};
use std::io;

pub const MODULE_NAME: &IdentStr = ident_str!("auth_payload");

/// The original message prefix of the Bitcoin wallet includes the length of the message `x18`
/// We remove the length because the bitcoin consensus codec serialization format already contains the length information
const MESSAGE_INFO_PREFIX: &[u8] = b"Bitcoin Signed Message:\n";
const MESSAGE_INFO: &[u8] = b"Rooch Transaction:\n";

const TX_HASH_HEX_LENGTH: usize = 64;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignData {
    pub message_prefix: Vec<u8>,
    pub message_info: Vec<u8>,
}

impl SignData {
    pub fn new(
        message_prefix: Vec<u8>,
        message_info_without_tx_hash: Vec<u8>,
        tx_data: &RoochTransactionData,
    ) -> Self {
        let message_info = {
            let tx_hash_hex = hex::encode(tx_data.tx_hash().as_bytes()).into_bytes();
            let mut message_info = message_info_without_tx_hash;
            message_info.extend_from_slice(&tx_hash_hex);
            message_info
        };
        SignData {
            message_prefix,
            message_info,
        }
    }

    pub fn new_with_default(tx_data: &RoochTransactionData) -> Self {
        Self::new(MESSAGE_INFO_PREFIX.to_vec(), MESSAGE_INFO.to_vec(), tx_data)
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut data = Vec::new();
        self.consensus_encode(&mut data)
            .expect("Serialize SignData should success");
        data
    }

    /// The message info without tx hash, the verifier should append the tx hash to the message info
    pub fn message_info_without_tx_hash(&self) -> Vec<u8> {
        self.message_info[..self.message_info.len() - TX_HASH_HEX_LENGTH].to_vec()
    }

    pub fn data_hash(&self) -> H256 {
        let data = self.encode();
        sha2_256_of(&data)
    }
}

impl Encodable for SignData {
    fn consensus_encode<S: io::Write + ?Sized>(&self, s: &mut S) -> Result<usize, io::Error> {
        let len = self.message_prefix.consensus_encode(s)?;
        Ok(len + self.message_info.consensus_encode(s)?)
    }
}

impl Decodable for SignData {
    fn consensus_decode<D: io::Read + ?Sized>(
        d: &mut D,
    ) -> Result<Self, bitcoin::consensus::encode::Error> {
        Ok(SignData {
            message_prefix: Decodable::consensus_decode(d)?,
            message_info: Decodable::consensus_decode(d)?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthPayload {
    // Message signature
    pub signature: Vec<u8>,
    // Some wallets add magic prefixes, such as unisat adding 'Bitcoin Signed Message:\n'
    pub message_prefix: Vec<u8>,
    // Description of a user-defined signature, the message info does not include the tx hash
    pub message_info: Vec<u8>,
    // Public key of address
    pub public_key: Vec<u8>,
    // Wallet address
    pub from_address: Vec<u8>,
}

impl MoveStructType for AuthPayload {
    const ADDRESS: AccountAddress = ROOCH_FRAMEWORK_ADDRESS;
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = ident_str!("AuthPayload");
}

impl MoveStructState for AuthPayload {
    fn struct_layout() -> move_core_types::value::MoveStructLayout {
        move_core_types::value::MoveStructLayout::new(vec![
            move_core_types::value::MoveTypeLayout::Vector(Box::new(
                move_core_types::value::MoveTypeLayout::U8,
            )),
            move_core_types::value::MoveTypeLayout::Vector(Box::new(
                move_core_types::value::MoveTypeLayout::U8,
            )),
            move_core_types::value::MoveTypeLayout::Vector(Box::new(
                move_core_types::value::MoveTypeLayout::U8,
            )),
            move_core_types::value::MoveTypeLayout::Vector(Box::new(
                move_core_types::value::MoveTypeLayout::U8,
            )),
            move_core_types::value::MoveTypeLayout::Vector(Box::new(
                move_core_types::value::MoveTypeLayout::U8,
            )),
        ])
    }
}

impl AuthPayload {
    pub fn new(sign_data: SignData, signature: Signature, bitcoin_address: String) -> Self {
        debug_assert_eq!(signature.scheme(), SignatureScheme::Secp256k1);
        let message_info = sign_data.message_info_without_tx_hash();
        AuthPayload {
            signature: signature.signature_bytes().to_vec(),
            message_prefix: sign_data.message_prefix,
            message_info,
            public_key: signature.public_key_bytes().to_vec(),
            from_address: bitcoin_address.into_bytes(),
        }
    }

    pub fn verify(&self, tx_data: &RoochTransactionData) -> Result<()> {
        let pk = Secp256k1PublicKey::from_bytes(&self.public_key)?;
        let sign_data = SignData::new(
            self.message_prefix.clone(),
            self.message_info.clone(),
            tx_data,
        );
        let message = sign_data.encode();
        let message_hash = sha2_256_of(&message).0.to_vec();
        let signature = Secp256k1Signature::from_bytes(&self.signature)?;
        pk.verify_with_hash::<Sha256>(&message_hash, &signature)?;
        Ok(())
    }

    pub fn from_address(&self) -> Result<String> {
        Ok(String::from_utf8(self.from_address.to_vec())?)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MultisignAuthPayload {
    pub signatures: Vec<Vec<u8>>,
    pub message_prefix: Vec<u8>,
    pub message_info: Vec<u8>,
    pub public_keys: Vec<Vec<u8>>,
}

impl MultisignAuthPayload {
    pub fn build_multisig_payload(mut payloads: Vec<AuthPayload>) -> Result<Self> {
        ensure!(payloads.len() > 1, "At least two signatures are required");
        let first_payload = payloads.remove(0);
        let message_prefix = first_payload.message_prefix.clone();
        let message_info = first_payload.message_info.clone();
        let mut signatures = vec![first_payload.signature];
        let mut public_keys = vec![first_payload.public_key];
        for payload in payloads {
            ensure!(
                payload.message_prefix == message_prefix,
                "All signatures must have the same message prefix"
            );
            ensure!(
                payload.message_info == message_info,
                "All signatures must have the same message info"
            );
            signatures.push(payload.signature);
            public_keys.push(payload.public_key);
        }
        Ok(Self {
            signatures,
            public_keys,
            message_prefix,
            message_info,
        })
    }
}
impl MoveStructType for MultisignAuthPayload {
    const ADDRESS: AccountAddress = ROOCH_FRAMEWORK_ADDRESS;
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = ident_str!("MultisignAuthPayload");
}

impl MoveStructState for MultisignAuthPayload {
    fn struct_layout() -> move_core_types::value::MoveStructLayout {
        move_core_types::value::MoveStructLayout::new(vec![
            move_core_types::value::MoveTypeLayout::Vector(Box::new(
                move_core_types::value::MoveTypeLayout::Vector(Box::new(
                    move_core_types::value::MoveTypeLayout::U8,
                )),
            )),
            move_core_types::value::MoveTypeLayout::Vector(Box::new(
                move_core_types::value::MoveTypeLayout::U8,
            )),
            move_core_types::value::MoveTypeLayout::Vector(Box::new(
                move_core_types::value::MoveTypeLayout::U8,
            )),
            move_core_types::value::MoveTypeLayout::Vector(Box::new(
                move_core_types::value::MoveTypeLayout::Vector(Box::new(
                    move_core_types::value::MoveTypeLayout::U8,
                )),
            )),
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        crypto::RoochKeyPair,
        framework::empty::Empty,
        transaction::{Authenticator, RoochTransactionData},
    };

    #[test]
    pub fn test_sign_and_verify() {
        let kp = RoochKeyPair::generate_secp256k1();
        let tx_data = RoochTransactionData::new_for_test(
            ROOCH_FRAMEWORK_ADDRESS.into(),
            0,
            Empty::empty_function_call().into(),
        );
        let auth = Authenticator::bitcoin(&kp, &tx_data);
        let auth_payload = bcs::from_bytes::<AuthPayload>(&auth.payload).unwrap();
        auth_payload.verify(&tx_data).unwrap();
    }
}

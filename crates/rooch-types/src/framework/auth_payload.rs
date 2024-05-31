// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::{
    crypto::{RoochSignature, Signature, SignatureScheme},
    transaction::RoochTransactionData,
};
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

pub const MODULE_NAME: &IdentStr = ident_str!("auth_payload");

const SIGN_INFO_PREFIX: &[u8] = b"Bitcoin Signed Message:\n";
const SIGN_INFO: &[u8] = b"Rooch Transaction\n";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignData {
    pub sign_info_prefix: Vec<u8>,
    pub sign_info: Vec<u8>,
    pub tx_hash_hex: Vec<u8>,
}

impl SignData {
    pub fn new(tx_data: &RoochTransactionData) -> Self {
        let tx_hash_hex = hex::encode(tx_data.tx_hash().as_bytes()).into_bytes();
        SignData {
            sign_info_prefix: SIGN_INFO_PREFIX.to_vec(),
            sign_info: SIGN_INFO.to_vec(),
            tx_hash_hex,
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        // We keep the encode format consistent with the Bitcoin wallet
        let mut data = Vec::new();
        let sign_prefix_len = self.sign_info_prefix.len() as u8;
        if sign_prefix_len > 0 {
            data.push(sign_prefix_len);
            data.extend_from_slice(&self.sign_info_prefix);
        }
        let sign_info_len = self.sign_info.len() as u8;
        let tx_hash_len = self.tx_hash_hex.len() as u8;
        if sign_info_len > 0 {
            data.push(sign_info_len + tx_hash_len);
            data.extend_from_slice(&self.sign_info);
            data.extend_from_slice(&self.tx_hash_hex);
        } else {
            data.push(tx_hash_len);
            data.extend_from_slice(&self.tx_hash_hex);
        }
        data
    }

    pub fn data_hash(&self) -> H256 {
        let data = self.encode();
        sha2_256_of(&data)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthPayload {
    // Message sin
    pub sign: Vec<u8>,
    // Some wallets add magic prefixes, such as unisat adding 'Bitcoin Signed Message:\n'
    pub sign_info_prefix: Vec<u8>,
    // Description of a user-defined signature
    pub sign_info: Vec<u8>,
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
        AuthPayload {
            sign: signature.signature_bytes().to_vec(),
            sign_info_prefix: sign_data.sign_info_prefix,
            sign_info: sign_data.sign_info,
            public_key: signature.public_key_bytes().to_vec(),
            from_address: bitcoin_address.into_bytes(),
        }
    }

    pub fn verify(&self, tx_data: &RoochTransactionData) -> Result<(), anyhow::Error> {
        let pk = Secp256k1PublicKey::from_bytes(&self.public_key)?;
        let tx_hash_hex = hex::encode(tx_data.tx_hash().as_bytes()).into_bytes();
        let sign_data = SignData {
            sign_info_prefix: self.sign_info_prefix.clone(),
            sign_info: self.sign_info.clone(),
            tx_hash_hex,
        };
        let message = sign_data.encode();
        let message_hash = sha2_256_of(&message).0.to_vec();
        let signature = Secp256k1Signature::from_bytes(&self.sign)?;
        pk.verify_with_hash::<Sha256>(&message_hash, &signature)?;
        Ok(())
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

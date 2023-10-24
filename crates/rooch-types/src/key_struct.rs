// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::address::RoochAddress;
use fastcrypto::encoding::{Base64, Encoding};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EncryptionData {
    pub nonce: String,
    pub ciphertext: String,
    pub tag: String,
}
pub struct GenerateNewKeyPair {
    pub mnemonic_phrase: String,
    pub private_key_encryption: EncryptionData,
    pub mnemonic_phrase_encryption: EncryptionData,
}
pub struct GeneratedKeyPair {
    pub address: RoochAddress,
    pub key_pair_data: GenerateNewKeyPair,
}

impl EncryptionData {
    // The data is for test only, please do not use the data for applications.
    pub fn new_for_test() -> EncryptionData {
        let nonce = [202, 31, 86, 27, 113, 29, 104, 237, 218, 110, 152, 145].to_vec();
        let ciphertext = [
            86, 255, 133, 44, 42, 219, 86, 153, 245, 192, 200, 93, 172, 157, 89, 211, 13, 158, 128,
            21, 131, 19, 74, 203, 194, 159, 3, 164, 136, 125, 69, 221,
        ]
        .to_vec();
        let tag = [
            139, 112, 155, 74, 182, 134, 97, 95, 41, 119, 202, 17, 146, 40, 11, 75,
        ]
        .to_vec();

        EncryptionData {
            nonce: Base64::encode(nonce),
            ciphertext: Base64::encode(ciphertext),
            tag: Base64::encode(tag),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MnemonicResult {
    pub mnemonic_phrase: String,
    pub mnemonic_phrase_key: String,
    pub mnemonic_data: MnemonicData,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MnemonicData {
    // pub mnemonic_phrase: String,
    pub addresses: Vec<RoochAddress>,
    pub mnemonic_phrase_encryption: EncryptionData,
}

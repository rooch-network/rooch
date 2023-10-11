// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::address::RoochAddress;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EncryptionData {
    pub nonce: Vec<u8>,
    pub ciphertext: Vec<u8>,
    pub tag: Vec<u8>,
}
pub struct GenerateNewKeyPair {
    pub encryption: EncryptionData,
    pub mnemonic: String,
}
pub struct GeneratedKeyPair {
    pub address: RoochAddress,
    pub result: GenerateNewKeyPair,
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
            nonce,
            ciphertext,
            tag,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MnemonicPhraseGeneratedAddress {
    pub addresses: Vec<RoochAddress>,
}

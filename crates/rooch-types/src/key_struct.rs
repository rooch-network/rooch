// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};

use crate::keypair_type::KeyPairType;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EncryptionData {
    pub hashed_password: String,
    pub nonce: Vec<u8>,
    pub ciphertext: Vec<u8>,
    pub tag: Vec<u8>,
}
pub struct GenerateNewKeyPair {
    pub key_pair_type: KeyPairType,
    pub encryption: EncryptionData,
    pub mnemonic: String,
}
pub struct GeneratedKeyPair<Addr, KeyPair> {
    pub address: Addr,
    pub key_pair: KeyPair,
    pub result: GenerateNewKeyPair,
}

impl EncryptionData {
    // The data is for test only, please do not use the data for applications.
    pub fn new_for_test() -> EncryptionData {
        let hashed_password = "$argon2id$v=19$m=19456,t=2,p=1$zc3Nzc3Nzc3Nzc3Nzc3Nzc3Nzc3Nzc3Nzc3Nzc3Nzc0$RysE6tj+Zu0lLhtKJIedVHrKn9FspulS3vLj/UPaVvQ".to_owned();
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
            hashed_password,
            nonce,
            ciphertext,
            tag,
        }
    }
}

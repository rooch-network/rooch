// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use rand::Rng;
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
    pub fn new_random() -> EncryptionData {
        let hashed_password = generate_random_string(32);
        let nonce = generate_random_bytes(12);
        let ciphertext = generate_random_bytes(32);
        let tag = generate_random_bytes(16);

        EncryptionData {
            hashed_password,
            nonce,
            ciphertext,
            tag,
        }
    }
}

fn generate_random_string(length: usize) -> String {
    let mut rng = rand::thread_rng();
    let random_string: String = (0..length)
        .map(|_| rng.gen_range(b'a'..=b'z') as char)
        .collect();
    random_string
}

fn generate_random_bytes(length: usize) -> Vec<u8> {
    let mut rng = rand::thread_rng();
    let random_bytes: Vec<u8> = (0..length).map(|_| rng.gen()).collect();
    random_bytes
}

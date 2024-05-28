// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::{address::RoochAddress, error::RoochError};
use anyhow::Result;
use argon2::Argon2;
use chacha20poly1305::aead::Aead;
use chacha20poly1305::AeadCore;
use chacha20poly1305::ChaCha20Poly1305;
use chacha20poly1305::KeyInit;
use fastcrypto::encoding::{Base64, Encoding};
use rand::rngs::OsRng;
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

    pub fn encrypt(data: &[u8], password: Option<String>) -> Result<Self> {
        let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);
        let mut output_key_material = [0u8; 32];
        Argon2::default()
            .hash_password_into(
                password.unwrap_or_default().as_bytes(),
                &nonce,
                &mut output_key_material,
            )
            .map_err(|e| RoochError::KeyConversionError(e.to_string()))?;

        let cipher = ChaCha20Poly1305::new_from_slice(&output_key_material)
            .map_err(|e| RoochError::KeyConversionError(e.to_string()))?;

        let ciphertext_with_tag = cipher
            .encrypt(&nonce, data)
            .map_err(|e| RoochError::KeyConversionError(e.to_string()))?;

        let ciphertext = ciphertext_with_tag[..ciphertext_with_tag.len() - 16].to_vec();
        let tag = ciphertext_with_tag[ciphertext_with_tag.len() - 16..].to_vec();

        Ok(EncryptionData {
            nonce: Base64::encode(nonce),
            ciphertext: Base64::encode(ciphertext),
            tag: Base64::encode(tag),
        })
    }

    pub fn decrypt(&self, password: Option<String>) -> Result<Vec<u8>> {
        let nonce = Base64::decode(&self.nonce)
            .map_err(|e| anyhow::Error::new(RoochError::KeyConversionError(e.to_string())))?;
        let ciphertext = Base64::decode(&self.ciphertext)
            .map_err(|e| anyhow::Error::new(RoochError::KeyConversionError(e.to_string())))?;
        let tag = Base64::decode(&self.tag)
            .map_err(|e| anyhow::Error::new(RoochError::KeyConversionError(e.to_string())))?;

        let mut output_key_material = [0u8; 32];
        Argon2::default()
            .hash_password_into(
                password.unwrap_or_default().as_bytes(),
                nonce.as_slice(),
                &mut output_key_material,
            )
            .map_err(|e| RoochError::KeyConversionError(e.to_string()))?;

        let cipher = ChaCha20Poly1305::new_from_slice(&output_key_material)
            .map_err(|e| RoochError::KeyConversionError(e.to_string()))?;

        let mut ciphertext_with_tag = Vec::with_capacity(tag.len() + ciphertext.len());
        ciphertext_with_tag.extend(ciphertext);
        ciphertext_with_tag.extend(tag);

        let data = cipher
            .decrypt(nonce.as_slice().into(), &*ciphertext_with_tag)
            .map_err(|e| RoochError::KeyConversionError(e.to_string()))?;
        Ok(data)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MnemonicResult {
    pub mnemonic_phrase: String,
    pub mnemonic_data: MnemonicData,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MnemonicData {
    pub addresses: Vec<RoochAddress>,
    pub mnemonic_phrase_encryption: EncryptionData,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_data() {
        let data = b"test data";
        let password = "password".to_string();
        let encryption_data = EncryptionData::encrypt(data, Some(password.clone())).unwrap();
        let decrypted_data = encryption_data.decrypt(Some(password)).unwrap();
        assert_eq!(data.to_vec(), decrypted_data);

        let wrong_password = "wrong_password".to_string();
        let result = encryption_data.decrypt(Some(wrong_password));
        assert!(result.is_err());
    }
}

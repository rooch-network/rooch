// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use argon2::password_hash::{PasswordHash, PasswordHasher, SaltString};
use argon2::Argon2;
use argon2::PasswordVerifier;
use bip32::{DerivationPath, XPrv};
use bip39::{Language, Mnemonic, MnemonicType, Seed};
use chacha20poly1305::aead::Aead;
use chacha20poly1305::{AeadCore, ChaCha20Poly1305, KeyInit};
use fastcrypto::ed25519::{Ed25519KeyPair, Ed25519PrivateKey};
use fastcrypto::secp256k1::recoverable::{
    Secp256k1RecoverableKeyPair, Secp256k1RecoverablePrivateKey,
};
use fastcrypto::traits::{KeyPair, ToFromBytes};
use rand::rngs::OsRng;
use rooch_types::address::{EthereumAddress, RoochAddress};
use rooch_types::crypto::RoochKeyPair;
use rooch_types::error::RoochError;
use rooch_types::key_struct::{EncryptionData, GenerateNewKeyPair, GeneratedKeyPair};
use rooch_types::multichain_id::RoochMultiChainID;
use slip10_ed25519::derive_ed25519_private_key;
use std::string::String;

// Purpose constants
pub const DERIVATION_PATH_PURPOSE_ED25519: u32 = 44;
pub const DERIVATION_PATH_PURPOSE_SCHNORR: u32 = 44;
pub const DERIVATION_PATH_PURPOSE_ECDSA: u32 = 54;
pub const DERIVATION_PATH_PURPOSE_SECP256R1: u32 = 74;

type EncryptionKeyResult = Result<(Vec<u8>, Vec<u8>, Vec<u8>), RoochError>;

pub fn encrypt_private_key(
    private_key: &[u8],
    password: Option<&str>,
) -> Result<EncryptionData, RoochError> {
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

    let ciphertext_with_tag = match cipher.encrypt(&nonce, &*private_key) {
        Ok(ciphertext) => ciphertext,
        Err(_) => {
            return Err(RoochError::KeyConversionError(
                "Encryption failed".to_owned(),
            ))
        }
    };

    let ciphertext = ciphertext_with_tag[..ciphertext_with_tag.len() - 16].to_vec();
    let tag = ciphertext_with_tag[ciphertext_with_tag.len() - 16..].to_vec();

    Ok(EncryptionData {
        nonce: nonce.to_vec(),
        ciphertext,
        tag,
    })
}

pub fn decrypt_private_key(
    nonce: &[u8],
    ciphertext: &[u8],
    tag: &[u8],
    password: Option<&str>,
) -> Result<Vec<u8>, RoochError> {
    let mut output_key_material = [0u8; 32];
    Argon2::default()
        .hash_password_into(
            password.unwrap_or_default().as_bytes(),
            nonce,
            &mut output_key_material,
        )
        .map_err(|e| RoochError::KeyConversionError(e.to_string()))?;

    let cipher = ChaCha20Poly1305::new_from_slice(&output_key_material)
        .map_err(|e| RoochError::KeyConversionError(e.to_string()))?;

    let mut ciphertext_with_tag = Vec::with_capacity(tag.len() + ciphertext.len());
    ciphertext_with_tag.extend_from_slice(ciphertext);
    ciphertext_with_tag.extend_from_slice(tag);

    match cipher.decrypt(nonce.into(), &*ciphertext_with_tag) {
        Ok(pk) => Ok(pk),
        Err(_) => Err(RoochError::KeyConversionError(
            "Decryption failed".to_owned(),
        )),
    }
}

pub fn verify_password(
    password: Option<&str>,
    password_hash: String,
) -> Result<bool, argon2::password_hash::Error> {
    let parsed_hash = PasswordHash::new(&password_hash)?;
    Ok(Argon2::default()
        .verify_password(password.unwrap_or_default().as_bytes(), &parsed_hash)
        .is_ok())
}

pub fn hash_password(nonce: &[u8], password: Option<&str>) -> Result<String, RoochError> {
    let salt = SaltString::encode_b64(&nonce)
        .map_err(|e| RoochError::KeyConversionError(e.to_string()))?;
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.unwrap_or_default().as_bytes(), &salt)
        .map_err(|e| RoochError::KeyConversionError(e.to_string()))?
        .to_string();
    Ok(password_hash)
}

pub fn derive_private_key_from_path(
    seed: &[u8],
    derivation_path: Option<DerivationPath>,
) -> Result<Vec<u8>, RoochError> {
    let path = validate_path(derivation_path)?;
    let indexes = path.iter().map(|i| i.into()).collect::<Vec<_>>();
    let derived = derive_ed25519_private_key(seed, &indexes);
    let sk = Ed25519PrivateKey::from_bytes(&derived)
        .map_err(|e| RoochError::SignatureKeyGenError(e.to_string()))?;
    Ok(sk.as_bytes().to_vec())
}

pub fn derive_address_from_private_key(private_key: Vec<u8>) -> Result<RoochAddress, RoochError> {
    let kp: Ed25519KeyPair = Ed25519KeyPair::from(
        Ed25519PrivateKey::from_bytes(&private_key)
            .map_err(|e| RoochError::SignatureKeyGenError(e.to_string()))?,
    );
    let address: RoochAddress = kp.public().into();
    Ok(address)
}

pub fn retrieve_key_pair(
    encryption: &EncryptionData,
    password: Option<String>,
) -> Result<RoochKeyPair, RoochError> {
    // Assuming the password entered herein is correct.
    let private_key = decrypt_private_key(
        &encryption.nonce,
        &encryption.ciphertext,
        &encryption.tag,
        password,
    )?;
    let kp: Ed25519KeyPair = Ed25519KeyPair::from(
        Ed25519PrivateKey::from_bytes(&private_key)
            .map_err(|e| RoochError::SignatureKeyGenError(e.to_string()))?,
    );
    Ok(kp.into())
}

pub fn validate_path(path: Option<DerivationPath>) -> Result<DerivationPath, RoochError> {
    let (purpose, coin_type) = (
        DERIVATION_PATH_PURPOSE_ED25519,
        RoochMultiChainID::Sui as u32,
    );

    match path {
        Some(p) => {
            if let &[p_purpose, p_coin_type, account, change, address] = p.as_ref() {
                if p_purpose == bip32::ChildNumber(purpose)
                    && p_coin_type == bip32::ChildNumber(coin_type)
                    && account.is_hardened()
                    && change.is_hardened()
                    && address.is_hardened()
                {
                    Ok(p)
                } else {
                    Err(RoochError::SignatureKeyGenError("Invalid path".to_owned()))
                }
            } else {
                Err(RoochError::SignatureKeyGenError("Invalid path".to_owned()))
            }
        }
        None => Ok(format!("m/{}'/{}/0'/0'/0'", purpose, coin_type)
            .parse()
            .map_err(|_| RoochError::SignatureKeyGenError("Cannot parse path".to_owned()))?),
    }
}

pub fn generate_new_key_pair(
    derivation_path: Option<DerivationPath>,
    word_length: Option<String>,
    password: Option<String>,
) -> Result<GeneratedKeyPair, anyhow::Error> {
    let mnemonic = Mnemonic::new(parse_word_length(word_length)?, Language::English);
    let seed = Seed::new(&mnemonic, "");

    let sk = derive_private_key_from_path(seed.as_bytes(), derivation_path)?;

    let (nonce, ciphertext, tag) = encrypt_private_key(&sk.clone(), password.clone())
        .expect("Encryption failed for private key");

    let address = derive_address_from_private_key(sk)?;

    let encryption = EncryptionData {
        nonce,
        ciphertext,
        tag,
    };

    let result = GenerateNewKeyPair {
        encryption,
        mnemonic: mnemonic.phrase().to_string(),
    };

    Ok(GeneratedKeyPair { address, result })
}

fn parse_word_length(s: Option<String>) -> Result<MnemonicType, anyhow::Error> {
    match s.as_deref() {
        Some("word12") => Ok(MnemonicType::Words12),
        Some("word15") => Ok(MnemonicType::Words15),
        Some("word18") => Ok(MnemonicType::Words18),
        Some("word21") => Ok(MnemonicType::Words21),
        Some("word24") => Ok(MnemonicType::Words24),
        None => Ok(MnemonicType::Words12),
        _ => Err(anyhow::anyhow!("Invalid word length")),
    }
}

/// Get a keypair from a random encryption data
pub fn get_key_pair_from_red() -> (RoochAddress, EncryptionData) {
    let random_encryption_data = EncryptionData::new_for_test();
    let kp: RoochKeyPair =
        retrieve_key_pair(&random_encryption_data, Some("".to_owned()), None).unwrap();

    ((&kp.public()).into(), random_encryption_data)
}

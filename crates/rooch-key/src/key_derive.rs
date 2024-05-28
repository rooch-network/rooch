// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use argon2::password_hash::{PasswordHash, PasswordHasher, SaltString};
use argon2::Argon2;
use argon2::PasswordVerifier;
use bip39::{Language, Mnemonic, MnemonicType, Seed};
use bitcoin::address::Address;
use bitcoin::bip32::{self, ChildNumber, DerivationPath, Xpriv, Xpub};
use bitcoin::hex::FromHex;
use bitcoin::secp256k1::ffi::types::AlignedType;
use bitcoin::secp256k1::Secp256k1;
use bitcoin::Network;
use chacha20poly1305::aead::Aead;
use chacha20poly1305::{AeadCore, ChaCha20Poly1305, KeyInit};
use fastcrypto::ed25519::{Ed25519KeyPair, Ed25519PrivateKey};
use fastcrypto::encoding::{Base64, Encoding};
use fastcrypto::traits::{KeyPair, ToFromBytes};
use rand::rngs::OsRng;
use rooch_types::address::RoochAddress;
use rooch_types::crypto::RoochKeyPair;
use rooch_types::error::RoochError;
use rooch_types::key_struct::{EncryptionData, GenerateNewKeyPair, GeneratedKeyPair};
use rooch_types::multichain_id::RoochMultiChainID;
use slip10_ed25519::derive_ed25519_private_key;
use std::str::FromStr;
use std::string::String;

// Purpose constants
pub const DERIVATION_PATH_PURPOSE_ED25519: u32 = 44;
pub const DERIVATION_PATH_PURPOSE_SCHNORR: u32 = 44;
pub const DERIVATION_PATH_PURPOSE_ECDSA: u32 = 54;
pub const DERIVATION_PATH_PURPOSE_SECP256R1: u32 = 74;
pub const DERIVATION_PATH_PURPOSE_BIP84: u32 = 84;

pub fn encrypt_key(key: &[u8], password: Option<String>) -> Result<EncryptionData, RoochError> {
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

    let ciphertext_with_tag = match cipher.encrypt(&nonce, key) {
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
        nonce: Base64::encode(nonce),
        ciphertext: Base64::encode(ciphertext),
        tag: Base64::encode(tag),
    })
}

pub fn decrypt_key(
    nonce: &[u8],
    ciphertext: &[u8],
    tag: &[u8],
    password: Option<String>,
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
    password: Option<String>,
    password_hash: String,
) -> Result<bool, RoochError> {
    let parsed_hash = match PasswordHash::new(&password_hash) {
        Ok(parsed) => parsed,
        Err(err) => {
            return Err(RoochError::InvalidPasswordError(format!(
                "PasswordHash error: {}",
                err
            )))
        }
    };
    Ok(Argon2::default()
        .verify_password(password.unwrap_or_default().as_bytes(), &parsed_hash)
        .is_ok())
}

pub fn hash_password(nonce: &[u8], password: Option<String>) -> Result<String, RoochError> {
    let salt =
        SaltString::encode_b64(nonce).map_err(|e| RoochError::KeyConversionError(e.to_string()))?;
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.unwrap_or_default().as_bytes(), &salt)
        .map_err(|e| RoochError::KeyConversionError(e.to_string()))?
        .to_string();
    Ok(password_hash)
}

pub(crate) fn derive_private_key_from_path(
    seed: &[u8],
    derivation_path: Option<DerivationPath>,
) -> Result<Vec<u8>, anyhow::Error> {
    let path = validate_derivation_path(derivation_path)?;
    let indexes = path.to_u32_vec();
    let derived = derive_ed25519_private_key(seed, &indexes);
    let sk = Ed25519PrivateKey::from_bytes(&derived)
        .map_err(|e| RoochError::SignatureKeyGenError(e.to_string()))?;
    Ok(sk.as_bytes().to_vec())
    // // we need secp256k1 context for key derivation
    // let mut buf: Vec<AlignedType> = Vec::new();
    // buf.resize(Secp256k1::preallocate_size(), AlignedType::zeroed());
    // let secp = Secp256k1::preallocated_new(buf.as_mut_slice()).unwrap();

    // // calculate root key from seed
    // let root = Xpriv::new_master(Network::Bitcoin, &seed)?;
    // println!("Root key: {}", root);

    // // derive child xpub
    // //let path = DerivationPath::from_str("84h/0h/0h").unwrap();
    // let child = root.derive_priv(&secp, &path)?;
    // println!("Child at {}: {}", path, child);
    // let xpub = Xpub::from_priv(&secp, &child);
    // println!("Public key at {}: {}", path, xpub);

    // //let zero = ChildNumber::from_normal_idx(0).unwrap();
    // //let public_key = xpub.derive_pub(&secp, &[zero, zero]).unwrap().public_key;
    // Ok(child.to_priv().to_bytes())
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
    let nonce = Base64::decode(&encryption.nonce)
        .map_err(|e| RoochError::KeyConversionError(e.to_string()))?;
    let ciphertext = Base64::decode(&encryption.ciphertext)
        .map_err(|e| RoochError::KeyConversionError(e.to_string()))?;
    let tag = Base64::decode(&encryption.tag)
        .map_err(|e| RoochError::KeyConversionError(e.to_string()))?;

    let private_key = decrypt_key(&nonce, &ciphertext, &tag, password)?;

    let kp = Ed25519KeyPair::from(
        Ed25519PrivateKey::from_bytes(&private_key)
            .map_err(|e| RoochError::SignatureKeyGenError(e.to_string()))?,
    );

    Ok(kp.into())
}

fn validate_derivation_path(path: Option<DerivationPath>) -> Result<DerivationPath, anyhow::Error> {
    let (purpose, coin_type) = (
        DERIVATION_PATH_PURPOSE_ED25519,
        RoochMultiChainID::Rooch as u32,
    );

    match path {
        Some(p) => {
            if let &[p_purpose, p_coin_type, account, change, address] = p.as_ref() {
                if p_purpose == bip32::ChildNumber::from_hardened_idx(purpose)?
                    && p_coin_type == bip32::ChildNumber::from_hardened_idx(coin_type)?
                    && account.is_hardened()
                    && change.is_hardened()
                    && address.is_hardened()
                {
                    Ok(p)
                } else {
                    Err(anyhow::anyhow!(
                        "Invalid derivation path: {}, purpose:{}, coin_type: {}",
                        p,
                        p_purpose,
                        p_coin_type
                    ))
                }
            } else {
                Err(anyhow::anyhow!("Invalid derivation path: {}", p))
            }
        }
        None => Ok(format!("m/{}'/{}'/0'/0'/0'", purpose, coin_type)
            .parse()
            .map_err(|e| anyhow::anyhow!("Cannot parse derivation path, error:{}", e))?),
    }
}

/// Derivation path template
/// Which tells a wallet how to derive a specific key within a tree of keys
/// https://github.com/bitcoin/bips/blob/master/bip-0044.mediawiki
/// for ed25529
/// m / purpose' / coin_type' / account' / change' / address_index'
pub(crate) fn generate_derivation_path(account_index: u32) -> Result<DerivationPath, RoochError> {
    // let (purpose, coin_type) = (
    //     DERIVATION_PATH_PURPOSE_BIP84,
    //     RoochMultiChainID::Bitcoin as u32,
    // );
    let (purpose, coin_type) = (
        DERIVATION_PATH_PURPOSE_ED25519,
        RoochMultiChainID::Rooch as u32,
    );

    DerivationPath::from_str(
        format!("m/{}'/{}'/0'/0'/{}'", purpose, coin_type, account_index).as_str(),
    )
    .map_err(|_| RoochError::SignatureKeyGenError("Cannot parse derivation path".to_owned()))
}

pub fn generate_new_key_pair(
    mnemonic_phrase: Option<String>,
    derivation_path: Option<DerivationPath>,
    word_length: Option<String>,
    password: Option<String>,
) -> Result<GeneratedKeyPair, anyhow::Error> {
    // Reuse the mnemonic phrase to derive new address
    let mnemonic = match mnemonic_phrase {
        Some(phrase) => {
            Mnemonic::validate(phrase.as_str(), Language::English)?;
            Mnemonic::from_phrase(phrase.as_str(), Language::English)?
        }
        None => Mnemonic::new(parse_word_length(word_length)?, Language::English),
    };
    let seed = Seed::new(&mnemonic, "");

    let sk = derive_private_key_from_path(seed.as_bytes(), derivation_path)?;

    let private_key_encryption =
        encrypt_key(&sk, password.clone()).expect("Encryption failed for private key");
    let mnemonic_phrase_encryption = encrypt_key(mnemonic.phrase().as_bytes(), password)
        .expect("Encryption failed for mnemonic phrase");

    let address = derive_address_from_private_key(sk)?;

    let result = GenerateNewKeyPair {
        private_key_encryption,
        mnemonic_phrase_encryption,
        mnemonic_phrase: mnemonic.phrase().to_string(),
    };

    Ok(GeneratedKeyPair {
        address,
        key_pair_data: result,
    })
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
pub(crate) fn generate_key_pair_for_tests() -> (RoochAddress, EncryptionData) {
    let random_encryption_data = EncryptionData::new_for_test();
    let kp: RoochKeyPair = retrieve_key_pair(&random_encryption_data, None).unwrap();

    ((&kp.public()).into(), random_encryption_data)
}

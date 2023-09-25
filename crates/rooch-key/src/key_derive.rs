// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use bip32::XPrv;
use bip32::{ChildNumber, DerivationPath};
use bip39::{Language, Mnemonic, MnemonicType, Seed};
use fastcrypto::ed25519::Ed25519KeyPair;
use fastcrypto::secp256k1::recoverable::{
    Secp256k1RecoverableKeyPair, Secp256k1RecoverablePrivateKey,
};
use fastcrypto::traits::KeyPair;
use fastcrypto::{ed25519::Ed25519PrivateKey, traits::ToFromBytes};
use rooch_types::address::{EthereumAddress, RoochAddress};
use rooch_types::crypto::RoochKeyPair;
use rooch_types::error::RoochError;
use rooch_types::multichain_id::RoochMultiChainID;
use slip10_ed25519::derive_ed25519_private_key;
use std::string::String;

use crate::keypair::KeyPairType;

use chacha20poly1305::{
    aead::{Aead, KeyInit},
    ChaCha20Poly1305,
};

// Purpose
/// Ed25519 follows SLIP-0010 using hardened path: m/44'/784'/0'/0'/{index}'
/// Note that the purpose node is used to distinguish signature auth validator ids.
pub const DERVIATION_PATH_PURPOSE_ED25519: u32 = 44;
/// BIP39 is used to generate mnemonic seed words and derive a binary seed from them.
/// BIP32 is used to derive the path m/44'/1237'/<account>'/0/0 (according to the Nostr entry on SLIP44).
/// A basic client can simply use an account of 0 to derive a single key. For more advanced use-cases you can increment account, allowing generation of practically infinite keys from the 5-level path with hardened derivation.
/// Other types of clients can still get fancy and use other derivation paths for their own other purposes.
pub const DERVIATION_PATH_PURPOSE_SCHNORR: u32 = 44;
pub const DERVIATION_PATH_PURPOSE_ECDSA: u32 = 54;
pub const DERVIATION_PATH_PURPOSE_SECP256R1: u32 = 74;

pub trait CoinOperations<Addr, KeyPair> {
    fn derive_key_pair_from_path(
        &self,
        seed: &[u8],
        derivation_path: Option<DerivationPath>,
    ) -> Result<(Addr, KeyPair), RoochError>;
}

impl CoinOperations<RoochAddress, RoochKeyPair> for KeyPairType {
    fn derive_key_pair_from_path(
        &self,
        seed: &[u8],
        derivation_path: Option<DerivationPath>,
    ) -> Result<(RoochAddress, RoochKeyPair), RoochError> {
        let path = validate_path(self, derivation_path)?;
        let indexes = path.into_iter().map(|i| i.into()).collect::<Vec<_>>();
        let derived = derive_ed25519_private_key(seed, &indexes);
        let sk = Ed25519PrivateKey::from_bytes(&derived)
            .map_err(|e| RoochError::SignatureKeyGenError(e.to_string()))?;
        let (_, encrypted_ciphertext_pk) 
            = encrypt_private_key(sk.as_bytes(), seed).expect("Encryption failed");
        let kp: Ed25519KeyPair = Ed25519KeyPair::from(
            Ed25519PrivateKey::from_bytes(&encrypted_ciphertext_pk)
                .map_err(|e| RoochError::SignatureKeyGenError(e.to_string()))?,
        );
        let address: RoochAddress = kp.public().into();
        Ok((address, kp.into())) // Cast to KeyPair
    }
}

impl CoinOperations<EthereumAddress, Secp256k1RecoverableKeyPair> for KeyPairType {
    fn derive_key_pair_from_path(
        &self,
        seed: &[u8],
        derivation_path: Option<DerivationPath>,
    ) -> Result<(EthereumAddress, Secp256k1RecoverableKeyPair), RoochError> {
        let path = validate_path(self, derivation_path)?;
        let child_xprv = XPrv::derive_from_path(seed, &path)
            .map_err(|e| RoochError::SignatureKeyGenError(e.to_string()))?;
        let (_, encrypted_ciphertext_pk) =
            encrypt_private_key(child_xprv.private_key().to_bytes().as_slice(), seed)
                .expect("Encryption failed");
        let kp = Secp256k1RecoverableKeyPair::from(
            Secp256k1RecoverablePrivateKey::from_bytes(&encrypted_ciphertext_pk)
                .map_err(|e| RoochError::SignatureKeyGenError(e.to_string()))?,
        );
        let address: EthereumAddress = EthereumAddress::from(kp.public.clone());
        Ok((address, kp)) // Cast to KeyPair
    }
}

pub fn validate_path(
    key_pair_type: &KeyPairType,
    path: Option<DerivationPath>,
) -> Result<DerivationPath, RoochError> {
    match key_pair_type {
        KeyPairType::RoochKeyPairType => {
            const DERIVATION_PATH_COIN_TYPE_SUI: u32 = RoochMultiChainID::Sui as u32;
            // Rooch key pair type
            match path {
                Some(p) => {
                    // The derivation path must be hardened at all levels with purpose = 44, coin_type = 784
                    if let &[purpose, coin_type, account, change, address] = p.as_ref() {
                        if Some(purpose)
                            == ChildNumber::new(DERVIATION_PATH_PURPOSE_ED25519, true).ok()
                            && Some(coin_type)
                                == ChildNumber::new(DERIVATION_PATH_COIN_TYPE_SUI, true).ok()
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
                None => Ok(format!(
                "m/{DERVIATION_PATH_PURPOSE_ED25519}'/{DERIVATION_PATH_COIN_TYPE_SUI}'/0'/0'/0'"
            )
                .parse()
                .map_err(|_| RoochError::SignatureKeyGenError("Cannot parse path".to_owned()))?),
            }
        }
        KeyPairType::EthereumKeyPairType => {
            const DERIVATION_PATH_COIN_TYPE_SUI: u32 = RoochMultiChainID::Sui as u32;
            // Ethereum key pair type
            match path {
                Some(p) => {
                    // The derivation path must be hardened at all levels with purpose = 54, coin_type = 784
                    if let &[purpose, coin_type, account, change, address] = p.as_ref() {
                        if Some(purpose)
                            == ChildNumber::new(DERVIATION_PATH_PURPOSE_ECDSA, true).ok()
                            && Some(coin_type)
                                == ChildNumber::new(DERIVATION_PATH_COIN_TYPE_SUI, true).ok()
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
                None => Ok(format!(
                    "m/{DERVIATION_PATH_PURPOSE_ECDSA}'/{DERIVATION_PATH_COIN_TYPE_SUI}'/0'/0'/0'"
                )
                .parse()
                .map_err(|_| RoochError::SignatureKeyGenError("Cannot parse path".to_owned()))?),
            }
        }
    }
}

pub fn generate_new_key_pair<Addr, KeyPair>(
    key_pair_type: KeyPairType,
    derivation_path: Option<DerivationPath>,
    word_length: Option<String>,
    password: Option<String>,
) -> Result<(Addr, KeyPair, KeyPairType, String), anyhow::Error>
where
    KeyPairType: CoinOperations<Addr, KeyPair>,
{
    let mnemonic = Mnemonic::new(parse_word_length(word_length)?, Language::English);
    let password_str = password.as_deref().unwrap_or("");
    let seed = Seed::new(&mnemonic, password_str);

    let (address, key_pair) =
        key_pair_type.derive_key_pair_from_path(seed.as_bytes(), derivation_path)?;

    Ok((
        address,
        key_pair,
        key_pair_type,
        mnemonic.phrase().to_string(),
    ))
}

// Encrypt the private key using ChaCha20Poly1305
pub fn encrypt_private_key(private_key: &[u8], seed: &[u8]) -> Result<(Vec<u8>, Vec<u8>), anyhow::Error> {
    // Calculate nonce and encryption key from seed
    let mut nonce = [0u8; 12];
    nonce.copy_from_slice(&seed[..12]);
    let mut encryption_key = [0u8; 32];
    encryption_key.copy_from_slice(&seed[12..44]);
    // Create a ChaCha20Poly1305 cipher with the key
    let cipher = ChaCha20Poly1305::new_from_slice(&encryption_key)?;
    // Encrypt the private key data to a ciphertext with a tag
    let ciphertext_with_tag = match cipher.encrypt(&nonce.into(), private_key.as_ref()) {
        Ok(ciphertext) => ciphertext,
        Err(_) => return Err(anyhow::Error::msg("Encryption failed")),
    };

    // Extract the ciphertext without the tag
    let ciphertext = ciphertext_with_tag[..ciphertext_with_tag.len() - 16].to_vec();
    
    Ok((ciphertext_with_tag, ciphertext))
}

// Decrypt the private key using ChaCha20Poly1305
pub fn decrypt_private_key(ciphertext_with_tag: &[u8], seed: &[u8]) -> Result<Vec<u8>, anyhow::Error> {
    // Calculate nonce and encryption key from seed
    let mut nonce = [0u8; 12];
    nonce.copy_from_slice(&seed[..12]);
    let mut encryption_key = [0u8; 32];
    encryption_key.copy_from_slice(&seed[12..44]);

    // Create a ChaCha20Poly1305 cipher with the key
    let cipher = ChaCha20Poly1305::new_from_slice(&encryption_key)?;

    // Decrypt the ciphertext to a private key
    let private_key = match cipher.decrypt(&nonce.into(), ciphertext_with_tag) {
        Ok(pk) => pk,
        Err(_) => return Err(anyhow::Error::msg("Decryption failed")),
    };

    Ok(private_key.to_vec())
}

fn parse_word_length(s: Option<String>) -> Result<MnemonicType, anyhow::Error> {
    match s {
        None => Ok(MnemonicType::Words12),
        Some(s) => match s.as_str() {
            "word12" => Ok(MnemonicType::Words12),
            "word15" => Ok(MnemonicType::Words15),
            "word18" => Ok(MnemonicType::Words18),
            "word21" => Ok(MnemonicType::Words21),
            "word24" => Ok(MnemonicType::Words24),
            _ => anyhow::bail!("Invalid word length"),
        },
    }
}

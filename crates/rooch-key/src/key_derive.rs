// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use bip32::XPrv;
use bip32::{ChildNumber, DerivationPath};
use bip39::{Language, Mnemonic, MnemonicType, Seed};
use chacha20poly1305::AeadCore;
use fastcrypto::ed25519::Ed25519KeyPair;
use fastcrypto::secp256k1::recoverable::{
    Secp256k1RecoverableKeyPair, Secp256k1RecoverablePrivateKey,
};
use fastcrypto::traits::KeyPair;
use fastcrypto::{ed25519::Ed25519PrivateKey, traits::ToFromBytes};
use rand::rngs::OsRng;
use rooch_types::address::{EthereumAddress, RoochAddress};
use rooch_types::crypto::RoochKeyPair;
use rooch_types::error::RoochError;
use rooch_types::key_struct::{EncryptionData, GenerateNewKeyPair, GeneratedKeyPair};
use rooch_types::keypair_type::KeyPairType;
use rooch_types::multichain_id::RoochMultiChainID;
use slip10_ed25519::derive_ed25519_private_key;
use std::string::String;

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

type EncryptionKeyResult = Result<(Vec<u8>, Vec<u8>, Vec<u8>), RoochError>;

pub trait CoinOperations<Addr, KeyPair, PrivKey> {
    fn derive_private_key_from_path(
        &self,
        seed: &[u8],
        derivation_path: Option<DerivationPath>,
    ) -> Result<PrivKey, RoochError>;
    fn derive_key_pair_from_ciphertext(
        &self,
        ciphertext: Vec<u8>,
    ) -> Result<(Addr, KeyPair), RoochError>;
    fn retrieve_key_pair(
        &self,
        encryption: &EncryptionData,
        password: Option<String>,
    ) -> Result<KeyPair, RoochError>;
    fn encrypt_private_key(
        &self,
        private_key: PrivKey,
        password: Option<String>,
    ) -> EncryptionKeyResult;
    fn hash_password(
        &self,
        private_key: PrivKey,
        password: Option<String>,
    ) -> Result<String, RoochError>;
}

impl CoinOperations<RoochAddress, RoochKeyPair, Ed25519PrivateKey> for KeyPairType {
    fn derive_private_key_from_path(
        &self,
        seed: &[u8],
        derivation_path: Option<DerivationPath>,
    ) -> Result<Ed25519PrivateKey, RoochError> {
        let path = validate_path(self, derivation_path)?;
        let indexes = path.into_iter().map(|i| i.into()).collect::<Vec<_>>();
        let derived = derive_ed25519_private_key(seed, &indexes);
        let sk = Ed25519PrivateKey::from_bytes(&derived)
            .map_err(|e| RoochError::SignatureKeyGenError(e.to_string()))?;
        Ok(sk)
    }

    fn derive_key_pair_from_ciphertext(
        &self,
        ciphertext: Vec<u8>,
    ) -> Result<(RoochAddress, RoochKeyPair), RoochError> {
        let kp: Ed25519KeyPair = Ed25519KeyPair::from(
            Ed25519PrivateKey::from_bytes(&ciphertext)
                .map_err(|e| RoochError::SignatureKeyGenError(e.to_string()))?,
        );
        let address: RoochAddress = kp.public().into();
        Ok((address, kp.into()))
    }

    fn retrieve_key_pair(
        &self,
        encryption: &EncryptionData,
        password: Option<String>,
    ) -> Result<RoochKeyPair, RoochError> {
        let is_verified = verify_password(password.clone(), encryption.hashed_password.clone())
            .expect("Verification failed for password");
        if is_verified {
            let private_key = decrypt_private_key(
                &encryption.nonce,
                &encryption.ciphertext,
                &encryption.tag,
                password,
            )
            .expect("Decryption failed for private key");
            let kp: Ed25519KeyPair = Ed25519KeyPair::from(
                Ed25519PrivateKey::from_bytes(&private_key)
                    .map_err(|e| RoochError::SignatureKeyGenError(e.to_string()))?,
            );
            Ok(kp.into())
        } else {
            Err(RoochError::InvalidPasswordError(
                "Password is invalid".to_owned(),
            ))
        }
    }

    // Encrypt the private key using ChaCha20Poly1305 and Argon2
    fn encrypt_private_key(
        &self,
        private_key: Ed25519PrivateKey,
        password: Option<String>,
    ) -> Result<(Vec<u8>, Vec<u8>, Vec<u8>), RoochError> {
        // 96-bits; unique per message
        let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);

        // Derive the key material using nonce and password
        let mut output_key_material = [0u8; 32]; // Can be any desired size
        Argon2::default()
            .hash_password_into(
                password.unwrap_or("".to_owned()).as_bytes(),
                &nonce,
                &mut output_key_material,
            )
            .map_err(|e| RoochError::KeyConversionError(e.to_string()))?;

        // Create a ChaCha20Poly1305 cipher with the key material from password
        let cipher = ChaCha20Poly1305::new_from_slice(&output_key_material)
            .map_err(|e| RoochError::KeyConversionError(e.to_string()))?;

        // Encrypt the private key data to a ciphertext with a tag
        let ciphertext_with_tag = match cipher.encrypt(&nonce, private_key.as_bytes()) {
            Ok(ciphertext) => ciphertext,
            Err(_) => {
                return Err(RoochError::KeyConversionError(
                    "Encryption failed".to_owned(),
                ))
            }
        };

        // Extract the ciphertext without the tag
        let ciphertext = ciphertext_with_tag[..ciphertext_with_tag.len() - 16].to_vec();

        // Extract the tag (last 16 bytes)
        // The tag is useless here for deriving the key pair in function derive_key_pair_from_ciphertext
        // Because from_bytes needs exactly 32 bytes input to convert to the private keys
        // Poly1305 is a one-time authenticator designed by D. J. Bernstein.
        // Poly1305 takes a 32-byte one-time key and a message and produces a
        // 16-byte tag. This tag is used to authenticate the message.
        // https://www.rfc-editor.org/rfc/rfc7539
        let tag = {
            let start = ciphertext_with_tag.len() - 16;
            let end = ciphertext_with_tag.len();
            let mut tag = Vec::with_capacity(16);
            tag.extend_from_slice(&ciphertext_with_tag[start..end]);
            tag
        };

        Ok((nonce.to_vec(), ciphertext, tag))
    }

    // Hash the password using Argon2
    fn hash_password(
        &self,
        private_key: Ed25519PrivateKey,
        password: Option<String>,
    ) -> Result<String, RoochError> {
        // Encode private key into a salt
        let salt = SaltString::encode_b64(private_key.as_bytes())
            .map_err(|e| RoochError::KeyConversionError(e.to_string()))?;
        // Argon2 with default params (Argon2id v19)
        let argon2 = Argon2::default();
        // Hash password to PHC string ($argon2id$v=19$...)
        let password_hash = argon2
            .hash_password(password.unwrap_or("".to_owned()).as_bytes(), &salt)
            .map_err(|e| RoochError::KeyConversionError(e.to_string()))?
            .to_string();
        Ok(password_hash)
    }
}

impl CoinOperations<EthereumAddress, Secp256k1RecoverableKeyPair, Secp256k1RecoverablePrivateKey>
    for KeyPairType
{
    fn derive_private_key_from_path(
        &self,
        seed: &[u8],
        derivation_path: Option<DerivationPath>,
    ) -> Result<Secp256k1RecoverablePrivateKey, RoochError> {
        let path = validate_path(self, derivation_path)?;
        let child_xprv = XPrv::derive_from_path(seed, &path)
            .map_err(|e| RoochError::SignatureKeyGenError(e.to_string()))?;
        let sk = Secp256k1RecoverablePrivateKey::from_bytes(
            child_xprv.private_key().to_bytes().as_slice(),
        )
        .map_err(|e| RoochError::SignatureKeyGenError(e.to_string()))?;
        Ok(sk)
    }

    fn derive_key_pair_from_ciphertext(
        &self,
        ciphertext: Vec<u8>,
    ) -> Result<(EthereumAddress, Secp256k1RecoverableKeyPair), RoochError> {
        let kp = Secp256k1RecoverableKeyPair::from(
            Secp256k1RecoverablePrivateKey::from_bytes(&ciphertext)
                .map_err(|e| RoochError::SignatureKeyGenError(e.to_string()))?,
        );
        let address: EthereumAddress = EthereumAddress::from(kp.public.clone());
        Ok((address, kp)) // Cast to KeyPair
    }

    fn retrieve_key_pair(
        &self,
        encryption: &EncryptionData,
        password: Option<String>,
    ) -> Result<Secp256k1RecoverableKeyPair, RoochError> {
        let is_verified = verify_password(password.clone(), encryption.hashed_password.clone())
            .expect("Verification failed for password");
        if is_verified {
            let private_key = decrypt_private_key(
                &encryption.nonce,
                &encryption.ciphertext,
                &encryption.tag,
                password,
            )
            .expect("Decryption failed for private key");
            let kp: Secp256k1RecoverableKeyPair = Secp256k1RecoverableKeyPair::from(
                Secp256k1RecoverablePrivateKey::from_bytes(&private_key)
                    .map_err(|e| RoochError::SignatureKeyGenError(e.to_string()))?,
            );
            Ok(kp)
        } else {
            Err(RoochError::InvalidPasswordError(
                "Password is invalid".to_owned(),
            ))
        }
    }

    // Encrypt the private key using ChaCha20Poly1305
    fn encrypt_private_key(
        &self,
        private_key: Secp256k1RecoverablePrivateKey,
        password: Option<String>,
    ) -> Result<(Vec<u8>, Vec<u8>, Vec<u8>), RoochError> {
        // 96-bits; unique per message
        let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);

        // Derive the key material using nonce and password
        let mut output_key_material = [0u8; 32]; // Can be any desired size
        Argon2::default()
            .hash_password_into(
                password.unwrap_or("".to_owned()).as_bytes(),
                &nonce,
                &mut output_key_material,
            )
            .map_err(|e| RoochError::KeyConversionError(e.to_string()))?;

        // Create a ChaCha20Poly1305 cipher with the key material from password
        let cipher = ChaCha20Poly1305::new_from_slice(&output_key_material)
            .map_err(|e| RoochError::KeyConversionError(e.to_string()))?;

        // Encrypt the private key data to a ciphertext with a tag
        let ciphertext_with_tag = match cipher.encrypt(&nonce, private_key.as_bytes()) {
            Ok(ciphertext) => ciphertext,
            Err(_) => {
                return Err(RoochError::KeyConversionError(
                    "Encryption failed".to_owned(),
                ))
            }
        };

        // Extract the ciphertext without the tag
        let ciphertext = ciphertext_with_tag[..ciphertext_with_tag.len() - 16].to_vec();

        // Extract the tag (last 16 bytes)
        // The tag is useless here for deriving the key pair in function derive_key_pair_from_ciphertext
        // Because from_bytes needs exactly 32 bytes input to convert to the private keys
        // Poly1305 is a one-time authenticator designed by D. J. Bernstein.
        // Poly1305 takes a 32-byte one-time key and a message and produces a
        // 16-byte tag. This tag is used to authenticate the message.
        // https://www.rfc-editor.org/rfc/rfc7539
        let tag = {
            let start = ciphertext_with_tag.len() - 16;
            let end = ciphertext_with_tag.len();
            let mut tag = Vec::with_capacity(16);
            tag.extend_from_slice(&ciphertext_with_tag[start..end]);
            tag
        };

        Ok((nonce.to_vec(), ciphertext, tag))
    }

    // Hash the password using Argon2
    fn hash_password(
        &self,
        private_key: Secp256k1RecoverablePrivateKey,
        password: Option<String>,
    ) -> Result<String, RoochError> {
        // Encode private key into a salt
        let salt = SaltString::encode_b64(private_key.as_bytes())
            .map_err(|e| RoochError::KeyConversionError(e.to_string()))?;
        // Argon2 with default params (Argon2id v19)
        let argon2 = Argon2::default();
        // Hash password to PHC string ($argon2id$v=19$...)
        let password_hash = argon2
            .hash_password(password.unwrap_or("".to_owned()).as_bytes(), &salt)
            .map_err(|e| RoochError::KeyConversionError(e.to_string()))?
            .to_string();
        Ok(password_hash)
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

pub fn generate_new_key_pair<Addr, KeyPair, PrivKey>(
    key_pair_type: KeyPairType,
    derivation_path: Option<DerivationPath>,
    word_length: Option<String>,
    password: Option<String>,
) -> Result<GeneratedKeyPair<Addr, KeyPair>, anyhow::Error>
where
    KeyPairType: CoinOperations<Addr, KeyPair, PrivKey>,
{
    let mnemonic = Mnemonic::new(parse_word_length(word_length)?, Language::English);
    let seed = Seed::new(&mnemonic, "");

    let sk =
        key_pair_type.derive_private_key_from_path(seed.as_bytes(), derivation_path.clone())?;

    let (nonce, ciphertext, tag) = key_pair_type
        .encrypt_private_key(sk, password.clone())
        .expect("Encryption failed for private key");

    let sk_clone = key_pair_type.derive_private_key_from_path(seed.as_bytes(), derivation_path)?;

    let hashed_password = key_pair_type
        .hash_password(sk_clone, password)
        .expect("Encryption failed for password");

    let (address, key_pair) = key_pair_type.derive_key_pair_from_ciphertext(ciphertext.clone())?;

    let encryption = EncryptionData {
        hashed_password,
        nonce,
        ciphertext,
        tag,
    };

    let result = GenerateNewKeyPair {
        key_pair_type,
        encryption,
        mnemonic: mnemonic.phrase().to_string(),
    };

    Ok(GeneratedKeyPair {
        address,
        key_pair,
        result,
    })
}

// Verify the password against Argon2
pub fn verify_password(
    password: Option<String>,
    password_hash: String,
) -> Result<bool, argon2::password_hash::Error> {
    let parsed_hash = PasswordHash::new(&password_hash)?;
    Ok(Argon2::default()
        .verify_password(password.unwrap_or("".to_owned()).as_bytes(), &parsed_hash)
        .is_ok())
}

// Decrypt the private key using ChaCha20Poly1305 and Argon2
pub fn decrypt_private_key(
    nonce: &[u8],
    ciphertext: &[u8],
    tag: &[u8],
    password: Option<String>,
) -> Result<Vec<u8>, anyhow::Error> {
    // Derive the key material using nonce and password
    let mut output_key_material = [0u8; 32]; // Can be any desired size
    Argon2::default()
        .hash_password_into(
            password.unwrap_or("".to_owned()).as_bytes(),
            nonce,
            &mut output_key_material,
        )
        .map_err(|e| RoochError::KeyConversionError(e.to_string()))?;

    // Create a ChaCha20Poly1305 cipher with the key material from password
    let cipher = ChaCha20Poly1305::new_from_slice(&output_key_material)?;

    // Concatenate the tag and the ciphertext to reconstruct ciphertext_with_tag
    let mut ciphertext_with_tag = Vec::with_capacity(tag.len() + ciphertext.len());
    ciphertext_with_tag.extend_from_slice(ciphertext);
    ciphertext_with_tag.extend_from_slice(tag);

    // Decrypt the ciphertext_with_tag to a private key
    let private_key = match cipher.decrypt(nonce.into(), &*ciphertext_with_tag) {
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

/// Get a rooch keypair from a random encryption data
pub fn get_rooch_key_pair_from_red() -> (RoochAddress, EncryptionData) {
    let random_encryption_data = EncryptionData::new_for_test();
    let key_pair_type = KeyPairType::RoochKeyPairType;
    let kp: RoochKeyPair = key_pair_type
        .retrieve_key_pair(&random_encryption_data, Some("".to_owned()))
        .unwrap();

    ((&kp.public()).into(), random_encryption_data)
}

/// Get an ethereum keypair from a random encryption data
pub fn get_ethereum_key_pair_from_red() -> (EthereumAddress, EncryptionData) {
    let random_encryption_data = EncryptionData::new_for_test();
    let key_pair_type = KeyPairType::EthereumKeyPairType;

    let kp: Secp256k1RecoverableKeyPair = key_pair_type
        .retrieve_key_pair(&random_encryption_data, Some("".to_owned()))
        .unwrap();
    let address = EthereumAddress::from(kp.public().clone());

    (address, random_encryption_data)
}

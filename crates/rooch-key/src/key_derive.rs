// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::anyhow;
use bip32::XPrv;
use bip32::{ChildNumber, DerivationPath};
use bip39::{Language, Mnemonic, MnemonicType, Seed};
use fastcrypto::ed25519::Ed25519KeyPair;
use fastcrypto::secp256k1::recoverable::{
    Secp256k1RecoverableKeyPair, Secp256k1RecoverablePrivateKey,
};
use fastcrypto::secp256k1::schnorr::{SchnorrKeyPair, SchnorrPrivateKey};
use fastcrypto::secp256k1::{Secp256k1KeyPair, Secp256k1PrivateKey};
use fastcrypto::{
    ed25519::Ed25519PrivateKey,
    traits::{KeyPair, ToFromBytes},
};
use rooch_types::address::RoochAddress;
use rooch_types::crypto::{BuiltinScheme, RoochKeyPair};
use rooch_types::error::RoochError;
use slip10_ed25519::derive_ed25519_private_key;
use std::string::String;

// Coin type
pub const DERIVATION_PATH_COIN_TYPE_BTC: u64 = 0;
pub const DERIVATION_PATH_COIN_TYPE_ETH: u64 = 60;
pub const DERIVATION_PATH_COIN_TYPE_SUI: u64 = 784;
pub const DERIVATION_PATH_COIN_TYPE_LBTC: u64 = 998;
pub const DERIVATION_PATH_COIN_TYPE_NOSTR: u64 = 1237;
// Purpose
/// Ed25519 follows SLIP-0010 using hardened path: m/44'/784'/0'/0'/{index}'
/// Note that the purpose node is used to distinguish signature schemes.
pub const DERVIATION_PATH_PURPOSE_ED25519: u32 = 44;
/// BIP39 is used to generate mnemonic seed words and derive a binary seed from them.
/// BIP32 is used to derive the path m/44'/1237'/<account>'/0/0 (according to the Nostr entry on SLIP44).
/// A basic client can simply use an account of 0 to derive a single key. For more advanced use-cases you can increment account, allowing generation of practically infinite keys from the 5-level path with hardened derivation.
/// Other types of clients can still get fancy and use other derivation paths for their own other purposes.
pub const DERVIATION_PATH_PURPOSE_SCHNORR: u32 = 44;
pub const DERVIATION_PATH_PURPOSE_ECDSA: u32 = 54;
pub const DERVIATION_PATH_PURPOSE_SECP256R1: u32 = 74;

pub fn derive_key_pair_from_path(
    seed: &[u8],
    derivation_path: Option<DerivationPath>,
    crypto_scheme: &BuiltinScheme,
) -> Result<(RoochAddress, RoochKeyPair), RoochError> {
    let path = validate_path(crypto_scheme, derivation_path)?;
    match crypto_scheme {
        BuiltinScheme::Ed25519 => {
            let indexes = path.into_iter().map(|i| i.into()).collect::<Vec<_>>();
            let derived = derive_ed25519_private_key(seed, &indexes);
            let sk = Ed25519PrivateKey::from_bytes(&derived)
                .map_err(|e| RoochError::SignatureKeyGenError(e.to_string()))?;
            let kp: Ed25519KeyPair = sk.into();
            Ok((kp.public().into(), RoochKeyPair::Ed25519(kp)))
        }
        BuiltinScheme::MultiEd25519 => {
            todo!()
        }
        BuiltinScheme::Ecdsa => {
            let child_xprv = XPrv::derive_from_path(seed, &path)
                .map_err(|e| RoochError::SignatureKeyGenError(e.to_string()))?;
            let kp = Secp256k1KeyPair::from(
                Secp256k1PrivateKey::from_bytes(child_xprv.private_key().to_bytes().as_slice())
                    .map_err(|e| RoochError::SignatureKeyGenError(e.to_string()))?,
            );
            Ok((kp.public().into(), RoochKeyPair::Ecdsa(kp)))
        }
        BuiltinScheme::EcdsaRecoverable => {
            let child_xprv = XPrv::derive_from_path(seed, &path)
                .map_err(|e| RoochError::SignatureKeyGenError(e.to_string()))?;
            let kp = Secp256k1RecoverableKeyPair::from(
                Secp256k1RecoverablePrivateKey::from_bytes(
                    child_xprv.private_key().to_bytes().as_slice(),
                )
                .map_err(|e| RoochError::SignatureKeyGenError(e.to_string()))?,
            );
            Ok((kp.public().into(), RoochKeyPair::EcdsaRecoverable(kp)))
        }
        BuiltinScheme::Schnorr => {
            let child_xprv = XPrv::derive_from_path(seed, &path)
                .map_err(|e| RoochError::SignatureKeyGenError(e.to_string()))?;
            let kp = SchnorrKeyPair::from(
                SchnorrPrivateKey::from_bytes(child_xprv.private_key().to_bytes().as_slice())
                    .map_err(|e| RoochError::SignatureKeyGenError(e.to_string()))?,
            );
            Ok((kp.public().into(), RoochKeyPair::Schnorr(kp)))
        }
    }
}

pub fn validate_path(
    crypto_scheme: &BuiltinScheme,
    path: Option<DerivationPath>,
) -> Result<DerivationPath, RoochError> {
    // The derivation path must be hardened at all levels with purpose = 44, coin_type = 784
    match crypto_scheme {
        BuiltinScheme::Ed25519 => {
            match path {
                Some(p) => {
                    // The derivation path must be hardened at all levels with purpose = 44, coin_type = 784
                    if let &[purpose, coin_type, account, change, address] = p.as_ref() {
                        if Some(purpose)
                            == ChildNumber::new(DERVIATION_PATH_PURPOSE_ED25519, true).ok()
                            && Some(coin_type)
                                == ChildNumber::new(
                                    DERIVATION_PATH_COIN_TYPE_SUI.try_into().unwrap(),
                                    true,
                                )
                                .ok()
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
        BuiltinScheme::MultiEd25519 => {
            todo!()
        }
        BuiltinScheme::Ecdsa => {
            match path {
                Some(p) => {
                    // The derivation path must be hardened at all levels with purpose = 44, coin_type = 784
                    if let &[purpose, coin_type, account, change, address] = p.as_ref() {
                        if Some(purpose)
                            == ChildNumber::new(DERVIATION_PATH_PURPOSE_ECDSA, true).ok()
                            && Some(coin_type)
                                == ChildNumber::new(
                                    DERIVATION_PATH_COIN_TYPE_SUI.try_into().unwrap(),
                                    true,
                                )
                                .ok()
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
        BuiltinScheme::EcdsaRecoverable => {
            match path {
                Some(p) => {
                    // The derivation path must be hardened at all levels with purpose = 44, coin_type = 784
                    if let &[purpose, coin_type, account, change, address] = p.as_ref() {
                        if Some(purpose)
                            == ChildNumber::new(DERVIATION_PATH_PURPOSE_ECDSA, true).ok()
                            && Some(coin_type)
                                == ChildNumber::new(
                                    DERIVATION_PATH_COIN_TYPE_SUI.try_into().unwrap(),
                                    true,
                                )
                                .ok()
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
        BuiltinScheme::Schnorr => {
            match path {
                Some(p) => {
                    // The derivation path must be hardened at all levels with purpose = 44, coin_type = 784
                    if let &[purpose, coin_type, account, change, address] = p.as_ref() {
                        if Some(purpose) == ChildNumber::new(DERVIATION_PATH_PURPOSE_SCHNORR, true).ok()
                            && Some(coin_type) == ChildNumber::new(DERIVATION_PATH_COIN_TYPE_NOSTR.try_into().unwrap(), true).ok()
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
                    // for a signle key
                    "m/{DERVIATION_PATH_PURPOSE_SCHNORR}'/{DERIVATION_PATH_COIN_TYPE_NOSTR}'/0'/0'/0'"
                )
                .parse()
                .map_err(|_| RoochError::SignatureKeyGenError("Cannot parse path".to_owned()))?),
            }
        }
    }
}

pub fn generate_new_key(
    crypto_scheme: BuiltinScheme,
    derivation_path: Option<DerivationPath>,
    word_length: Option<String>,
) -> Result<(RoochAddress, RoochKeyPair, BuiltinScheme, String), anyhow::Error> {
    let mnemonic = Mnemonic::new(parse_word_length(word_length)?, Language::English);
    let seed = Seed::new(&mnemonic, "");
    match derive_key_pair_from_path(seed.as_bytes(), derivation_path, &crypto_scheme) {
        Ok((address, kp)) => Ok((address, kp, crypto_scheme, mnemonic.phrase().to_string())),
        Err(e) => Err(anyhow!("Failed to generate keypair: {:?}", e)),
    }
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

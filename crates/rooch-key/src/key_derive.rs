// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::anyhow;
use bip32::{ChildNumber, DerivationPath};
use bip39::{Language, Mnemonic, MnemonicType, Seed};
use fastcrypto::ed25519::Ed25519KeyPair;
use fastcrypto::hash::HashFunction;
use fastcrypto::{
    ed25519::Ed25519PrivateKey,
    traits::{KeyPair, ToFromBytes},
};
use moveos_types::h256::H256;
use rooch_types::account::{DefaultHash, RoochKeyPair, SignatureScheme};
use rooch_types::address::RoochAddress;
use rooch_types::error::RoochError;
use slip10_ed25519::derive_ed25519_private_key;
use std::string::String;

pub const DERIVATION_PATH_COIN_TYPE: u32 = 784;
pub const DERVIATION_PATH_PURPOSE_ED25519: u32 = 44;
pub const DERVIATION_PATH_PURPOSE_SECP256K1: u32 = 54;
pub const DERVIATION_PATH_PURPOSE_SECP256R1: u32 = 74;

/// Ed25519 follows SLIP-0010 using hardened path: m/44'/784'/0'/0'/{index}'
/// Secp256k1 follows BIP-32/44 using path where the first 3 levels are hardened: m/54'/784'/0'/0/{index}
/// Secp256r1 follows BIP-32/44 using path where the first 3 levels are hardened: m/74'/784'/0'/0/{index}
/// Note that the purpose node is used to distinguish signature schemes.
pub fn derive_key_pair_from_path(
    seed: &[u8],
    derivation_path: Option<DerivationPath>,
    key_scheme: &SignatureScheme,
) -> Result<(RoochAddress, RoochKeyPair), RoochError> {
    let path = validate_path(key_scheme, derivation_path)?;
    let indexes = path.into_iter().map(|i| i.into()).collect::<Vec<_>>();
    let derived = derive_ed25519_private_key(seed, &indexes);
    let sk = Ed25519PrivateKey::from_bytes(&derived)
        .map_err(|e| RoochError::SignatureKeyGenError(e.to_string()))?;
    let kp: Ed25519KeyPair = sk.into();
    let pk = kp.public();

    let mut hasher = DefaultHash::default();
    // TODO: fix flag
    hasher.update([0x00]);
    hasher.update(pk);
    let g_arr = hasher.finalize();

    Ok((RoochAddress(H256(g_arr.digest)), RoochKeyPair::Ed25519(kp)))
}

pub fn validate_path(
    _: &SignatureScheme,
    path: Option<DerivationPath>,
) -> Result<DerivationPath, RoochError> {
    // The derivation path must be hardened at all levels with purpose = 44, coin_type = 784
    match path {
        Some(p) => {
            // The derivation path must be hardened at all levels with purpose = 44, coin_type = 784
            if let &[purpose, coin_type, account, change, address] = p.as_ref() {
                if Some(purpose) == ChildNumber::new(DERVIATION_PATH_PURPOSE_ED25519, true).ok()
                    && Some(coin_type) == ChildNumber::new(DERIVATION_PATH_COIN_TYPE, true).ok()
                    && account.is_hardened()
                    && change.is_hardened()
                    && address.is_hardened()
                {
                    Ok(p)
                } else {
                    Err(RoochError::SignatureKeyGenError("Invalid path".to_string()))
                }
            } else {
                Err(RoochError::SignatureKeyGenError("Invalid path".to_string()))
            }
        }
        None => Ok(format!(
            "m/{DERVIATION_PATH_PURPOSE_ED25519}'/{DERIVATION_PATH_COIN_TYPE}'/0'/0'/0'"
        )
        .parse()
        .map_err(|_| RoochError::SignatureKeyGenError("Cannot parse path".to_string()))?),
    }
}

pub fn generate_new_key(
    key_scheme: SignatureScheme,
    derivation_path: Option<DerivationPath>,
    word_length: Option<String>,
) -> Result<(RoochAddress, RoochKeyPair, SignatureScheme, String), anyhow::Error> {
    let mnemonic = Mnemonic::new(parse_word_length(word_length)?, Language::English);
    let seed = Seed::new(&mnemonic, "");
    match derive_key_pair_from_path(seed.as_bytes(), derivation_path, &key_scheme) {
        Ok((address, kp)) => Ok((address, kp, key_scheme, mnemonic.phrase().to_string())),
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

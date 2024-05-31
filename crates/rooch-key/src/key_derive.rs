// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use argon2::password_hash::{PasswordHash, PasswordHasher, SaltString};
use argon2::Argon2;
use argon2::PasswordVerifier;
use bip32::{ChildNumber, DerivationPath, XPrv};
use bip39::{Language, Mnemonic, MnemonicType, Seed};
use fastcrypto::secp256k1::{Secp256k1KeyPair, Secp256k1PrivateKey};
use fastcrypto::traits::ToFromBytes;
use rooch_types::crypto::RoochKeyPair;
use rooch_types::error::RoochError;
use rooch_types::key_struct::{EncryptionData, GenerateNewKeyPair, GeneratedKeyPair};
use rooch_types::multichain_id::RoochMultiChainID;
use std::str::FromStr;
use std::string::String;

// Purpose constants
pub const DERIVATION_PATH_PURPOSE_ED25519: u32 = 44;
pub const DERIVATION_PATH_PURPOSE_SCHNORR: u32 = 44;
pub const DERIVATION_PATH_PURPOSE_ECDSA: u32 = 54;
pub const DERIVATION_PATH_PURPOSE_SECP256R1: u32 = 74;
pub const DERIVATION_PATH_PURPOSE_BIP86: u32 = 86;

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

pub fn derive_bitcoin_private_key_from_path(
    seed: &[u8],
    path: DerivationPath,
) -> Result<Secp256k1KeyPair, anyhow::Error> {
    validate_derivation_path(&path)?;
    let child_xprv = XPrv::derive_from_path(seed, &path)
        .map_err(|e| RoochError::SignatureKeyGenError(e.to_string()))?;
    let kp = Secp256k1KeyPair::from(
        Secp256k1PrivateKey::from_bytes(child_xprv.private_key().to_bytes().as_slice())
            .map_err(|e| RoochError::SignatureKeyGenError(e.to_string()))?,
    );
    Ok(kp)
}

fn validate_derivation_path(path: &DerivationPath) -> Result<(), anyhow::Error> {
    let (purpose, coin_type) = (
        DERIVATION_PATH_PURPOSE_BIP86,
        RoochMultiChainID::Bitcoin as u32,
    );

    if let &[p_purpose, p_coin_type, account, change, address] = path.as_ref() {
        if p_purpose == ChildNumber::new(purpose, true)?
            && p_coin_type == ChildNumber::new(coin_type, true)?
            && account.is_hardened()
            && !change.is_hardened()
            && !address.is_hardened()
        {
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "Invalid derivation path: {}, purpose:{}, coin_type: {}",
                path,
                p_purpose,
                p_coin_type
            ))
        }
    } else {
        Err(anyhow::anyhow!("Invalid derivation path: {}", path))
    }
}

/// Derivation path template
/// Which tells a wallet how to derive a specific key within a tree of keys
/// https://github.com/bitcoin/bips/blob/master/bip-0044.mediawiki
/// https://github.com/bitcoin/bips/blob/master/bip-0086.mediawiki
/// m / purpose' / coin_type' / account' / change / address_index
pub(crate) fn generate_derivation_path(account_index: u32) -> Result<DerivationPath, RoochError> {
    let (purpose, coin_type) = (
        DERIVATION_PATH_PURPOSE_BIP86,
        RoochMultiChainID::Bitcoin as u32,
    );

    DerivationPath::from_str(
        format!("m/{}'/{}'/0'/0/{}", purpose, coin_type, account_index).as_str(),
    )
    .map_err(|_| RoochError::SignatureKeyGenError("Cannot parse derivation path".to_owned()))
}

pub(crate) fn generate_new_key_pair(
    mnemonic_phrase: Option<String>,
    derivation_path: DerivationPath,
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

    let sk = derive_bitcoin_private_key_from_path(seed.as_bytes(), derivation_path)?;
    let rooch_kp = RoochKeyPair::Secp256k1(sk);
    let private_key_encryption = EncryptionData::encrypt_with_type(&rooch_kp, password.clone())?;
    let mnemonic_phrase_encryption =
        EncryptionData::encrypt(mnemonic.phrase().as_bytes(), password)?;

    let address = rooch_kp.public().rooch_address()?;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_key_pair() {
        let key_pair0 =
            generate_new_key_pair(None, generate_derivation_path(0).unwrap(), None, None).unwrap();
        let mnemonic_phrase = key_pair0.key_pair_data.mnemonic_phrase;
        let key_pair1 = generate_new_key_pair(
            Some(mnemonic_phrase.clone()),
            generate_derivation_path(1).unwrap(),
            None,
            None,
        )
        .unwrap();

        let recovery_key_pair0 = generate_new_key_pair(
            Some(mnemonic_phrase.clone()),
            generate_derivation_path(0).unwrap(),
            None,
            None,
        )
        .unwrap();
        let recovery_key_pair1 = generate_new_key_pair(
            Some(mnemonic_phrase),
            generate_derivation_path(1).unwrap(),
            None,
            None,
        )
        .unwrap();

        assert_eq!(key_pair0.address, recovery_key_pair0.address);
        assert_eq!(key_pair1.address, recovery_key_pair1.address);
    }
}

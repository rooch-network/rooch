// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::key_derive::{
    derive_address_from_private_key, derive_private_key_from_path, encrypt_key,
    generate_derivation_path, generate_new_key_pair, hash_password,
};
use crate::keystore::ImportedMnemonic;
use bip32::DerivationPath;
use bip39::{Language, Mnemonic, Seed};
use fastcrypto::encoding::{Base64, Encoding};
use rooch_types::key_struct::{MnemonicData, MnemonicResult};
use rooch_types::{
    address::RoochAddress,
    authentication_key::AuthenticationKey,
    crypto::{PublicKey, RoochKeyPair, Signature},
    error::RoochError,
    key_struct::{EncryptionData, GeneratedKeyPair},
    transaction::rooch::{RoochTransaction, RoochTransactionData},
};
use serde::Serialize;

pub trait AccountKeystore {
    fn add_address_encryption_data(
        &mut self,
        address: RoochAddress,
        encryption: EncryptionData,
    ) -> Result<(), anyhow::Error>;
    fn get_address_public_keys(
        &self,
        password: Option<String>,
    ) -> Result<Vec<(RoochAddress, PublicKey)>, anyhow::Error>;
    fn get_public_key(&self, password: Option<String>) -> Result<PublicKey, anyhow::Error>;
    fn get_key_pairs(
        &self,
        address: &RoochAddress,
        password: Option<String>,
    ) -> Result<Vec<RoochKeyPair>, anyhow::Error>;
    fn get_key_pair_with_password(
        &self,
        address: &RoochAddress,
        password: Option<String>,
    ) -> Result<RoochKeyPair, anyhow::Error>;

    fn get_password_hash(&self) -> String;

    fn get_if_password_is_empty(&self) -> bool;

    fn set_password_hash_with_indicator(
        &mut self,
        password_hash: String,
        is_password_empty: bool,
    ) -> Result<(), anyhow::Error>;

    fn update_address_encryption_data(
        &mut self,
        address: &RoochAddress,
        encryption: EncryptionData,
    ) -> Result<(), anyhow::Error>;
    fn nullify(&mut self, address: &RoochAddress) -> Result<(), anyhow::Error>;

    fn sign_hashed(
        &self,
        address: &RoochAddress,
        msg: &[u8],
        password: Option<String>,
    ) -> Result<Signature, anyhow::Error>;

    fn sign_transaction(
        &self,
        address: &RoochAddress,
        msg: RoochTransactionData,
        password: Option<String>,
    ) -> Result<RoochTransaction, anyhow::Error>;

    fn sign_secure<T>(
        &self,
        address: &RoochAddress,
        msg: &T,
        password: Option<String>,
    ) -> Result<Signature, anyhow::Error>
    where
        T: Serialize;

    fn addresses(&self) -> Vec<RoochAddress>;

    fn generate_and_add_new_key(
        &mut self,
        mnemonic_phrase: Option<String>,
        derivation_path: Option<DerivationPath>,
        word_length: Option<String>,
        password: Option<String>,
    ) -> Result<GeneratedKeyPair, anyhow::Error> {
        // load mnemonic phrase from keystore
        let one_mnemonic = self.get_mnemonics(password.clone())?.pop();
        let mnemonic_phrase = if mnemonic_phrase.is_some() {
            mnemonic_phrase
        } else {
            one_mnemonic
                .clone()
                .map(|mnemonic| mnemonic.mnemonic_phrase)
        };
        let derivation_path = if derivation_path.is_none() && one_mnemonic.is_some() {
            let account_index = one_mnemonic.clone().unwrap().mnemonic_data.addresses.len() as u32;
            Some(generate_derivation_path(account_index)?)
        } else {
            None
        };

        let result =
            generate_new_key_pair(mnemonic_phrase, derivation_path, word_length, password)?;
        let new_address = result.address;
        self.add_address_encryption_data(
            new_address,
            result.key_pair_data.private_key_encryption.clone(),
        )?;
        // reuse mnemonic if mnemonic already generate
        if let Some(mut update_mnemonic) = one_mnemonic {
            update_mnemonic.mnemonic_data.addresses.push(new_address);
            self.update_mnemonic_data(
                update_mnemonic.mnemonic_phrase_key,
                update_mnemonic.mnemonic_data,
            )?;
            // generate mnemonic for the first time
        } else {
            let mnemonic_key = hash_password(
                &Base64::decode(&result.key_pair_data.private_key_encryption.nonce)
                    .map_err(|e| RoochError::KeyConversionError(e.to_string()))?,
                Some(result.key_pair_data.mnemonic_phrase.clone()),
            )?;
            let mnemonic_data = MnemonicData {
                addresses: vec![new_address],
                mnemonic_phrase_encryption: result.key_pair_data.mnemonic_phrase_encryption.clone(),
            };
            self.add_mnemonic_data(mnemonic_key, mnemonic_data)?;
        }

        Ok(result)
    }

    fn import_from_mnemonic(
        &mut self,
        phrase: &str,
        derivation_path: Option<DerivationPath>,
        password: Option<String>,
    ) -> Result<ImportedMnemonic, anyhow::Error> {
        let mnemonic = Mnemonic::from_phrase(phrase, Language::English)?;
        let seed = Seed::new(&mnemonic, "");

        let sk = derive_private_key_from_path(seed.as_bytes(), derivation_path)?;

        let encryption = encrypt_key(&sk, password).expect("Encryption failed for private key");

        let address = derive_address_from_private_key(sk)?;

        let result = ImportedMnemonic {
            address,
            encryption: encryption.clone(),
        };

        self.add_address_encryption_data(result.address, encryption)?;

        Ok(result)
    }

    fn update_address_with_encryption_data(
        &mut self,
        _address: &RoochAddress,
        phrase: String,
        derivation_path: Option<DerivationPath>,
        password: Option<String>,
    ) -> Result<EncryptionData, anyhow::Error> {
        let mnemonic = Mnemonic::from_phrase(&phrase, Language::English)?;
        let seed = Seed::new(&mnemonic, "");

        let sk = derive_private_key_from_path(seed.as_bytes(), derivation_path)?;

        let encryption_data =
            encrypt_key(&sk, password).expect("Encryption failed for private key");

        let address = derive_address_from_private_key(sk)?;

        self.update_address_encryption_data(&address, encryption_data.clone())?;

        Ok(encryption_data)
    }

    fn nullify_address(&mut self, address: &RoochAddress) -> Result<(), anyhow::Error> {
        self.nullify(address)?;
        Ok(())
    }

    fn generate_session_key(
        &mut self,
        address: &RoochAddress,
        password: Option<String>,
    ) -> Result<AuthenticationKey, anyhow::Error>;

    fn sign_transaction_via_session_key(
        &self,
        address: &RoochAddress,
        msg: RoochTransactionData,
        authentication_key: &AuthenticationKey,
        password: Option<String>,
    ) -> Result<RoochTransaction, anyhow::Error>;

    fn get_mnemonics(&self, password: Option<String>)
        -> Result<Vec<MnemonicResult>, anyhow::Error>;

    fn add_mnemonic_data(
        &mut self,
        mnemonic_phrase: String,
        mnemonic_data: MnemonicData,
    ) -> Result<(), anyhow::Error>;

    fn update_mnemonic_data(
        &mut self,
        mnemonic_phrase: String,
        mnemonic_data: MnemonicData,
    ) -> Result<(), anyhow::Error>;
}

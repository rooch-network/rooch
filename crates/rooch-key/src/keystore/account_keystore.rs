// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::types::LocalAccount;
use crate::key_derive::{generate_derivation_path, generate_new_key_pair};
use rooch_types::framework::session_key::SessionKey;
use rooch_types::key_struct::{MnemonicData, MnemonicResult};
use rooch_types::{
    address::RoochAddress,
    authentication_key::AuthenticationKey,
    crypto::{RoochKeyPair, Signature},
    key_struct::{EncryptionData, GeneratedKeyPair},
    transaction::rooch::{RoochTransaction, RoochTransactionData},
};
use serde::Serialize;

pub trait AccountKeystore {
    fn init_keystore(
        &mut self,
        mnemonic_phrase: Option<String>,
        word_length: Option<String>,
        password: Option<String>,
    ) -> Result<GeneratedKeyPair, anyhow::Error> {
        let derivation_path = generate_derivation_path(0)?;
        let result =
            generate_new_key_pair(mnemonic_phrase, derivation_path, word_length, password)?;
        let new_address = result.address;
        self.add_address_encryption_data_to_keys(
            new_address,
            result.key_pair_data.private_key_encryption.clone(),
        )?;
        let mnemonic_data = MnemonicData {
            addresses: vec![new_address],
            mnemonic_phrase_encryption: result.key_pair_data.mnemonic_phrase_encryption.clone(),
        };
        self.init_mnemonic_data(mnemonic_data)?;
        Ok(result)
    }

    fn init_mnemonic_data(&mut self, mnemonic_data: MnemonicData) -> Result<(), anyhow::Error>;

    fn add_addresses_to_mnemonic_data(
        &mut self,
        address: RoochAddress,
    ) -> Result<(), anyhow::Error>;

    fn get_mnemonic(&self, password: Option<String>) -> Result<MnemonicResult, anyhow::Error>;

    fn generate_and_add_new_key(
        &mut self,
        password: Option<String>,
    ) -> Result<GeneratedKeyPair, anyhow::Error> {
        // load mnemonic phrase from keystore
        let mnemonic = self.get_mnemonic(password.clone())?;
        let account_index = mnemonic.mnemonic_data.addresses.len() as u32;
        let derivation_path = generate_derivation_path(account_index)?;
        let result = generate_new_key_pair(
            Some(mnemonic.mnemonic_phrase),
            derivation_path,
            None,
            password,
        )?;
        let new_address = result.address;
        self.add_address_encryption_data_to_keys(
            new_address,
            result.key_pair_data.private_key_encryption.clone(),
        )?;
        self.add_addresses_to_mnemonic_data(new_address)?;
        Ok(result)
    }

    fn export_mnemonic_phrase(
        &mut self,
        password: Option<String>,
    ) -> Result<String, anyhow::Error> {
        // load mnemonic phrase from keystore
        let mnemonic = self.get_mnemonic(password.clone())?;
        let mnemonic_phrase = mnemonic.mnemonic_phrase;
        Ok(mnemonic_phrase)
    }

    fn import_external_account(
        &mut self,
        address: RoochAddress,
        kp: RoochKeyPair,
        password: Option<String>,
    ) -> Result<(), anyhow::Error> {
        let private_key_encryption = EncryptionData::encrypt_with_type(&kp, password)?;
        self.add_address_encryption_data_to_keys(address, private_key_encryption)?;
        Ok(())
    }

    /// Get all local accounts
    //TODO refactor the keystore, save the public key out of the encryption data, so that we don't need to require password to get the public key
    fn get_accounts(&self, password: Option<String>) -> Result<Vec<LocalAccount>, anyhow::Error>;

    /// Get local account by address
    fn get_account(
        &self,
        address: &RoochAddress,
        password: Option<String>,
    ) -> Result<Option<LocalAccount>, anyhow::Error> {
        let accounts = self.get_accounts(password)?;
        let account = accounts.iter().find(|account| account.address == *address);
        Ok(account.cloned())
    }

    fn contains_address(&self, address: &RoochAddress) -> bool;

    fn add_address_encryption_data_to_keys(
        &mut self,
        address: RoochAddress,
        encryption: EncryptionData,
    ) -> Result<(), anyhow::Error>;

    fn get_key_pair(
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

    fn nullify_address(&mut self, address: &RoochAddress) -> Result<(), anyhow::Error> {
        self.nullify(address)?;
        Ok(())
    }

    fn generate_session_key(
        &mut self,
        address: &RoochAddress,
        password: Option<String>,
    ) -> Result<AuthenticationKey, anyhow::Error>;

    /// Binding on-chain SessionKey to LocalSessionKey
    fn binding_session_key(
        &mut self,
        address: RoochAddress,
        session_key: SessionKey,
    ) -> Result<(), anyhow::Error>;

    fn get_session_key(
        &self,
        address: &RoochAddress,
        authentication_key: &AuthenticationKey,
        password: Option<String>,
    ) -> Result<Option<RoochKeyPair>, anyhow::Error>;

    fn sign_transaction_via_session_key(
        &self,
        address: &RoochAddress,
        msg: RoochTransactionData,
        authentication_key: &AuthenticationKey,
        password: Option<String>,
    ) -> Result<RoochTransaction, anyhow::Error>;
}

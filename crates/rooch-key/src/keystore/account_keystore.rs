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
        self.add_address_encryption_data(
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
        self.add_address_encryption_data(
            new_address,
            result.key_pair_data.private_key_encryption.clone(),
        )?;
        Ok(result)
    }

    fn get_accounts(&self, password: Option<String>) -> Result<Vec<LocalAccount>, anyhow::Error>;

    fn add_address_encryption_data(
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

    fn sign_transaction_via_session_key(
        &self,
        address: &RoochAddress,
        msg: RoochTransactionData,
        authentication_key: &AuthenticationKey,
        password: Option<String>,
    ) -> Result<RoochTransaction, anyhow::Error>;
}

// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::types::LocalAccount;
use crate::keystore::account_keystore::AccountKeystore;
use crate::keystore::base_keystore::BaseKeyStore;
use rooch_types::key_struct::{MnemonicData, MnemonicResult};
use rooch_types::{
    address::RoochAddress,
    authentication_key::AuthenticationKey,
    crypto::{RoochKeyPair, Signature},
    key_struct::EncryptionData,
    transaction::rooch::{RoochTransaction, RoochTransactionData},
};
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct InMemKeystore {
    keystore: BaseKeyStore,
}

impl AccountKeystore for InMemKeystore {
    fn get_accounts(&self, password: Option<String>) -> Result<Vec<LocalAccount>, anyhow::Error> {
        self.keystore.get_accounts(password)
    }

    fn add_address_encryption_data(
        &mut self,
        address: RoochAddress,
        encryption: EncryptionData,
    ) -> Result<(), anyhow::Error> {
        self.keystore
            .add_address_encryption_data(address, encryption)
    }

    fn get_key_pair(
        &self,
        address: &RoochAddress,
        password: Option<String>,
    ) -> Result<RoochKeyPair, anyhow::Error> {
        self.keystore.get_key_pair(address, password)
    }

    fn nullify(&mut self, address: &RoochAddress) -> Result<(), anyhow::Error> {
        self.keystore.nullify(address)
    }

    fn sign_hashed(
        &self,
        address: &RoochAddress,
        msg: &[u8],
        password: Option<String>,
    ) -> Result<Signature, anyhow::Error> {
        self.keystore.sign_hashed(address, msg, password)
    }

    fn sign_transaction(
        &self,
        address: &RoochAddress,
        msg: RoochTransactionData,
        password: Option<String>,
    ) -> Result<RoochTransaction, anyhow::Error> {
        self.keystore.sign_transaction(address, msg, password)
    }

    fn sign_secure<T>(
        &self,
        address: &RoochAddress,
        msg: &T,
        password: Option<String>,
    ) -> Result<Signature, anyhow::Error>
    where
        T: Serialize,
    {
        self.keystore.sign_secure(address, msg, password)
    }

    fn addresses(&self) -> Vec<RoochAddress> {
        // Create an empty Vec to store the addresses.
        let mut addresses = Vec::new();

        // Iterate over the `keys` and `session_keys` BTreeMaps.
        for key in self.keystore.keys.keys() {
            addresses.push(*key);
        }

        for key in self.keystore.session_keys.keys() {
            addresses.push(*key);
        }

        addresses
    }

    fn generate_session_key(
        &mut self,
        address: &RoochAddress,
        password: Option<String>,
    ) -> Result<AuthenticationKey, anyhow::Error> {
        self.keystore.generate_session_key(address, password)
    }

    fn binding_session_key(
        &mut self,
        address: RoochAddress,
        session_key: rooch_types::framework::session_key::SessionKey,
    ) -> Result<(), anyhow::Error> {
        self.keystore.binding_session_key(address, session_key)
    }

    fn sign_transaction_via_session_key(
        &self,
        address: &RoochAddress,
        msg: RoochTransactionData,
        authentication_key: &AuthenticationKey,
        password: Option<String>,
    ) -> Result<RoochTransaction, anyhow::Error> {
        self.keystore
            .sign_transaction_via_session_key(address, msg, authentication_key, password)
    }

    fn set_password_hash_with_indicator(
        &mut self,
        password_hash: String,
        is_password_empty: bool,
    ) -> Result<(), anyhow::Error> {
        self.keystore.password_hash = Some(password_hash);
        self.keystore.is_password_empty = is_password_empty;
        Ok(())
    }

    fn get_password_hash(&self) -> String {
        self.keystore.password_hash.clone().unwrap_or_default()
    }

    fn get_if_password_is_empty(&self) -> bool {
        self.keystore.is_password_empty
    }

    fn get_mnemonic(&self, password: Option<String>) -> Result<MnemonicResult, anyhow::Error> {
        self.keystore.get_mnemonic(password)
    }

    fn init_mnemonic_data(&mut self, mnemonic_data: MnemonicData) -> Result<(), anyhow::Error> {
        self.keystore.init_mnemonic_data(mnemonic_data)
    }
}

impl InMemKeystore {
    pub fn new_insecure_for_tests(initial_key_number: usize) -> Self {
        let mut keystore = BaseKeyStore::new();
        keystore.init_keystore(None, None, None).unwrap();
        for _ in 0..initial_key_number {
            keystore.generate_and_add_new_key(None).unwrap();
        }

        Self { keystore }
    }
}

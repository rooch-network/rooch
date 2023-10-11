// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::key_derive::get_key_pair_from_red;
use crate::keystore::account_keystore::AccountKeystore;
use crate::keystore::base_keystore::BaseKeyStore;
use rooch_types::key_struct::{MnemonicData, MnemonicResult};
use rooch_types::{
    address::RoochAddress,
    authentication_key::AuthenticationKey,
    crypto::{PublicKey, RoochKeyPair, Signature},
    error::RoochError,
    key_struct::EncryptionData,
    transaction::rooch::{RoochTransaction, RoochTransactionData},
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct InMemKeystore {
    keystore: BaseKeyStore,
}

impl AccountKeystore for InMemKeystore {
    fn add_address_encryption_data(
        &mut self,
        address: RoochAddress,
        encryption: EncryptionData,
    ) -> Result<(), anyhow::Error> {
        self.keystore
            .add_address_encryption_data(address, encryption)
    }

    fn get_address_public_keys(
        &self,
        password: Option<String>,
    ) -> Result<Vec<(RoochAddress, PublicKey)>, RoochError> {
        self.keystore.get_address_public_keys(password)
    }

    fn get_public_key(&self, password: Option<String>) -> Result<PublicKey, anyhow::Error> {
        self.keystore.get_public_key(password)
    }

    fn get_key_pairs(
        &self,
        address: &RoochAddress,
        password: Option<String>,
    ) -> Result<Vec<RoochKeyPair>, anyhow::Error> {
        self.keystore.get_key_pairs(address, password)
    }

    fn get_key_pair_by_password(
        &self,
        address: &RoochAddress,
        password: Option<String>,
    ) -> Result<RoochKeyPair, RoochError> {
        self.keystore.get_key_pair_by_password(address, password)
    }

    fn update_address_encryption_data(
        &mut self,
        address: &RoochAddress,
        encryption: EncryptionData,
    ) -> Result<(), anyhow::Error> {
        self.keystore
            .update_address_encryption_data(address, encryption)
    }

    fn nullify(&mut self, address: &RoochAddress) -> Result<(), anyhow::Error> {
        self.keystore.nullify(address)
    }

    fn sign_hashed(
        &self,
        address: &RoochAddress,
        msg: &[u8],
        password: Option<String>,
    ) -> Result<Signature, RoochError> {
        self.keystore.sign_hashed(address, msg, password)
    }

    fn sign_transaction(
        &self,
        address: &RoochAddress,
        msg: RoochTransactionData,
        password: Option<String>,
    ) -> Result<RoochTransaction, RoochError> {
        self.keystore.sign_transaction(address, msg, password)
    }

    fn sign_secure<T>(
        &self,
        address: &RoochAddress,
        msg: &T,
        password: Option<String>,
    ) -> Result<Signature, RoochError>
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

    fn sign_transaction_via_session_key(
        &self,
        address: &RoochAddress,
        msg: RoochTransactionData,
        authentication_key: &AuthenticationKey,
        password: Option<String>,
    ) -> Result<RoochTransaction, signature::Error> {
        self.keystore
            .sign_transaction_via_session_key(address, msg, authentication_key, password)
    }

    fn get_mnemonics(
        &self,
        password: Option<String>,
    ) -> Result<Vec<MnemonicResult>, anyhow::Error> {
        self.keystore.get_mnemonics(password)
    }

    fn add_mnemonic_data(
        &mut self,
        mnemonic_phrase: String,
        mnemonic_data: MnemonicData,
    ) -> Result<(), anyhow::Error> {
        self.keystore
            .add_mnemonic_data(mnemonic_phrase, mnemonic_data)
    }

    fn update_mnemonic_data(
        &mut self,
        mnemonic_phrase: String,
        mnemonic_data: MnemonicData,
    ) -> Result<(), anyhow::Error> {
        self.keystore
            .update_mnemonic_data(mnemonic_phrase, mnemonic_data)
    }
}

impl InMemKeystore {
    pub fn new_insecure_for_tests(initial_key_number: usize) -> Self {
        let keys = (0..initial_key_number)
            .map(|_| get_key_pair_from_red())
            .map(|(addr, data)| (addr, data))
            .collect::<BTreeMap<RoochAddress, EncryptionData>>();

        Self {
            keystore: BaseKeyStore::new(keys),
        }
    }
}

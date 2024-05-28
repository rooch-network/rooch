// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::types::LocalAccount;
use crate::key_derive::retrieve_key_pair;
use crate::keystore::account_keystore::AccountKeystore;
use crate::keystore::base_keystore::BaseKeyStore;
use anyhow::anyhow;
use rooch_types::key_struct::{MnemonicData, MnemonicResult};
use rooch_types::{
    address::RoochAddress,
    authentication_key::AuthenticationKey,
    crypto::{RoochKeyPair, Signature},
    key_struct::EncryptionData,
    transaction::rooch::{RoochTransaction, RoochTransactionData},
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct FileBasedKeystore {
    pub(crate) keystore: BaseKeyStore,
    pub(crate) path: Option<PathBuf>,
}

impl AccountKeystore for FileBasedKeystore {
    fn init_mnemonic_data(&mut self, mnemonic_data: MnemonicData) -> Result<(), anyhow::Error> {
        self.keystore.init_mnemonic_data(mnemonic_data)?;
        self.save()?;
        Ok(())
    }

    fn get_accounts(&self, password: Option<String>) -> Result<Vec<LocalAccount>, anyhow::Error> {
        self.keystore.get_accounts(password)
    }

    fn add_address_encryption_data(
        &mut self,
        address: RoochAddress,
        encryption: EncryptionData,
    ) -> Result<(), anyhow::Error> {
        self.keystore
            .add_address_encryption_data(address, encryption)?;
        self.save()?;
        Ok(())
    }

    fn get_key_pair(
        &self,
        address: &RoochAddress,
        password: Option<String>,
    ) -> Result<RoochKeyPair, anyhow::Error> {
        self.keystore.get_key_pair(address, password)
    }

    fn nullify(&mut self, address: &RoochAddress) -> Result<(), anyhow::Error> {
        self.keystore.nullify(address)?;
        self.save()?;
        Ok(())
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
        let auth_key = self.keystore.generate_session_key(address, password)?;
        self.save()?;
        Ok(auth_key)
    }

    fn binding_session_key(
        &mut self,
        address: RoochAddress,
        session_key: rooch_types::framework::session_key::SessionKey,
    ) -> Result<(), anyhow::Error> {
        self.keystore.binding_session_key(address, session_key)?;
        self.save()?;
        Ok(())
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
        self.save()?;
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
}

impl FileBasedKeystore {
    pub fn new(path: &PathBuf) -> Result<Self, anyhow::Error> {
        let keystore = if path.exists() {
            let reader = BufReader::new(File::open(path).map_err(|e| {
                anyhow!(
                    "Can't open FileBasedKeystore from Rooch path {:?}: {}",
                    path,
                    e
                )
            })?);
            serde_json::from_reader(reader).map_err(|e| {
                anyhow!(
                    "Can't deserialize FileBasedKeystore from Rooch path {:?}: {}",
                    path,
                    e
                )
            })?
        } else {
            BaseKeyStore::new()
        };

        Ok(Self {
            keystore,
            path: Some(path.to_path_buf()),
        })
    }

    pub fn load(path: &PathBuf) -> Result<Self, anyhow::Error> {
        if path.exists() {
            let reader = BufReader::new(File::open(path).map_err(|e| {
                anyhow!(
                    "Can't open FileBasedKeystore from Rooch path {:?}: {}",
                    path,
                    e
                )
            })?);
            let keystore = serde_json::from_reader(reader).map_err(|e| {
                anyhow!(
                    "Can't deserialize FileBasedKeystore from Rooch path {:?}: {}",
                    path,
                    e
                )
            })?;
            Ok(Self {
                keystore,
                path: Some(path.to_path_buf()),
            })
        } else {
            Err(anyhow!("Key store path {:?} does not exist", path))
        }
    }

    pub fn set_path(&mut self, path: &Path) {
        self.path = Some(path.to_path_buf());
    }

    pub fn save(&self) -> Result<(), anyhow::Error> {
        if let Some(path) = &self.path {
            let store = serde_json::to_string_pretty(&self.keystore)?;
            fs::write(path, store)?;
        }
        Ok(())
    }

    pub fn key_pairs(
        &self,
        _address: &RoochAddress,
        password: Option<String>,
    ) -> Result<Vec<RoochKeyPair>, anyhow::Error> {
        // Collect references to RoochKeyPair objects from all inner maps.
        let key_pairs: Vec<RoochKeyPair> = self
            .keystore
            .keys
            .values() // Get inner maps
            .flat_map(|encryption| {
                // Transform EncryptionData into RoochKeyPair using your conversion function.
                Some(retrieve_key_pair(encryption, password.clone()))
            })
            .collect::<Result<_, _>>()?;

        Ok(key_pairs)
    }
}

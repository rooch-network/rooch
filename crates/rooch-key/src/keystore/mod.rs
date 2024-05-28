// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::keystore::account_keystore::AccountKeystore;
use crate::keystore::file_keystore::FileBasedKeystore;
use enum_dispatch::enum_dispatch;
use memory_keystore::InMemKeystore;
use rooch_types::key_struct::{GeneratedKeyPair, MnemonicData, MnemonicResult};
use rooch_types::{
    address::RoochAddress,
    authentication_key::AuthenticationKey,
    crypto::{PublicKey, RoochKeyPair, Signature},
    key_struct::EncryptionData,
    transaction::rooch::{RoochTransaction, RoochTransactionData},
};
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::fmt::Write;

pub mod account_keystore;
pub mod base_keystore;
pub mod file_keystore;
pub mod memory_keystore;
pub mod types;

pub struct ImportedMnemonic {
    pub address: RoochAddress,
    pub encryption: EncryptionData,
}

#[derive(Serialize, Deserialize, Debug)]
#[enum_dispatch(AccountKeystore)]
pub enum Keystore {
    File(FileBasedKeystore),
    InMem(InMemKeystore),
}

impl AccountKeystore for Keystore {
    fn init_keystore(
        &mut self,
        mnemonic_phrase: Option<String>,
        word_length: Option<String>,
        password: Option<String>,
    ) -> Result<GeneratedKeyPair, anyhow::Error> {
        match self {
            Keystore::File(file_keystore) => {
                file_keystore.init_keystore(mnemonic_phrase, word_length, password)
            }
            Keystore::InMem(inmem_keystore) => {
                inmem_keystore.init_keystore(mnemonic_phrase, word_length, password)
            }
        }
    }

    fn init_mnemonic_data(&mut self, mnemonic_data: MnemonicData) -> Result<(), anyhow::Error> {
        match self {
            Keystore::File(file_keystore) => file_keystore.init_mnemonic_data(mnemonic_data),
            Keystore::InMem(inmem_keystore) => inmem_keystore.init_mnemonic_data(mnemonic_data),
        }
    }

    fn get_accounts(
        &self,
        password: Option<String>,
    ) -> Result<Vec<types::LocalAccount>, anyhow::Error> {
        match self {
            Keystore::File(file_keystore) => file_keystore.get_accounts(password),
            Keystore::InMem(inmem_keystore) => inmem_keystore.get_accounts(password),
        }
    }

    fn sign_transaction_via_session_key(
        &self,
        address: &RoochAddress,
        msg: RoochTransactionData,
        authentication_key: &AuthenticationKey,
        password: Option<String>,
    ) -> Result<RoochTransaction, anyhow::Error> {
        // Implement this method by delegating the call to the appropriate variant (File or InMem)
        match self {
            Keystore::File(file_keystore) => file_keystore.sign_transaction_via_session_key(
                address,
                msg,
                authentication_key,
                password,
            ),
            Keystore::InMem(inmem_keystore) => inmem_keystore.sign_transaction_via_session_key(
                address,
                msg,
                authentication_key,
                password,
            ),
        }
    }

    fn add_address_encryption_data(
        &mut self,
        address: RoochAddress,
        encryption: EncryptionData,
    ) -> Result<(), anyhow::Error> {
        // Implement this method to add a key pair to the appropriate variant (File or InMem)
        match self {
            Keystore::File(file_keystore) => {
                file_keystore.add_address_encryption_data(address, encryption)
            }
            Keystore::InMem(inmem_keystore) => {
                inmem_keystore.add_address_encryption_data(address, encryption)
            }
        }
    }

    fn get_key_pair_with_password(
        &self,
        address: &RoochAddress,
        password: Option<String>,
    ) -> Result<RoochKeyPair, anyhow::Error> {
        // Implement this method to get the key pair by coin ID from the appropriate variant (File or InMem)
        match self {
            Keystore::File(file_keystore) => {
                file_keystore.get_key_pair_with_password(address, password)
            }
            Keystore::InMem(inmem_keystore) => {
                inmem_keystore.get_key_pair_with_password(address, password)
            }
        }
    }

    fn nullify(&mut self, address: &RoochAddress) -> Result<(), anyhow::Error> {
        // Implement this method to nullify the key pair by coin ID for the appropriate variant (File or InMem)
        match self {
            Keystore::File(file_keystore) => file_keystore.nullify(address),
            Keystore::InMem(inmem_keystore) => inmem_keystore.nullify(address),
        }
    }

    fn sign_hashed(
        &self,
        address: &RoochAddress,
        msg: &[u8],
        password: Option<String>,
    ) -> Result<Signature, anyhow::Error> {
        // Implement this method to sign a hashed message for the appropriate variant (File or InMem)
        match self {
            Keystore::File(file_keystore) => file_keystore.sign_hashed(address, msg, password),
            Keystore::InMem(inmem_keystore) => inmem_keystore.sign_hashed(address, msg, password),
        }
    }

    fn sign_transaction(
        &self,
        address: &RoochAddress,
        msg: RoochTransactionData,
        password: Option<String>,
    ) -> Result<RoochTransaction, anyhow::Error> {
        // Implement this method to sign a transaction for the appropriate variant (File or InMem)
        match self {
            Keystore::File(file_keystore) => file_keystore.sign_transaction(address, msg, password),
            Keystore::InMem(inmem_keystore) => {
                inmem_keystore.sign_transaction(address, msg, password)
            }
        }
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
        // Implement this method to sign a secure message for the appropriate variant (File or InMem)
        match self {
            Keystore::File(file_keystore) => file_keystore.sign_secure(address, msg, password),
            Keystore::InMem(inmem_keystore) => inmem_keystore.sign_secure(address, msg, password),
        }
    }

    fn generate_session_key(
        &mut self,
        address: &RoochAddress,
        password: Option<String>,
    ) -> Result<AuthenticationKey, anyhow::Error> {
        // Implement this method to generate a session key for the appropriate variant (File or InMem)
        match self {
            Keystore::File(file_keystore) => file_keystore.generate_session_key(address, password),
            Keystore::InMem(inmem_keystore) => {
                inmem_keystore.generate_session_key(address, password)
            }
        }
    }

    fn binding_session_key(
        &mut self,
        address: RoochAddress,
        session_key: rooch_types::framework::session_key::SessionKey,
    ) -> Result<(), anyhow::Error> {
        match self {
            Keystore::File(file_keystore) => {
                file_keystore.binding_session_key(address, session_key)
            }
            Keystore::InMem(inmem_keystore) => {
                inmem_keystore.binding_session_key(address, session_key)
            }
        }
    }

    fn addresses(&self) -> Vec<RoochAddress> {
        match self {
            Keystore::File(file_keystore) => file_keystore.addresses(),
            Keystore::InMem(inmem_keystore) => inmem_keystore.addresses(),
        }
    }

    fn set_password_hash_with_indicator(
        &mut self,
        password_hash: String,
        is_password_empty: bool,
    ) -> Result<(), anyhow::Error> {
        match self {
            Keystore::File(file_keystore) => {
                file_keystore.set_password_hash_with_indicator(password_hash, is_password_empty)
            }
            Keystore::InMem(inmem_keystore) => {
                inmem_keystore.set_password_hash_with_indicator(password_hash, is_password_empty)
            }
        }
    }

    fn get_password_hash(&self) -> String {
        match self {
            Keystore::File(file_keystore) => file_keystore.get_password_hash(),
            Keystore::InMem(inmem_keystore) => inmem_keystore.get_password_hash(),
        }
    }

    fn get_if_password_is_empty(&self) -> bool {
        match self {
            Keystore::File(file_keystore) => file_keystore.get_if_password_is_empty(),
            Keystore::InMem(inmem_keystore) => inmem_keystore.get_if_password_is_empty(),
        }
    }

    fn get_mnemonic(&self, password: Option<String>) -> Result<MnemonicResult, anyhow::Error> {
        match self {
            Keystore::File(file_keystore) => file_keystore.get_mnemonic(password),
            Keystore::InMem(inmem_keystore) => inmem_keystore.get_mnemonic(password),
        }
    }
}

impl Display for Keystore {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut writer = String::new();
        match self {
            Keystore::File(file) => {
                writeln!(writer, "Keystore Type : Rooch File")?;
                write!(writer, "Keystore Path : {:?}", file.path)?;
            }
            Keystore::InMem(_) => {
                writeln!(writer, "Keystore Type : Rooch InMem")?;
            }
        }
        write!(f, "{}", writer)
    }
}

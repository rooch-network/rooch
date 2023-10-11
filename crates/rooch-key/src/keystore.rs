// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::key_derive::{
    derive_address_from_private_key, derive_private_key_from_path, encrypt_private_key,
    generate_new_key_pair, get_key_pair_from_red, retrieve_key_pair,
};
use anyhow::{anyhow, Error, Ok};
use bip32::DerivationPath;
use bip39::{Language, Mnemonic, Seed};
use enum_dispatch::enum_dispatch;
use rooch_types::{
    address::RoochAddress,
    authentication_key::AuthenticationKey,
    crypto::{PublicKey, RoochKeyPair, Signature},
    error::RoochError,
    key_struct::{EncryptionData, GeneratedKeyPair},
    transaction::{
        authenticator,
        rooch::{RoochTransaction, RoochTransactionData},
    },
};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use std::collections::BTreeMap;
use std::fmt::Display;
use std::fmt::Write;
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

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

pub trait AccountKeystore {
    fn add_encryption_data(
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
    fn update_encryption_data(
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
        derivation_path: Option<DerivationPath>,
        word_length: Option<String>,
        password: Option<String>,
    ) -> Result<GeneratedKeyPair, anyhow::Error> {
        let result = generate_new_key_pair(derivation_path, word_length, password)?;

        self.add_encryption_data(result.address, result.result.encryption.clone())?;

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

        let encryption =
            encrypt_private_key(&sk, password).expect("Encryption failed for private key");

        let address = derive_address_from_private_key(sk)?;

        let result = ImportedMnemonic {
            address,
            encryption: encryption.clone(),
        };

        self.add_encryption_data(result.address, encryption)?;

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
            encrypt_private_key(&sk, password).expect("Encryption failed for private key");

        let address = derive_address_from_private_key(sk)?;

        self.update_encryption_data(&address, encryption_data.clone())?;

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

    fn set_password_hash_with_indicator(
        &mut self,
        password_hash: String,
        is_password_empty: bool,
    ) -> Result<(), anyhow::Error>;
}

impl AccountKeystore for Keystore {
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

    fn add_encryption_data(
        &mut self,
        address: RoochAddress,
        encryption: EncryptionData,
    ) -> Result<(), anyhow::Error> {
        // Implement this method to add a key pair to the appropriate variant (File or InMem)
        match self {
            Keystore::File(file_keystore) => file_keystore.add_encryption_data(address, encryption),
            Keystore::InMem(inmem_keystore) => {
                inmem_keystore.add_encryption_data(address, encryption)
            }
        }
    }

    fn get_address_public_keys(
        &self,
        password: Option<String>,
    ) -> Result<Vec<(RoochAddress, PublicKey)>, anyhow::Error> {
        // Implement this method to collect public keys from the appropriate variant (File or InMem)
        match self {
            Keystore::File(file_keystore) => file_keystore.get_address_public_keys(password),
            Keystore::InMem(inmem_keystore) => inmem_keystore.get_address_public_keys(password),
        }
    }

    fn get_public_key(&self, password: Option<String>) -> Result<PublicKey, anyhow::Error> {
        // Implement this method to get the public key by coin ID from the appropriate variant (File or InMem)
        match self {
            Keystore::File(file_keystore) => file_keystore.get_public_key(password),
            Keystore::InMem(inmem_keystore) => inmem_keystore.get_public_key(password),
        }
    }

    fn get_key_pairs(
        &self,
        address: &RoochAddress,
        password: Option<String>,
    ) -> Result<Vec<RoochKeyPair>, anyhow::Error> {
        // Implement this method to get key pairs for the given address from the appropriate variant (File or InMem)
        match self {
            Keystore::File(file_keystore) => file_keystore.get_key_pairs(address, password),
            Keystore::InMem(inmem_keystore) => inmem_keystore.get_key_pairs(address, password),
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

    fn update_encryption_data(
        &mut self,
        address: &RoochAddress,
        encryption: EncryptionData,
    ) -> Result<(), anyhow::Error> {
        // Implement this method to update the key pair by coin ID for the appropriate variant (File or InMem)
        match self {
            Keystore::File(file_keystore) => {
                file_keystore.update_encryption_data(address, encryption)
            }
            Keystore::InMem(inmem_keystore) => {
                inmem_keystore.update_encryption_data(address, encryption)
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

#[derive(Default, Debug, Serialize, Deserialize)]
#[serde_as]
pub(crate) struct BaseKeyStore {
    keys: BTreeMap<RoochAddress, EncryptionData>,
    /// RoochAddress -> BTreeMap<AuthenticationKey, RoochKeyPair>
    #[serde_as(as = "BTreeMap<DisplayFromStr, BTreeMap<DisplayFromStr, _>>")]
    session_keys: BTreeMap<RoochAddress, BTreeMap<AuthenticationKey, EncryptionData>>,
    password_hash: Option<String>,
    is_password_empty: bool,
}

impl BaseKeyStore {
    pub fn new(keys: BTreeMap<RoochAddress, EncryptionData>) -> Self {
        Self {
            keys,
            session_keys: BTreeMap::new(),
            password_hash: None,
            is_password_empty: true,
        }
    }
}

impl AccountKeystore for BaseKeyStore {
    fn get_key_pair_with_password(
        &self,
        address: &RoochAddress,
        password: Option<String>,
    ) -> Result<RoochKeyPair, anyhow::Error> {
        if let Some(encryption) = self.keys.get(address) {
            let keypair: RoochKeyPair = retrieve_key_pair(encryption, password)?;
            Ok(keypair)
        } else {
            Err(Error::new(RoochError::SignMessageError(format!(
                "Cannot find key for address: [{:?}]",
                address
            ))))
        }
    }

    fn sign_hashed(
        &self,
        address: &RoochAddress,
        msg: &[u8],
        password: Option<String>,
    ) -> Result<Signature, anyhow::Error> {
        Ok(Signature::new_hashed(
            msg,
            &self.get_key_pair_with_password(address, password)?,
        ))
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
        Ok(Signature::new_secure(
            msg,
            &self.get_key_pair_with_password(address, password)?,
        ))
    }

    fn sign_transaction(
        &self,
        address: &RoochAddress,
        msg: RoochTransactionData,
        password: Option<String>,
    ) -> Result<RoochTransaction, anyhow::Error> {
        let kp = self
            .get_key_pair_with_password(address, password)
            .ok()
            .ok_or_else(|| {
                RoochError::SignMessageError(format!("Cannot find key for address: [{address}]"))
            })?;

        let signature = Signature::new_hashed(msg.hash().as_bytes(), &kp);

        let auth = authenticator::Authenticator::rooch(signature);

        Ok(RoochTransaction::new(msg, auth))
    }

    fn add_encryption_data(
        &mut self,
        address: RoochAddress,
        encryption: EncryptionData,
    ) -> Result<(), anyhow::Error> {
        self.keys.entry(address).or_insert(encryption);
        Ok(())
    }

    fn get_public_key(&self, password: Option<String>) -> Result<PublicKey, anyhow::Error> {
        self.keys
            .values()
            .find_map(|encryption| {
                let keypair: RoochKeyPair = retrieve_key_pair(encryption, password.clone()).ok()?;
                Some(keypair.public())
            })
            .ok_or_else(|| anyhow::Error::msg("No valid public key found"))
    }

    fn get_address_public_keys(
        &self,
        password: Option<String>,
    ) -> Result<Vec<(RoochAddress, PublicKey)>, anyhow::Error> {
        let mut result = Vec::new();
        for (address, encryption) in &self.keys {
            let keypair: RoochKeyPair = retrieve_key_pair(encryption, password.clone())?;
            let public_key = keypair.public();
            result.push((*address, public_key));
        }
        Ok(result)
    }

    fn get_key_pairs(
        &self,
        address: &RoochAddress,
        password: Option<String>,
    ) -> Result<Vec<RoochKeyPair>, anyhow::Error> {
        match self.keys.get(address) {
            Some(encryption) => {
                let kp = retrieve_key_pair(encryption, password)?;
                Ok(vec![kp])
            }
            None => Err(anyhow!("Cannot find key for address: [{address}]")),
        }
    }

    fn update_encryption_data(
        &mut self,
        address: &RoochAddress,
        encryption: EncryptionData,
    ) -> Result<(), anyhow::Error> {
        self.keys.entry(*address).or_insert(encryption);
        Ok(())
    }

    fn nullify(&mut self, address: &RoochAddress) -> Result<(), anyhow::Error> {
        self.keys.remove(address);
        Ok(())
    }

    fn generate_session_key(
        &mut self,
        address: &RoochAddress,
        password: Option<String>,
    ) -> Result<AuthenticationKey, anyhow::Error> {
        //TODO define derivation_path for session key
        let result = generate_new_key_pair(None, None, password.clone())?;
        let kp: RoochKeyPair = retrieve_key_pair(&result.result.encryption, password)?;
        let authentication_key = kp.public().authentication_key();
        let inner_map = self
            .session_keys
            .entry(*address)
            .or_insert_with(BTreeMap::new);
        inner_map.insert(authentication_key.clone(), result.result.encryption);
        Ok(authentication_key)
    }

    fn sign_transaction_via_session_key(
        &self,
        address: &RoochAddress,
        msg: RoochTransactionData,
        authentication_key: &AuthenticationKey,
        password: Option<String>,
    ) -> Result<RoochTransaction, anyhow::Error> {
        let encryption = self
            .session_keys
            .get(address)
            .ok_or_else(|| {
                signature::Error::from_source(format!(
                    "Cannot find SessionKey for address: [{address}]"
                ))
            })?
            .get(authentication_key)
            .ok_or_else(|| {
                signature::Error::from_source(format!(
                    "Cannot find SessionKey for authentication_key: [{authentication_key}]"
                ))
            })?;

        let kp: RoochKeyPair =
            retrieve_key_pair(encryption, password).map_err(signature::Error::from_source)?;

        let signature = Signature::new_hashed(msg.hash().as_bytes(), &kp);

        let auth = authenticator::Authenticator::rooch(signature);
        Ok(RoochTransaction::new(msg, auth))
    }

    fn addresses(&self) -> Vec<RoochAddress> {
        // Create an empty Vec to store the addresses.
        let mut addresses = Vec::new();

        // Iterate over the `keys` and `session_keys` BTreeMaps.
        for key in self.keys.keys() {
            addresses.push(*key);
        }

        for key in self.session_keys.keys() {
            addresses.push(*key);
        }

        addresses
    }

    fn set_password_hash_with_indicator(
        &mut self,
        password_hash: String,
        is_password_empty: bool,
    ) -> Result<(), anyhow::Error> {
        self.password_hash = Some(password_hash);
        self.is_password_empty = is_password_empty;
        Ok(())
    }

    fn get_password_hash(&self) -> String {
        self.password_hash.clone().unwrap_or_default()
    }

    fn get_if_password_is_empty(&self) -> bool {
        self.is_password_empty
    }
}

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct FileBasedKeystore {
    keystore: BaseKeyStore,
    path: Option<PathBuf>,
}

impl AccountKeystore for FileBasedKeystore {
    fn get_key_pair_with_password(
        &self,
        address: &RoochAddress,
        password: Option<String>,
    ) -> Result<RoochKeyPair, anyhow::Error> {
        self.keystore.get_key_pair_with_password(address, password)
    }

    fn sign_hashed(
        &self,
        address: &RoochAddress,
        msg: &[u8],
        password: Option<String>,
    ) -> Result<Signature, anyhow::Error> {
        self.keystore.sign_hashed(address, msg, password)
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

    fn sign_transaction(
        &self,
        address: &RoochAddress,
        msg: RoochTransactionData,
        password: Option<String>,
    ) -> Result<RoochTransaction, anyhow::Error> {
        self.keystore.sign_transaction(address, msg, password)
    }

    fn add_encryption_data(
        &mut self,
        address: RoochAddress,
        encryption: EncryptionData,
    ) -> Result<(), anyhow::Error> {
        self.keystore.add_encryption_data(address, encryption)?;
        //TODO should check test env at here?
        if std::env::var_os("TEST_ENV").is_none() {
            self.save()?;
        }
        Ok(())
    }

    fn get_public_key(&self, password: Option<String>) -> Result<PublicKey, anyhow::Error> {
        self.keystore.get_public_key(password)
    }

    fn get_address_public_keys(
        &self,
        password: Option<String>,
    ) -> Result<Vec<(RoochAddress, PublicKey)>, anyhow::Error> {
        self.keystore.get_address_public_keys(password)
    }

    fn get_key_pairs(
        &self,
        address: &RoochAddress,
        password: Option<String>,
    ) -> Result<Vec<RoochKeyPair>, anyhow::Error> {
        self.keystore.get_key_pairs(address, password)
    }

    fn update_encryption_data(
        &mut self,
        address: &RoochAddress,
        encryption: EncryptionData,
    ) -> Result<(), anyhow::Error> {
        self.keystore.update_encryption_data(address, encryption)?;
        //TODO should check test env at here?
        if std::env::var_os("TEST_ENV").is_none() {
            self.save()?;
        }
        Ok(())
    }

    fn nullify(&mut self, address: &RoochAddress) -> Result<(), anyhow::Error> {
        self.keystore.nullify(address)?;
        //TODO should check test env at here?
        if std::env::var_os("TEST_ENV").is_none() {
            self.save()?;
        }
        Ok(())
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

    fn set_password_hash_with_indicator(
        &mut self,
        password_hash: String,
        is_password_empty: bool,
    ) -> Result<(), anyhow::Error> {
        self.keystore.password_hash = Some(password_hash);
        self.keystore.is_password_empty = is_password_empty;
        if std::env::var_os("TEST_ENV").is_none() {
            self.save()?;
        }
        Ok(())
    }

    fn get_password_hash(&self) -> String {
        self.keystore.password_hash.clone().unwrap_or_default()
    }

    fn get_if_password_is_empty(&self) -> bool {
        self.keystore.is_password_empty
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
            BaseKeyStore::new(BTreeMap::new())
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
            //TODO crypto the keystore
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

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct InMemKeystore {
    keystore: BaseKeyStore,
}

impl AccountKeystore for InMemKeystore {
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

    fn sign_transaction(
        &self,
        address: &RoochAddress,
        msg: RoochTransactionData,
        password: Option<String>,
    ) -> Result<RoochTransaction, anyhow::Error> {
        self.keystore.sign_transaction(address, msg, password)
    }

    fn add_encryption_data(
        &mut self,
        address: RoochAddress,
        encryption: EncryptionData,
    ) -> Result<(), anyhow::Error> {
        self.keystore.add_encryption_data(address, encryption)
    }

    fn get_public_key(&self, password: Option<String>) -> Result<PublicKey, anyhow::Error> {
        self.keystore.get_public_key(password)
    }

    fn get_address_public_keys(
        &self,
        password: Option<String>,
    ) -> Result<Vec<(RoochAddress, PublicKey)>, anyhow::Error> {
        self.keystore.get_address_public_keys(password)
    }

    fn get_key_pairs(
        &self,
        address: &RoochAddress,
        password: Option<String>,
    ) -> Result<Vec<RoochKeyPair>, anyhow::Error> {
        self.keystore.get_key_pairs(address, password)
    }

    fn update_encryption_data(
        &mut self,
        address: &RoochAddress,
        encryption: EncryptionData,
    ) -> Result<(), anyhow::Error> {
        self.keystore.update_encryption_data(address, encryption)
    }

    fn nullify(&mut self, address: &RoochAddress) -> Result<(), anyhow::Error> {
        self.keystore.nullify(address)
    }

    fn get_key_pair_with_password(
        &self,
        address: &RoochAddress,
        password: Option<String>,
    ) -> Result<RoochKeyPair, anyhow::Error> {
        self.keystore.get_key_pair_with_password(address, password)
    }

    fn sign_hashed(
        &self,
        address: &RoochAddress,
        msg: &[u8],
        password: Option<String>,
    ) -> Result<Signature, anyhow::Error> {
        self.keystore.sign_hashed(address, msg, password)
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
    ) -> Result<RoochTransaction, anyhow::Error> {
        self.keystore
            .sign_transaction_via_session_key(address, msg, authentication_key, password)
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

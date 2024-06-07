// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::types::{LocalAccount, LocalSessionKey};
use crate::keystore::account_keystore::AccountKeystore;
use anyhow::ensure;
use rooch_types::framework::session_key::SessionKey;
use rooch_types::key_struct::{MnemonicData, MnemonicResult};
use rooch_types::{
    address::RoochAddress,
    authentication_key::AuthenticationKey,
    crypto::{RoochKeyPair, Signature},
    error::RoochError,
    key_struct::EncryptionData,
    transaction::{
        authenticator,
        rooch::{RoochTransaction, RoochTransactionData},
    },
};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use std::collections::BTreeMap;

#[derive(Default, Debug, Serialize, Deserialize)]
#[serde_as]
pub(crate) struct BaseKeyStore {
    #[serde(default)]
    pub(crate) keys: BTreeMap<RoochAddress, EncryptionData>,
    #[serde(default)]
    pub(crate) mnemonic: Option<MnemonicData>,
    #[serde(default)]
    #[serde_as(as = "BTreeMap<DisplayFromStr, BTreeMap<DisplayFromStr, _>>")]
    pub(crate) session_keys: BTreeMap<RoochAddress, BTreeMap<AuthenticationKey, LocalSessionKey>>,
    #[serde(default)]
    pub(crate) password_hash: Option<String>,
    #[serde(default)]
    pub(crate) is_password_empty: bool,
}

impl BaseKeyStore {
    pub fn new() -> Self {
        Self {
            keys: BTreeMap::new(),
            mnemonic: None,
            session_keys: BTreeMap::new(),
            password_hash: None,
            is_password_empty: true,
        }
    }
}

impl AccountKeystore for BaseKeyStore {
    fn init_mnemonic_data(&mut self, mnemonic_data: MnemonicData) -> Result<(), anyhow::Error> {
        ensure!(self.mnemonic.is_none(), "Mnemonic data already exists");
        self.mnemonic = Some(mnemonic_data);
        Ok(())
    }

    fn get_accounts(&self, password: Option<String>) -> Result<Vec<LocalAccount>, anyhow::Error> {
        let mut accounts = BTreeMap::new();
        for (address, encryption) in &self.keys {
            let keypair: RoochKeyPair = encryption.decrypt_with_type(password.clone())?;
            let public_key = keypair.public();
            let bitcoin_address = public_key.bitcoin_address()?;
            let has_session_key = self.session_keys.get(address).is_some();
            let local_account = LocalAccount {
                address: *address,
                bitcoin_address,
                public_key,
                has_session_key,
            };
            accounts.insert(*address, local_account);
        }
        Ok(accounts.into_values().collect())
    }

    fn get_key_pair(
        &self,
        address: &RoochAddress,
        password: Option<String>,
    ) -> Result<RoochKeyPair, anyhow::Error> {
        if let Some(encryption) = self.keys.get(address) {
            let keypair: RoochKeyPair = encryption.decrypt_with_type::<RoochKeyPair>(password)?;
            Ok(keypair)
        } else {
            Err(anyhow::Error::new(RoochError::SignMessageError(format!(
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
        Ok(Signature::sign(msg, &self.get_key_pair(address, password)?))
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
        Ok(Signature::sign_secure(
            msg,
            &self.get_key_pair(address, password)?,
        ))
    }

    fn sign_transaction(
        &self,
        address: &RoochAddress,
        msg: RoochTransactionData,
        password: Option<String>,
    ) -> Result<RoochTransaction, anyhow::Error> {
        let kp = self.get_key_pair(address, password).ok().ok_or_else(|| {
            RoochError::SignMessageError(format!("Cannot find key for address: [{address}]"))
        })?;
        let auth = authenticator::Authenticator::bitcoin(&kp, &msg);
        Ok(RoochTransaction::new(msg, auth))
    }

    fn add_address_encryption_data(
        &mut self,
        address: RoochAddress,
        encryption: EncryptionData,
    ) -> Result<(), anyhow::Error> {
        self.keys.entry(address).or_insert(encryption);
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
        let kp: RoochKeyPair = RoochKeyPair::generate_ed25519();
        let authentication_key = kp.public().authentication_key();
        let inner_map = self.session_keys.entry(*address).or_default();
        let private_key_encryption = EncryptionData::encrypt_with_type(&kp, password)?;
        let local_session_key = LocalSessionKey {
            session_key: None,
            private_key: private_key_encryption,
        };
        inner_map.insert(authentication_key.clone(), local_session_key);
        Ok(authentication_key)
    }

    fn binding_session_key(
        &mut self,
        address: RoochAddress,
        session_key: SessionKey,
    ) -> Result<(), anyhow::Error> {
        let inner_map: &mut BTreeMap<AuthenticationKey, LocalSessionKey> =
            self.session_keys.entry(address).or_default();
        let authentication_key = session_key.authentication_key();
        let local_session_key = inner_map.get_mut(&authentication_key).ok_or_else(||{
            anyhow::Error::new(RoochError::KeyConversionError(format!("Cannot find session key for address:[{address}] and authentication_key:[{authentication_key}]", address = address, authentication_key = authentication_key)))
        })?;
        local_session_key.session_key = Some(session_key);
        Ok(())
    }

    fn sign_transaction_via_session_key(
        &self,
        address: &RoochAddress,
        msg: RoochTransactionData,
        authentication_key: &AuthenticationKey,
        password: Option<String>,
    ) -> Result<RoochTransaction, anyhow::Error> {
        let local_session_key = self
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

        let kp: RoochKeyPair = local_session_key
            .private_key
            .decrypt_with_type(password)
            .map_err(signature::Error::from_source)?;

        let auth = authenticator::Authenticator::rooch(&kp, &msg);
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

    fn get_mnemonic(&self, password: Option<String>) -> Result<MnemonicResult, anyhow::Error> {
        match &self.mnemonic {
            Some(mnemonic_data) => {
                let mnemonic_phrase = mnemonic_data.mnemonic_phrase_encryption.decrypt(password)?;

                let mnemonic_phrase = String::from_utf8(mnemonic_phrase)
                    .map_err(|e| anyhow::anyhow!("Parse mnemonic phrase error:{}", e))?;
                Ok(MnemonicResult {
                    mnemonic_phrase,
                    mnemonic_data: mnemonic_data.clone(),
                })
            }
            None => Err(anyhow::Error::new(RoochError::KeyConversionError(
                "Cannot find mnemonic data, please init the keystore first".to_string(),
            ))),
        }
    }
}

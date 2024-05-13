// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeMap;

use super::types::{AddressMapping, LocalAccount, LocalSessionKey};
use crate::key_derive::{decrypt_key, generate_new_key_pair, retrieve_key_pair};
use crate::keystore::account_keystore::AccountKeystore;
use anyhow::anyhow;
use fastcrypto::encoding::{Base64, Encoding};
use rooch_types::framework::session_key::SessionKey;
use rooch_types::key_struct::{MnemonicData, MnemonicResult};
use rooch_types::{
    address::RoochAddress,
    authentication_key::AuthenticationKey,
    crypto::{PublicKey, RoochKeyPair, Signature},
    error::RoochError,
    key_struct::EncryptionData,
    transaction::{
        authenticator,
        rooch::{RoochTransaction, RoochTransactionData},
    },
};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

#[derive(Default, Debug, Serialize, Deserialize)]
#[serde_as]
pub(crate) struct BaseKeyStore {
    #[serde(default)]
    pub(crate) keys: BTreeMap<RoochAddress, EncryptionData>,
    #[serde(default)]
    pub(crate) mnemonics: BTreeMap<String, MnemonicData>,
    #[serde(default)]
    #[serde_as(as = "BTreeMap<DisplayFromStr, BTreeMap<DisplayFromStr, _>>")]
    pub(crate) session_keys: BTreeMap<RoochAddress, BTreeMap<AuthenticationKey, LocalSessionKey>>,
    #[serde(default)]
    pub(crate) password_hash: Option<String>,
    #[serde(default)]
    pub(crate) is_password_empty: bool,
    #[serde(default)]
    pub(crate) address_mapping: AddressMapping,
}

impl BaseKeyStore {
    pub fn new(keys: BTreeMap<RoochAddress, EncryptionData>) -> Self {
        Self {
            keys,
            mnemonics: BTreeMap::new(),
            session_keys: BTreeMap::new(),
            password_hash: None,
            is_password_empty: true,
            address_mapping: AddressMapping::default(),
        }
    }
}

impl AccountKeystore for BaseKeyStore {
    fn get_accounts(&self, password: Option<String>) -> Result<Vec<LocalAccount>, anyhow::Error> {
        let mut accounts = BTreeMap::new();
        for (address, encryption) in &self.keys {
            let keypair: RoochKeyPair = retrieve_key_pair(encryption, password.clone())?;
            let public_key = keypair.public();
            let multichain_address = self
                .address_mapping
                .rooch_to_multichain
                .get(address)
                .cloned();
            let has_session_key = self.session_keys.get(address).is_some();
            let local_account = LocalAccount {
                address: *address,
                multichain_address,
                public_key: Some(public_key),
                has_session_key,
            };
            accounts.insert(*address, local_account);
        }
        for address in self.session_keys.keys() {
            if accounts.contains_key(address) {
                continue;
            }
            let multichain_address = self
                .address_mapping
                .rooch_to_multichain
                .get(address)
                .cloned();
            let has_session_key = true;
            let local_account = LocalAccount {
                address: *address,
                multichain_address,
                public_key: None,
                has_session_key,
            };
            accounts.insert(*address, local_account);
        }
        Ok(accounts.into_values().collect())
    }

    fn get_key_pair_with_password(
        &self,
        address: &RoochAddress,
        password: Option<String>,
    ) -> Result<RoochKeyPair, anyhow::Error> {
        if let Some(encryption) = self.keys.get(address) {
            let keypair: RoochKeyPair = retrieve_key_pair(encryption, password)?;
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

        let signature = Signature::new_hashed(msg.tx_hash().as_bytes(), &kp);

        let auth = authenticator::Authenticator::rooch(signature);

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

    fn update_address_encryption_data(
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
        session_key: Option<SessionKey>,
    ) -> Result<AuthenticationKey, anyhow::Error> {
        //TODO define derivation_path for session key
        let result = generate_new_key_pair(None, None, None, password.clone())?;
        let kp: RoochKeyPair =
            retrieve_key_pair(&result.key_pair_data.private_key_encryption, password)?;
        let authentication_key = kp.public().authentication_key();
        let inner_map = self.session_keys.entry(*address).or_default();
        let local_session_key = LocalSessionKey {
            session_key,
            private_key: result.key_pair_data.private_key_encryption,
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
        //TODO should we check the scope of session key here?
        // let session_key = local_session_key.session_key.as_ref().ok_or_else(||{
        //     signature::Error::from_source(
        //         format!("SessionKey for authentication_key:[{authentication_key}] do not binding to on-chain SessionKey")
        //     )
        // })?;
        // ensure!(session_key.is_scope_match_with_action(&msg.action), signature::Error::from_source(
        //     format!("SessionKey for authentication_key:[{authentication_key}] scope do not match with transaction")
        // ));
        let kp: RoochKeyPair = retrieve_key_pair(&local_session_key.private_key, password)
            .map_err(signature::Error::from_source)?;

        let signature = Signature::new_hashed(msg.tx_hash().as_bytes(), &kp);

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

    fn get_mnemonics(
        &self,
        password: Option<String>,
    ) -> Result<Vec<MnemonicResult>, anyhow::Error> {
        match self.mnemonics.first_key_value() {
            Some((k, v)) => {
                let nonce = Base64::decode(&v.mnemonic_phrase_encryption.nonce).map_err(|e| {
                    anyhow::Error::new(RoochError::KeyConversionError(e.to_string()))
                })?;
                let ciphertext =
                    Base64::decode(&v.mnemonic_phrase_encryption.ciphertext).map_err(|e| {
                        anyhow::Error::new(RoochError::KeyConversionError(e.to_string()))
                    })?;
                let tag = Base64::decode(&v.mnemonic_phrase_encryption.tag).map_err(|e| {
                    anyhow::Error::new(RoochError::KeyConversionError(e.to_string()))
                })?;

                let mnemonic_phrase = decrypt_key(
                    nonce.as_slice(),
                    ciphertext.as_slice(),
                    tag.as_slice(),
                    password,
                )?;

                let mnemonic_phrase = String::from_utf8(mnemonic_phrase)
                    .map_err(|e| anyhow::anyhow!("Parse mnemonic phrase error:{}", e))?;
                let mnemonic_generated_address = MnemonicResult {
                    mnemonic_phrase,
                    mnemonic_phrase_key: k.clone(),
                    mnemonic_data: v.clone(),
                };
                Ok(vec![mnemonic_generated_address])
            }
            None => Ok(vec![]),
        }
    }

    fn add_mnemonic_data(
        &mut self,
        mnemonic_phrase: String,
        mnemonic_data: MnemonicData,
    ) -> Result<(), anyhow::Error> {
        self.mnemonics
            .entry(mnemonic_phrase)
            .or_insert(mnemonic_data);
        Ok(())
    }

    fn update_mnemonic_data(
        &mut self,
        mnemonic_phrase: String,
        mnemonic_data: MnemonicData,
    ) -> Result<(), anyhow::Error> {
        // insert or update
        self.mnemonics.insert(mnemonic_phrase, mnemonic_data);
        Ok(())
    }
}

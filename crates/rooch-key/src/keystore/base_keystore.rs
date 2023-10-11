// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::key_derive::{decrypt_key, generate_new_key_pair, retrieve_key_pair};
use crate::keystore::account_keystore::AccountKeystore;
use anyhow::anyhow;
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
use std::collections::BTreeMap;

#[derive(Default, Debug, Serialize, Deserialize)]
#[serde_as]
pub(crate) struct BaseKeyStore {
    pub(crate) keys: BTreeMap<RoochAddress, EncryptionData>,
    pub(crate) mnemonics: BTreeMap<String, MnemonicData>,
    #[serde_as(as = "BTreeMap<DisplayFromStr, BTreeMap<DisplayFromStr, _>>")]
    pub(crate) session_keys: BTreeMap<RoochAddress, BTreeMap<AuthenticationKey, EncryptionData>>,
}

impl BaseKeyStore {
    pub fn new(keys: BTreeMap<RoochAddress, EncryptionData>) -> Self {
        Self {
            keys,
            mnemonics: BTreeMap::new(),
            session_keys: BTreeMap::new(),
        }
    }
}

impl AccountKeystore for BaseKeyStore {
    fn get_key_pair_by_password(
        &self,
        address: &RoochAddress,
        password: Option<String>,
    ) -> Result<RoochKeyPair, RoochError> {
        if let Some(encryption) = self.keys.get(address) {
            let keypair: RoochKeyPair = retrieve_key_pair(encryption, password)?;
            Ok(keypair)
        } else {
            Err(RoochError::SignMessageError(format!(
                "Cannot find key for address: [{:?}]",
                address
            )))
        }
    }

    fn sign_hashed(
        &self,
        address: &RoochAddress,
        msg: &[u8],
        password: Option<String>,
    ) -> Result<Signature, RoochError> {
        Ok(Signature::new_hashed(
            msg,
            &self.get_key_pair_by_password(address, password)?,
        ))
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
        Ok(Signature::new_secure(
            msg,
            &self.get_key_pair_by_password(address, password)?,
        ))
    }

    fn sign_transaction(
        &self,
        address: &RoochAddress,
        msg: RoochTransactionData,
        password: Option<String>,
    ) -> Result<RoochTransaction, RoochError> {
        let kp = self
            .get_key_pair_by_password(address, password)
            .ok()
            .ok_or_else(|| {
                RoochError::SignMessageError(format!("Cannot find key for address: [{address}]"))
            })?;

        let signature = Signature::new_hashed(msg.hash().as_bytes(), &kp);

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
    ) -> Result<Vec<(RoochAddress, PublicKey)>, RoochError> {
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
    ) -> Result<AuthenticationKey, anyhow::Error> {
        //TODO define derivation_path for session key
        let result = generate_new_key_pair(None, None, None, password.clone())?;
        let kp: RoochKeyPair =
            retrieve_key_pair(&result.key_pair_data.private_key_encryption, password)?;
        let authentication_key = kp.public().authentication_key();
        let inner_map = self
            .session_keys
            .entry(*address)
            .or_insert_with(BTreeMap::new);
        inner_map.insert(
            authentication_key.clone(),
            result.key_pair_data.private_key_encryption,
        );
        Ok(authentication_key)
    }

    fn sign_transaction_via_session_key(
        &self,
        address: &RoochAddress,
        msg: RoochTransactionData,
        authentication_key: &AuthenticationKey,
        password: Option<String>,
    ) -> Result<RoochTransaction, signature::Error> {
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

    fn get_mnemonics(
        &self,
        password: Option<String>,
    ) -> Result<Vec<MnemonicResult>, anyhow::Error> {
        match self.mnemonics.first_key_value() {
            Some((k, v)) => {
                let mnemonic_phrase = decrypt_key(
                    &v.mnemonic_phrase_encryption.nonce,
                    &v.mnemonic_phrase_encryption.ciphertext,
                    &v.mnemonic_phrase_encryption.tag,
                    password,
                )?;

                let mnemonic_generated_address = MnemonicResult {
                    mnemonic_phrase: mnemonic_phrase.into(),
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

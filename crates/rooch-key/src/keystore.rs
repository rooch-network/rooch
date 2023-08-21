// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::key_derive::{derive_key_pair_from_path, generate_new_key};
use anyhow::anyhow;
use bip32::DerivationPath;
use bip39::{Language, Mnemonic, Seed};
use enum_dispatch::enum_dispatch;
use rand::{rngs::StdRng, SeedableRng};
use rooch_types::{
    address::RoochAddress,
    authentication_key::AuthenticationKey,
    crypto::{get_key_pair_from_rng, BuiltinScheme, PublicKey, RoochKeyPair, Signature},
    transaction::{
        authenticator,
        rooch::{RoochTransaction, RoochTransactionData},
    },
};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_with::serde_as;
use std::collections::BTreeMap;
use std::fmt::Write;
use std::fmt::{Display, Formatter};
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize)]
#[enum_dispatch(AccountKeystore)]
pub enum Keystore {
    File(FileBasedKeystore),
    InMem(InMemKeystore),
}

#[enum_dispatch]
pub trait AccountKeystore: Send + Sync {
    fn add_key_pair_by_scheme(
        &mut self,
        keypair: RoochKeyPair,
        scheme: BuiltinScheme,
    ) -> Result<(), anyhow::Error>;
    fn get_address_public_keys(&self) -> Vec<(RoochAddress, PublicKey)>;
    fn get_public_key_by_scheme(&self, scheme: BuiltinScheme) -> Result<PublicKey, anyhow::Error>;
    fn get_key_pairs(&self, address: &RoochAddress) -> Result<Vec<&RoochKeyPair>, anyhow::Error>;
    fn get_key_pair_by_scheme(
        &self,
        address: &RoochAddress,
        scheme: BuiltinScheme,
    ) -> Result<&RoochKeyPair, signature::Error>;
    fn update_key_pair_by_scheme(
        &mut self,
        address: &RoochAddress,
        keypair: RoochKeyPair,
        scheme: BuiltinScheme,
    ) -> Result<(), anyhow::Error>;
    fn nullify_key_pair_by_scheme(
        &mut self,
        address: &RoochAddress,
        scheme: BuiltinScheme,
    ) -> Result<(), anyhow::Error>;

    fn sign_hashed(
        &self,
        address: &RoochAddress,
        msg: &[u8],
        scheme: BuiltinScheme,
    ) -> Result<Signature, signature::Error>;

    fn sign_transaction(
        &self,
        address: &RoochAddress,
        msg: RoochTransactionData,
        scheme: BuiltinScheme,
    ) -> Result<RoochTransaction, signature::Error>;

    fn sign_secure<T>(
        &self,
        address: &RoochAddress,
        msg: &T,
        scheme: BuiltinScheme,
    ) -> Result<Signature, signature::Error>
    where
        T: Serialize;

    fn addresses(&self) -> Vec<RoochAddress> {
        self.get_address_public_keys()
            .iter()
            .map(|(address, _public_key)| *address)
            .collect()
    }

    fn generate_and_add_new_key(
        &mut self,
        crypto_scheme: BuiltinScheme,
        derivation_path: Option<DerivationPath>,
        word_length: Option<String>,
    ) -> Result<(RoochAddress, String, BuiltinScheme), anyhow::Error> {
        let (address, kp, scheme, phrase) =
            generate_new_key(crypto_scheme, derivation_path, word_length)?;
        self.add_key_pair_by_scheme(kp, scheme)?;
        Ok((address, phrase, scheme))
    }

    fn import_from_mnemonic(
        &mut self,
        phrase: &str,
        crypto_scheme: BuiltinScheme,
        derivation_path: Option<DerivationPath>,
    ) -> Result<RoochAddress, anyhow::Error> {
        let mnemonic = Mnemonic::from_phrase(phrase, Language::English)
            .map_err(|e| anyhow::anyhow!("Invalid mnemonic phrase: {:?}", e))?;
        let seed = Seed::new(&mnemonic, "");
        match derive_key_pair_from_path(seed.as_bytes(), derivation_path, &crypto_scheme) {
            Ok((address, kp)) => {
                self.add_key_pair_by_scheme(kp, crypto_scheme)?;
                Ok(address)
            }
            Err(e) => Err(anyhow!("error getting keypair {:?}", e)),
        }
    }

    fn update_address_with_key_pair_from_scheme(
        &mut self,
        address: &RoochAddress,
        phrase: String,
        crypto_scheme: BuiltinScheme,
        derivation_path: Option<DerivationPath>,
    ) -> Result<PublicKey, anyhow::Error> {
        let mnemonic = Mnemonic::from_phrase(phrase.as_str(), Language::English)
            .map_err(|e| anyhow::anyhow!("Invalid mnemonic phrase: {:?}", e))?;
        let seed = Seed::new(&mnemonic, "");
        match derive_key_pair_from_path(seed.as_bytes(), derivation_path, &crypto_scheme) {
            Ok((_, kp)) => {
                let public_key = kp.public();
                self.update_key_pair_by_scheme(address, kp, crypto_scheme)?;
                Ok(public_key)
            }
            Err(e) => Err(anyhow!("error getting keypair {:?}", e)),
        }
    }

    fn nullify_address_with_key_pair_from_scheme(
        &mut self,
        address: &RoochAddress,
        crypto_scheme: BuiltinScheme,
    ) -> Result<(), anyhow::Error> {
        self.nullify_key_pair_by_scheme(address, crypto_scheme)?;
        Ok(())
    }

    fn generate_session_key(
        &mut self,
        address: &RoochAddress,
    ) -> Result<AuthenticationKey, anyhow::Error>;

    fn sign_transaction_via_session_key(
        &self,
        address: &RoochAddress,
        msg: RoochTransactionData,
        authentication_key: &AuthenticationKey,
    ) -> Result<RoochTransaction, signature::Error>;
}

impl Display for Keystore {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut writer = String::new();
        match self {
            Keystore::File(file) => {
                writeln!(writer, "Keystore Type : File")?;
                write!(writer, "Keystore Path : {:?}", file.path)?;
                write!(f, "{}", writer)
            }
            Keystore::InMem(_) => {
                writeln!(writer, "Keystore Type : InMem")?;
                write!(f, "{}", writer)
            }
        }
    }
}

#[derive(Default, Serialize, Deserialize)]
#[serde_as]
pub(crate) struct BaseKeyStore {
    keys: BTreeMap<RoochAddress, BTreeMap<BuiltinScheme, RoochKeyPair>>,
    /// RoochAddress -> BTreeMap<AuthenticationKey, RoochKeyPair>
    #[serde_as(as = "BTreeMap<DisplayFromStr, BTreeMap<DisplayFromStr, _>>")]
    session_keys: BTreeMap<RoochAddress, BTreeMap<AuthenticationKey, RoochKeyPair>>,
}

impl BaseKeyStore {
    pub fn new(keys: BTreeMap<RoochAddress, BTreeMap<BuiltinScheme, RoochKeyPair>>) -> Self {
        Self {
            keys,
            session_keys: BTreeMap::new(),
        }
    }
}

impl AccountKeystore for BaseKeyStore {
    fn get_key_pair_by_scheme(
        &self,
        address: &RoochAddress,
        scheme: BuiltinScheme,
    ) -> Result<&RoochKeyPair, signature::Error> {
        if let Some(inner_map) = self.keys.get(address) {
            if let Some(keypair) = inner_map.get(&scheme) {
                Ok(keypair)
            } else {
                Err(signature::Error::from_source(format!(
                    "Scheme not found for address: [{:?}]",
                    scheme
                )))
            }
        } else {
            Err(signature::Error::from_source(format!(
                "Cannot find key for address: [{:?}]",
                address
            )))
        }
    }

    fn sign_hashed(
        &self,
        address: &RoochAddress,
        msg: &[u8],
        scheme: BuiltinScheme,
    ) -> Result<Signature, signature::Error> {
        Ok(Signature::new_hashed(
            msg,
            self.get_key_pair_by_scheme(address, scheme)?,
        ))
    }

    fn sign_secure<T>(
        &self,
        address: &RoochAddress,
        msg: &T,
        scheme: BuiltinScheme,
    ) -> Result<Signature, signature::Error>
    where
        T: Serialize,
    {
        Ok(Signature::new_secure(
            msg,
            self.get_key_pair_by_scheme(address, scheme)?,
        ))
    }

    fn sign_transaction(
        &self,
        address: &RoochAddress,
        msg: RoochTransactionData,
        scheme: BuiltinScheme,
    ) -> Result<RoochTransaction, signature::Error> {
        let kp = self
            .get_key_pair_by_scheme(address, scheme)
            .ok()
            .ok_or_else(|| {
                signature::Error::from_source(format!("Cannot find key for address: [{address}]"))
            })?;

        let signature = Signature::new_hashed(msg.hash().as_bytes(), kp);

        let auth = match kp.public().scheme() {
            BuiltinScheme::Ed25519 => authenticator::Authenticator::ed25519(signature),
            BuiltinScheme::Ecdsa => authenticator::Authenticator::ecdsa(signature),
            BuiltinScheme::EcdsaRecoverable => {
                authenticator::Authenticator::ecdsa_recoverable(signature)
            }
            BuiltinScheme::MultiEd25519 => todo!(),
            BuiltinScheme::Schnorr => authenticator::Authenticator::schnorr(signature),
        };

        Ok(RoochTransaction::new(msg, auth))
    }

    fn add_key_pair_by_scheme(
        &mut self,
        keypair: RoochKeyPair,
        scheme: BuiltinScheme,
    ) -> Result<(), anyhow::Error> {
        let address: RoochAddress = (&keypair.public()).into();
        self.keys
            .entry(address)
            .or_insert_with(BTreeMap::new)
            .insert(scheme, keypair);
        Ok(())
    }

    fn get_public_key_by_scheme(&self, scheme: BuiltinScheme) -> Result<PublicKey, anyhow::Error> {
        for inner_map in self.keys.values() {
            if let Some(keypair) = inner_map.get(&scheme) {
                return Ok(keypair.public());
            }
        }
        Err(anyhow!("Cannot find key for scheme: [{:?}]", scheme))
    }

    fn get_address_public_keys(&self) -> Vec<(RoochAddress, PublicKey)> {
        let mut result = Vec::new();
        for (address, inner_map) in &self.keys {
            for keypair in inner_map.values() {
                let public_key = keypair.public();
                result.push((*address, public_key));
            }
        }
        result
    }

    fn get_key_pairs(&self, address: &RoochAddress) -> Result<Vec<&RoochKeyPair>, anyhow::Error> {
        match self.keys.get(address) {
            Some(key_map) => {
                // Collect references to RoochKeyPair objects from the inner map into a Vec.
                let key_pairs: Vec<&RoochKeyPair> = key_map.values().collect();
                Ok(key_pairs)
            }
            None => Err(anyhow!("Cannot find key for address: [{address}]")),
        }
    }

    fn update_key_pair_by_scheme(
        &mut self,
        address: &RoochAddress,
        keypair: RoochKeyPair,
        scheme: BuiltinScheme,
    ) -> Result<(), anyhow::Error> {
        // First, get the inner map associated with the address
        let inner_map = self.keys.entry(*address).or_insert_with(BTreeMap::new);

        // Insert or update the keypair for the specified scheme in the inner map
        inner_map.insert(scheme, keypair);
        Ok(())
    }

    fn nullify_key_pair_by_scheme(
        &mut self,
        address: &RoochAddress,
        scheme: BuiltinScheme,
    ) -> Result<(), anyhow::Error> {
        // First, get the inner map associated with the address
        let inner_map = self.keys.entry(*address).or_insert_with(BTreeMap::new);

        // Remove or nullify the keypair for the specified scheme in the inner map
        inner_map.remove(&scheme);
        Ok(())
    }

    fn generate_session_key(
        &mut self,
        address: &RoochAddress,
    ) -> Result<AuthenticationKey, anyhow::Error> {
        //TODO define derivation_path for session key
        let (_address, kp, _scheme, _phrase) =
            generate_new_key(BuiltinScheme::Ed25519, None, None)?;
        let authentication_key = kp.public().authentication_key();
        let inner_map = self
            .session_keys
            .entry(*address)
            .or_insert_with(BTreeMap::new);
        inner_map.insert(authentication_key.clone(), kp);
        Ok(authentication_key)
    }

    fn sign_transaction_via_session_key(
        &self,
        address: &RoochAddress,
        msg: RoochTransactionData,
        authentication_key: &AuthenticationKey,
    ) -> Result<RoochTransaction, signature::Error> {
        let kp = self
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

        let signature = Signature::new_hashed(msg.hash().as_bytes(), kp);

        let auth = authenticator::Authenticator::ed25519(signature);
        Ok(RoochTransaction::new(msg, auth))
    }
}

#[derive(Default)]
pub struct FileBasedKeystore {
    keystore: BaseKeyStore,
    path: Option<PathBuf>,
}

impl Serialize for FileBasedKeystore {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(
            self.path
                .as_ref()
                .unwrap_or(&PathBuf::default())
                .to_str()
                .unwrap_or(""),
        )
    }
}

impl<'de> Deserialize<'de> for FileBasedKeystore {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error;
        FileBasedKeystore::new(&PathBuf::from(String::deserialize(deserializer)?))
            .map_err(D::Error::custom)
    }
}

impl AccountKeystore for FileBasedKeystore {
    fn get_key_pair_by_scheme(
        &self,
        address: &RoochAddress,
        scheme: BuiltinScheme,
    ) -> Result<&RoochKeyPair, signature::Error> {
        self.keystore.get_key_pair_by_scheme(address, scheme)
    }

    fn sign_hashed(
        &self,
        address: &RoochAddress,
        msg: &[u8],
        scheme: BuiltinScheme,
    ) -> Result<Signature, signature::Error> {
        self.keystore.sign_hashed(address, msg, scheme)
    }

    fn sign_secure<T>(
        &self,
        address: &RoochAddress,
        msg: &T,
        scheme: BuiltinScheme,
    ) -> Result<Signature, signature::Error>
    where
        T: Serialize,
    {
        self.keystore.sign_secure(address, msg, scheme)
    }

    fn sign_transaction(
        &self,
        address: &RoochAddress,
        msg: RoochTransactionData,
        scheme: BuiltinScheme,
    ) -> Result<RoochTransaction, signature::Error> {
        self.keystore.sign_transaction(address, msg, scheme)
    }

    fn add_key_pair_by_scheme(
        &mut self,
        keypair: RoochKeyPair,
        scheme: BuiltinScheme,
    ) -> Result<(), anyhow::Error> {
        self.keystore.add_key_pair_by_scheme(keypair, scheme)?;
        //TODO should check test env at here?
        if std::env::var_os("TEST_ENV").is_none() {
            self.save()?;
        }
        Ok(())
    }

    fn get_public_key_by_scheme(&self, scheme: BuiltinScheme) -> Result<PublicKey, anyhow::Error> {
        self.keystore.get_public_key_by_scheme(scheme)
    }

    fn get_address_public_keys(&self) -> Vec<(RoochAddress, PublicKey)> {
        self.keystore.get_address_public_keys()
    }

    fn get_key_pairs(&self, address: &RoochAddress) -> Result<Vec<&RoochKeyPair>, anyhow::Error> {
        self.keystore.get_key_pairs(address)
    }

    fn update_key_pair_by_scheme(
        &mut self,
        address: &RoochAddress,
        keypair: RoochKeyPair,
        scheme: BuiltinScheme,
    ) -> Result<(), anyhow::Error> {
        self.keystore
            .update_key_pair_by_scheme(address, keypair, scheme)?;
        //TODO should check test env at here?
        if std::env::var_os("TEST_ENV").is_none() {
            self.save()?;
        }
        Ok(())
    }

    fn nullify_key_pair_by_scheme(
        &mut self,
        address: &RoochAddress,
        scheme: BuiltinScheme,
    ) -> Result<(), anyhow::Error> {
        self.keystore.nullify_key_pair_by_scheme(address, scheme)?;
        //TODO should check test env at here?
        if std::env::var_os("TEST_ENV").is_none() {
            self.save()?;
        }
        Ok(())
    }

    fn generate_session_key(
        &mut self,
        address: &RoochAddress,
    ) -> Result<AuthenticationKey, anyhow::Error> {
        let auth_key = self.keystore.generate_session_key(address)?;
        self.save()?;
        Ok(auth_key)
    }

    fn sign_transaction_via_session_key(
        &self,
        address: &RoochAddress,
        msg: RoochTransactionData,
        authentication_key: &AuthenticationKey,
    ) -> Result<RoochTransaction, signature::Error> {
        self.keystore
            .sign_transaction_via_session_key(address, msg, authentication_key)
    }
}

impl FileBasedKeystore {
    pub fn new(path: &PathBuf) -> Result<Self, anyhow::Error> {
        let keystore = if path.exists() {
            let reader = BufReader::new(
                File::open(path)
                    .map_err(|e| anyhow!("Can't open FileBasedKeystore from {:?}: {}", path, e))?,
            );
            serde_json::from_reader(reader).map_err(|e| {
                anyhow!("Can't deserialize FileBasedKeystore from {:?}: {}", path, e)
            })?
        } else {
            BaseKeyStore::new(BTreeMap::new())
        };

        Ok(Self {
            keystore,
            path: Some(path.to_path_buf()),
        })
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

    pub fn key_pairs(&self) -> Vec<&RoochKeyPair> {
        self.keystore
            .keys
            .values()
            .flat_map(|inner_map| inner_map.values())
            .collect()
    }
}

#[derive(Default, Serialize, Deserialize)]
pub struct InMemKeystore {
    keystore: BaseKeyStore,
}

impl AccountKeystore for InMemKeystore {
    fn sign_secure<T>(
        &self,
        address: &RoochAddress,
        msg: &T,
        scheme: BuiltinScheme,
    ) -> Result<Signature, signature::Error>
    where
        T: Serialize,
    {
        self.keystore.sign_secure(address, msg, scheme)
    }

    fn sign_transaction(
        &self,
        address: &RoochAddress,
        msg: RoochTransactionData,
        scheme: BuiltinScheme,
    ) -> Result<RoochTransaction, signature::Error> {
        self.keystore.sign_transaction(address, msg, scheme)
    }

    fn add_key_pair_by_scheme(
        &mut self,
        keypair: RoochKeyPair,
        scheme: BuiltinScheme,
    ) -> Result<(), anyhow::Error> {
        self.keystore.add_key_pair_by_scheme(keypair, scheme)
    }

    fn get_public_key_by_scheme(&self, scheme: BuiltinScheme) -> Result<PublicKey, anyhow::Error> {
        self.keystore.get_public_key_by_scheme(scheme)
    }

    fn get_address_public_keys(&self) -> Vec<(RoochAddress, PublicKey)> {
        self.keystore.get_address_public_keys()
    }

    fn get_key_pairs(&self, address: &RoochAddress) -> Result<Vec<&RoochKeyPair>, anyhow::Error> {
        self.keystore.get_key_pairs(address)
    }

    fn update_key_pair_by_scheme(
        &mut self,
        address: &RoochAddress,
        keypair: RoochKeyPair,
        scheme: BuiltinScheme,
    ) -> Result<(), anyhow::Error> {
        self.keystore
            .update_key_pair_by_scheme(address, keypair, scheme)
    }

    fn nullify_key_pair_by_scheme(
        &mut self,
        address: &RoochAddress,
        scheme: BuiltinScheme,
    ) -> Result<(), anyhow::Error> {
        self.keystore.nullify_key_pair_by_scheme(address, scheme)
    }

    fn get_key_pair_by_scheme(
        &self,
        address: &RoochAddress,
        scheme: BuiltinScheme,
    ) -> Result<&RoochKeyPair, signature::Error> {
        self.keystore.get_key_pair_by_scheme(address, scheme)
    }

    fn sign_hashed(
        &self,
        address: &RoochAddress,
        msg: &[u8],
        scheme: BuiltinScheme,
    ) -> Result<Signature, signature::Error> {
        self.keystore.sign_hashed(address, msg, scheme)
    }

    fn generate_session_key(
        &mut self,
        address: &RoochAddress,
    ) -> Result<AuthenticationKey, anyhow::Error> {
        self.keystore.generate_session_key(address)
    }

    fn sign_transaction_via_session_key(
        &self,
        address: &RoochAddress,
        msg: RoochTransactionData,
        authentication_key: &AuthenticationKey,
    ) -> Result<RoochTransaction, signature::Error> {
        self.keystore
            .sign_transaction_via_session_key(address, msg, authentication_key)
    }
}

impl InMemKeystore {
    pub fn new_ed25519_insecure_for_tests(initial_key_number: usize) -> Self {
        let mut rng = StdRng::from_seed([0; 32]);
        let keys = (0..initial_key_number)
            .map(|_| get_key_pair_from_rng(&mut rng))
            .map(|(ad, k)| {
                (
                    ad,
                    BTreeMap::from_iter(vec![(BuiltinScheme::Ed25519, RoochKeyPair::Ed25519(k))]),
                )
            })
            .collect::<BTreeMap<RoochAddress, BTreeMap<BuiltinScheme, RoochKeyPair>>>();

        Self {
            keystore: BaseKeyStore::new(keys),
        }
    }

    pub fn new_ecdsa_insecure_for_tests(initial_key_number: usize) -> Self {
        let mut rng = StdRng::from_seed([0; 32]);
        let keys = (0..initial_key_number)
            .map(|_| get_key_pair_from_rng(&mut rng))
            .map(|(ad, k)| {
                (
                    ad,
                    BTreeMap::from_iter(vec![(BuiltinScheme::Ecdsa, RoochKeyPair::Ecdsa(k))]),
                )
            })
            .collect::<BTreeMap<RoochAddress, BTreeMap<BuiltinScheme, RoochKeyPair>>>();

        Self {
            keystore: BaseKeyStore::new(keys),
        }
    }

    pub fn new_ecdsa_recoverable_insecure_for_tests(initial_key_number: usize) -> Self {
        let mut rng = StdRng::from_seed([0; 32]);
        let keys = (0..initial_key_number)
            .map(|_| get_key_pair_from_rng(&mut rng))
            .map(|(ad, k)| {
                (
                    ad,
                    BTreeMap::from_iter(vec![(
                        BuiltinScheme::EcdsaRecoverable,
                        RoochKeyPair::EcdsaRecoverable(k),
                    )]),
                )
            })
            .collect::<BTreeMap<RoochAddress, BTreeMap<BuiltinScheme, RoochKeyPair>>>();

        Self {
            keystore: BaseKeyStore::new(keys),
        }
    }

    pub fn new_schnorr_insecure_for_tests(initial_key_number: usize) -> Self {
        let mut rng = StdRng::from_seed([0; 32]);
        let keys = (0..initial_key_number)
            .map(|_| get_key_pair_from_rng(&mut rng))
            .map(|(ad, k)| {
                (
                    ad,
                    BTreeMap::from_iter(vec![(BuiltinScheme::Schnorr, RoochKeyPair::Schnorr(k))]),
                )
            })
            .collect::<BTreeMap<RoochAddress, BTreeMap<BuiltinScheme, RoochKeyPair>>>();

        Self {
            keystore: BaseKeyStore::new(keys),
        }
    }
}

// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::key_derive::{generate_new_key_pair, CoinOperations};
use anyhow::anyhow;
use bip32::DerivationPath;
use bip39::{Language, Mnemonic, Seed};
use enum_dispatch::enum_dispatch;
use fastcrypto::{
    hash::Keccak256,
    secp256k1::recoverable::{
        Secp256k1RecoverableKeyPair, Secp256k1RecoverablePublicKey, Secp256k1RecoverableSignature,
    },
    traits::RecoverableSigner,
};
use rand::{rngs::StdRng, SeedableRng};
use rooch_types::{
    address::{EthereumAddress, RoochAddress},
    authentication_key::AuthenticationKey,
    coin_type::CoinID,
    crypto::{
        get_ethereum_key_pair_from_rng, get_rooch_key_pair_from_rng, PublicKey, RoochKeyPair,
        Signature,
    },
    transaction::{
        authenticator,
        ethereum::{EthereumTransaction, EthereumTransactionData},
        rooch::{RoochTransaction, RoochTransactionData},
    },
};
use serde::{Deserialize, Serialize};
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
pub enum Keystore<K: Ord, V> {
    File(FileBasedKeystore<K, V>),
    InMem(InMemKeystore<K, V>),
}

#[enum_dispatch]
pub trait AccountKeystore<Addr: Copy, PubKey, KeyPair, Sig, Transaction, TransactionData>:
    Send + Sync
{
    fn add_key_pair_by_coin_id(
        &mut self,
        keypair: KeyPair,
        coin_id: CoinID,
    ) -> Result<(), anyhow::Error>;
    fn get_address_public_keys(&self) -> Vec<(Addr, PubKey)>;
    fn get_public_key_by_coin_id(&self, coin_id: CoinID) -> Result<PubKey, anyhow::Error>;
    fn get_key_pairs(&self, address: &Addr) -> Result<Vec<&KeyPair>, anyhow::Error>;
    fn get_key_pair_by_coin_id(
        &self,
        address: &Addr,
        coin_id: CoinID,
    ) -> Result<&KeyPair, signature::Error>;
    fn update_key_pair_by_coin_id(
        &mut self,
        address: &Addr,
        keypair: KeyPair,
        coin_id: CoinID,
    ) -> Result<(), anyhow::Error>;
    fn nullify_key_pair_by_coin_id(
        &mut self,
        address: &Addr,
        coin_id: CoinID,
    ) -> Result<(), anyhow::Error>;

    fn sign_hashed(
        &self,
        address: &Addr,
        msg: &[u8],
        coin_id: CoinID,
    ) -> Result<Sig, signature::Error>;

    fn sign_transaction(
        &self,
        address: &Addr,
        msg: TransactionData,
        coin_id: CoinID,
    ) -> Result<Transaction, signature::Error>;

    fn sign_secure<T>(
        &self,
        address: &Addr,
        msg: &T,
        coin_id: CoinID,
    ) -> Result<Sig, signature::Error>
    where
        T: Serialize;

    fn addresses(&self) -> Vec<Addr> {
        self.get_address_public_keys()
            .iter()
            .map(|(address, _public_key)| *address)
            .collect()
    }

    fn generate_and_add_new_key(
        &mut self,
        coin_id: CoinID,
        derivation_path: Option<DerivationPath>,
        word_length: Option<String>,
    ) -> Result<(Addr, String, CoinID), anyhow::Error>
    where
        CoinID: CoinOperations<Addr, KeyPair>,
    {
        let (address, kp, coin_id, phrase) =
            generate_new_key_pair::<Addr, KeyPair>(coin_id, derivation_path, word_length)?;
        self.add_key_pair_by_coin_id(kp, coin_id)?;
        Ok((address, phrase, coin_id))
    }

    fn import_from_mnemonic(
        &mut self,
        phrase: &str,
        coin_id: CoinID,
        derivation_path: Option<DerivationPath>,
    ) -> Result<Addr, anyhow::Error>
    where
        CoinID: CoinOperations<Addr, KeyPair>,
    {
        let mnemonic = Mnemonic::from_phrase(phrase, Language::English)?;
        let seed = Seed::new(&mnemonic, "");

        let (address, kp) = coin_id.derive_key_pair_from_path(seed.as_bytes(), derivation_path)?;
        {
            self.add_key_pair_by_coin_id(kp, coin_id)?;
            Ok(address)
        }
    }

    fn update_address_with_key_pair_from_coin_id(
        &mut self,
        address: &Addr,
        phrase: String,
        coin_id: CoinID,
        derivation_path: Option<DerivationPath>,
    ) -> Result<KeyPair, anyhow::Error>
    where
        CoinID: CoinOperations<Addr, KeyPair>,
    {
        let mnemonic = Mnemonic::from_phrase(&phrase, Language::English)?;
        let seed = Seed::new(&mnemonic, "");
        let derivation_path_clone = derivation_path.clone();

        // Consider adding Clone capabilities
        let (_, kp) = coin_id.derive_key_pair_from_path(seed.as_bytes(), derivation_path)?;
        {
            self.update_key_pair_by_coin_id(address, kp, coin_id)?;
        };

        let (_, kp) = coin_id.derive_key_pair_from_path(seed.as_bytes(), derivation_path_clone)?;
        Ok(kp)
    }

    fn nullify_address_with_key_pair_from_coin_id(
        &mut self,
        address: &Addr,
        coin_id: CoinID,
    ) -> Result<(), anyhow::Error> {
        self.nullify_key_pair_by_coin_id(address, coin_id)?;
        Ok(())
    }

    fn generate_session_key(&mut self, address: &Addr) -> Result<AuthenticationKey, anyhow::Error>;

    fn sign_transaction_via_session_key(
        &self,
        address: &Addr,
        msg: TransactionData,
        authentication_key: &AuthenticationKey,
    ) -> Result<Transaction, signature::Error>;
}

impl
    AccountKeystore<
        RoochAddress,
        PublicKey,
        RoochKeyPair,
        Signature,
        RoochTransaction,
        RoochTransactionData,
    > for Keystore<RoochAddress, RoochKeyPair>
{
    fn sign_transaction_via_session_key(
        &self,
        address: &RoochAddress,
        msg: RoochTransactionData,
        authentication_key: &AuthenticationKey,
    ) -> Result<RoochTransaction, signature::Error> {
        // Implement this method by delegating the call to the appropriate variant (File or InMem)
        match self {
            Keystore::File(file_keystore) => {
                file_keystore.sign_transaction_via_session_key(address, msg, authentication_key)
            }
            Keystore::InMem(inmem_keystore) => {
                inmem_keystore.sign_transaction_via_session_key(address, msg, authentication_key)
            }
        }
    }

    fn add_key_pair_by_coin_id(
        &mut self,
        keypair: RoochKeyPair,
        coin_id: CoinID,
    ) -> Result<(), anyhow::Error> {
        // Implement this method to add a key pair to the appropriate variant (File or InMem)
        match self {
            Keystore::File(file_keystore) => {
                file_keystore.add_key_pair_by_coin_id(keypair, coin_id)
            }
            Keystore::InMem(inmem_keystore) => {
                inmem_keystore.add_key_pair_by_coin_id(keypair, coin_id)
            }
        }
    }

    fn get_address_public_keys(&self) -> Vec<(RoochAddress, PublicKey)> {
        // Implement this method to collect public keys from the appropriate variant (File or InMem)
        match self {
            Keystore::File(file_keystore) => file_keystore.get_address_public_keys(),
            Keystore::InMem(inmem_keystore) => inmem_keystore.get_address_public_keys(),
        }
    }

    fn get_public_key_by_coin_id(&self, coin_id: CoinID) -> Result<PublicKey, anyhow::Error> {
        // Implement this method to get the public key by coin ID from the appropriate variant (File or InMem)
        match self {
            Keystore::File(file_keystore) => file_keystore.get_public_key_by_coin_id(coin_id),
            Keystore::InMem(inmem_keystore) => inmem_keystore.get_public_key_by_coin_id(coin_id),
        }
    }

    fn get_key_pairs(&self, address: &RoochAddress) -> Result<Vec<&RoochKeyPair>, anyhow::Error> {
        // Implement this method to get key pairs for the given address from the appropriate variant (File or InMem)
        match self {
            Keystore::File(file_keystore) => file_keystore.get_key_pairs(address),
            Keystore::InMem(inmem_keystore) => inmem_keystore.get_key_pairs(address),
        }
    }

    fn get_key_pair_by_coin_id(
        &self,
        address: &RoochAddress,
        coin_id: CoinID,
    ) -> Result<&RoochKeyPair, signature::Error> {
        // Implement this method to get the key pair by coin ID from the appropriate variant (File or InMem)
        match self {
            Keystore::File(file_keystore) => {
                file_keystore.get_key_pair_by_coin_id(address, coin_id)
            }
            Keystore::InMem(inmem_keystore) => {
                inmem_keystore.get_key_pair_by_coin_id(address, coin_id)
            }
        }
    }

    fn update_key_pair_by_coin_id(
        &mut self,
        address: &RoochAddress,
        keypair: RoochKeyPair,
        coin_id: CoinID,
    ) -> Result<(), anyhow::Error> {
        // Implement this method to update the key pair by coin ID for the appropriate variant (File or InMem)
        match self {
            Keystore::File(file_keystore) => {
                file_keystore.update_key_pair_by_coin_id(address, keypair, coin_id)
            }
            Keystore::InMem(inmem_keystore) => {
                inmem_keystore.update_key_pair_by_coin_id(address, keypair, coin_id)
            }
        }
    }

    fn nullify_key_pair_by_coin_id(
        &mut self,
        address: &RoochAddress,
        coin_id: CoinID,
    ) -> Result<(), anyhow::Error> {
        // Implement this method to nullify the key pair by coin ID for the appropriate variant (File or InMem)
        match self {
            Keystore::File(file_keystore) => {
                file_keystore.nullify_key_pair_by_coin_id(address, coin_id)
            }
            Keystore::InMem(inmem_keystore) => {
                inmem_keystore.nullify_key_pair_by_coin_id(address, coin_id)
            }
        }
    }

    fn sign_hashed(
        &self,
        address: &RoochAddress,
        msg: &[u8],
        coin_id: CoinID,
    ) -> Result<Signature, signature::Error> {
        // Implement this method to sign a hashed message for the appropriate variant (File or InMem)
        match self {
            Keystore::File(file_keystore) => file_keystore.sign_hashed(address, msg, coin_id),
            Keystore::InMem(inmem_keystore) => inmem_keystore.sign_hashed(address, msg, coin_id),
        }
    }

    fn sign_transaction(
        &self,
        address: &RoochAddress,
        msg: RoochTransactionData,
        coin_id: CoinID,
    ) -> Result<RoochTransaction, signature::Error> {
        // Implement this method to sign a transaction for the appropriate variant (File or InMem)
        match self {
            Keystore::File(file_keystore) => file_keystore.sign_transaction(address, msg, coin_id),
            Keystore::InMem(inmem_keystore) => {
                inmem_keystore.sign_transaction(address, msg, coin_id)
            }
        }
    }

    fn sign_secure<T>(
        &self,
        address: &RoochAddress,
        msg: &T,
        coin_id: CoinID,
    ) -> Result<Signature, signature::Error>
    where
        T: Serialize,
    {
        // Implement this method to sign a secure message for the appropriate variant (File or InMem)
        match self {
            Keystore::File(file_keystore) => file_keystore.sign_secure(address, msg, coin_id),
            Keystore::InMem(inmem_keystore) => inmem_keystore.sign_secure(address, msg, coin_id),
        }
    }

    fn generate_session_key(
        &mut self,
        address: &RoochAddress,
    ) -> Result<AuthenticationKey, anyhow::Error> {
        // Implement this method to generate a session key for the appropriate variant (File or InMem)
        match self {
            Keystore::File(file_keystore) => file_keystore.generate_session_key(address),
            Keystore::InMem(inmem_keystore) => inmem_keystore.generate_session_key(address),
        }
    }
}

impl
    AccountKeystore<
        EthereumAddress,
        Secp256k1RecoverablePublicKey,
        Secp256k1RecoverableKeyPair,
        Secp256k1RecoverableSignature,
        EthereumTransaction,
        EthereumTransactionData,
    > for Keystore<EthereumAddress, Secp256k1RecoverableKeyPair>
{
    fn sign_transaction_via_session_key(
        &self,
        address: &EthereumAddress,
        msg: EthereumTransactionData,
        authentication_key: &AuthenticationKey,
    ) -> Result<EthereumTransaction, signature::Error> {
        match self {
            Keystore::File(file_keystore) => {
                file_keystore.sign_transaction_via_session_key(address, msg, authentication_key)
            }
            Keystore::InMem(inmem_keystore) => {
                inmem_keystore.sign_transaction_via_session_key(address, msg, authentication_key)
            }
        }
    }

    fn add_key_pair_by_coin_id(
        &mut self,
        keypair: Secp256k1RecoverableKeyPair,
        coin_id: CoinID,
    ) -> Result<(), anyhow::Error> {
        // Implement this method to add a key pair to the appropriate variant (File or InMem)
        match self {
            Keystore::File(file_keystore) => {
                file_keystore.add_key_pair_by_coin_id(keypair, coin_id)
            }
            Keystore::InMem(inmem_keystore) => {
                inmem_keystore.add_key_pair_by_coin_id(keypair, coin_id)
            }
        }
    }

    fn get_address_public_keys(&self) -> Vec<(EthereumAddress, Secp256k1RecoverablePublicKey)> {
        // Implement this method to collect public keys from the appropriate variant (File or InMem)
        match self {
            Keystore::File(file_keystore) => file_keystore.get_address_public_keys(),
            Keystore::InMem(inmem_keystore) => inmem_keystore.get_address_public_keys(),
        }
    }

    fn get_public_key_by_coin_id(
        &self,
        coin_id: CoinID,
    ) -> Result<Secp256k1RecoverablePublicKey, anyhow::Error> {
        // Implement this method to get the public key by coin ID from the appropriate variant (File or InMem)
        match self {
            Keystore::File(file_keystore) => file_keystore.get_public_key_by_coin_id(coin_id),
            Keystore::InMem(inmem_keystore) => inmem_keystore.get_public_key_by_coin_id(coin_id),
        }
    }

    fn get_key_pairs(
        &self,
        address: &EthereumAddress,
    ) -> Result<Vec<&Secp256k1RecoverableKeyPair>, anyhow::Error> {
        // Implement this method to get key pairs for the given address from the appropriate variant (File or InMem)
        match self {
            Keystore::File(file_keystore) => file_keystore.get_key_pairs(address),
            Keystore::InMem(inmem_keystore) => inmem_keystore.get_key_pairs(address),
        }
    }

    fn get_key_pair_by_coin_id(
        &self,
        address: &EthereumAddress,
        coin_id: CoinID,
    ) -> Result<&Secp256k1RecoverableKeyPair, signature::Error> {
        // Implement this method to get a key pair by coin ID from the appropriate variant (File or InMem)
        match self {
            Keystore::File(file_keystore) => {
                file_keystore.get_key_pair_by_coin_id(address, coin_id)
            }
            Keystore::InMem(inmem_keystore) => {
                inmem_keystore.get_key_pair_by_coin_id(address, coin_id)
            }
        }
    }

    fn update_key_pair_by_coin_id(
        &mut self,
        address: &EthereumAddress,
        keypair: Secp256k1RecoverableKeyPair,
        coin_id: CoinID,
    ) -> Result<(), anyhow::Error> {
        // Implement this method to update a key pair by coin ID in the appropriate variant (File or InMem)
        match self {
            Keystore::File(file_keystore) => {
                file_keystore.update_key_pair_by_coin_id(address, keypair, coin_id)
            }
            Keystore::InMem(inmem_keystore) => {
                inmem_keystore.update_key_pair_by_coin_id(address, keypair, coin_id)
            }
        }
    }

    fn nullify_key_pair_by_coin_id(
        &mut self,
        address: &EthereumAddress,
        coin_id: CoinID,
    ) -> Result<(), anyhow::Error> {
        // Implement this method to nullify a key pair by coin ID in the appropriate variant (File or InMem)
        match self {
            Keystore::File(file_keystore) => {
                file_keystore.nullify_key_pair_by_coin_id(address, coin_id)
            }
            Keystore::InMem(inmem_keystore) => {
                inmem_keystore.nullify_key_pair_by_coin_id(address, coin_id)
            }
        }
    }

    fn sign_hashed(
        &self,
        address: &EthereumAddress,
        msg: &[u8],
        coin_id: CoinID,
    ) -> Result<Secp256k1RecoverableSignature, signature::Error> {
        // Implement this method to sign a hashed message with the key pair for the given address and coin ID
        match self {
            Keystore::File(file_keystore) => file_keystore.sign_hashed(address, msg, coin_id),
            Keystore::InMem(inmem_keystore) => inmem_keystore.sign_hashed(address, msg, coin_id),
        }
    }

    fn sign_transaction(
        &self,
        address: &EthereumAddress,
        msg: EthereumTransactionData,
        coin_id: CoinID,
    ) -> Result<EthereumTransaction, signature::Error> {
        // Implement this method to sign a transaction with the key pair for the given address and coin ID
        match self {
            Keystore::File(file_keystore) => file_keystore.sign_transaction(address, msg, coin_id),
            Keystore::InMem(inmem_keystore) => {
                inmem_keystore.sign_transaction(address, msg, coin_id)
            }
        }
    }

    fn sign_secure<T>(
        &self,
        address: &EthereumAddress,
        msg: &T,
        coin_id: CoinID,
    ) -> Result<Secp256k1RecoverableSignature, signature::Error>
    where
        T: Serialize,
    {
        // Implement this method to sign a serializable message with the key pair for the given address and coin ID
        match self {
            Keystore::File(file_keystore) => file_keystore.sign_secure(address, msg, coin_id),
            Keystore::InMem(inmem_keystore) => inmem_keystore.sign_secure(address, msg, coin_id),
        }
    }

    fn generate_session_key(
        &mut self,
        address: &EthereumAddress,
    ) -> Result<AuthenticationKey, anyhow::Error> {
        // Implement this method to generate a session key for the given address
        match self {
            Keystore::File(file_keystore) => file_keystore.generate_session_key(address),
            Keystore::InMem(inmem_keystore) => inmem_keystore.generate_session_key(address),
        }
    }
}

impl Display for Keystore<RoochAddress, RoochKeyPair> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut writer = String::new();
        match self {
            Keystore::File(file) => {
                writeln!(writer, "Keystore Type : Rooch File")?;
                write!(writer, "Keystore Path : {:?}", file.path)?;
                write!(f, "{}", writer)?;
            }
            Keystore::InMem(_) => {
                writeln!(writer, "Keystore Type : Rooch InMem")?;
                write!(f, "{}", writer)?;
            }
        }
        Ok(())
    }
}

impl Display for Keystore<EthereumAddress, Secp256k1RecoverableKeyPair> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut writer = String::new();
        match self {
            Keystore::File(file) => {
                writeln!(writer, "Keystore Type : Ethereum File")?;
                write!(writer, "Keystore Path : {:?}", file.path)?;
                write!(f, "{}", writer)?;
            }
            Keystore::InMem(_) => {
                writeln!(writer, "Keystore Type : Ethereum InMem")?;
                write!(f, "{}", writer)?;
            }
        }
        Ok(())
    }
}

#[derive(Default, Serialize, Deserialize)]
#[serde_as]
pub(crate) struct BaseKeyStore<K, V>
where
    K: Ord,
{
    keys: BTreeMap<K, BTreeMap<CoinID, V>>,
    /// RoochAddress -> BTreeMap<AuthenticationKey, RoochKeyPair>
    /// EthereumAddress -> BTreeMap<AuthenticationKey, Secp256k1RecoverableKeyPair>

    #[serde_as(as = "BTreeMap<DisplayFromStr, BTreeMap<DisplayFromStr, _>>")]
    session_keys: BTreeMap<K, BTreeMap<AuthenticationKey, V>>,
}

impl<K, V> BaseKeyStore<K, V>
where
    K: Serialize + Deserialize<'static> + Ord,
    V: Serialize + Deserialize<'static>,
{
    pub fn new(keys: BTreeMap<K, BTreeMap<CoinID, V>>) -> Self {
        Self {
            keys,
            session_keys: BTreeMap::new(),
        }
    }
}

impl
    AccountKeystore<
        RoochAddress,
        PublicKey,
        RoochKeyPair,
        Signature,
        RoochTransaction,
        RoochTransactionData,
    > for BaseKeyStore<RoochAddress, RoochKeyPair>
{
    fn get_key_pair_by_coin_id(
        &self,
        address: &RoochAddress,
        coin_id: CoinID,
    ) -> Result<&RoochKeyPair, signature::Error> {
        if let Some(inner_map) = self.keys.get(address) {
            if let Some(keypair) = inner_map.get(&coin_id) {
                Ok(keypair)
            } else {
                Err(signature::Error::from_source(format!(
                    "CoinID not found for address: [{:?}]",
                    coin_id
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
        coin_id: CoinID,
    ) -> Result<Signature, signature::Error> {
        Ok(Signature::new_hashed(
            msg,
            self.get_key_pair_by_coin_id(address, coin_id)?,
        ))
    }

    fn sign_secure<T>(
        &self,
        address: &RoochAddress,
        msg: &T,
        coin_id: CoinID,
    ) -> Result<Signature, signature::Error>
    where
        T: Serialize,
    {
        Ok(Signature::new_secure(
            msg,
            self.get_key_pair_by_coin_id(address, coin_id)?,
        ))
    }

    fn sign_transaction(
        &self,
        address: &RoochAddress,
        msg: RoochTransactionData,
        coin_id: CoinID,
    ) -> Result<RoochTransaction, signature::Error> {
        let kp = self
            .get_key_pair_by_coin_id(address, coin_id)
            .ok()
            .ok_or_else(|| {
                signature::Error::from_source(format!("Cannot find key for address: [{address}]"))
            })?;

        let signature = Signature::new_hashed(msg.hash().as_bytes(), kp);

        let auth = authenticator::Authenticator::rooch(signature);

        Ok(RoochTransaction::new(msg, auth))
    }

    fn add_key_pair_by_coin_id(
        &mut self,
        keypair: RoochKeyPair,
        coin_id: CoinID,
    ) -> Result<(), anyhow::Error> {
        let address: RoochAddress = (&keypair.public()).into();
        self.keys
            .entry(address)
            .or_insert_with(BTreeMap::new)
            .insert(coin_id, keypair);
        Ok(())
    }

    fn get_public_key_by_coin_id(&self, coin_id: CoinID) -> Result<PublicKey, anyhow::Error> {
        for inner_map in self.keys.values() {
            if let Some(keypair) = inner_map.get(&coin_id) {
                return Ok(keypair.public());
            }
        }
        Err(anyhow!("Cannot find key for coin id: [{:?}]", coin_id))
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

    fn update_key_pair_by_coin_id(
        &mut self,
        address: &RoochAddress,
        keypair: RoochKeyPair,
        coin_id: CoinID,
    ) -> Result<(), anyhow::Error> {
        // First, get the inner map associated with the address
        let inner_map = self.keys.entry(*address).or_insert_with(BTreeMap::new);

        // Insert or update the keypair for the specified coin in the inner map
        inner_map.insert(coin_id, keypair);
        Ok(())
    }

    fn nullify_key_pair_by_coin_id(
        &mut self,
        address: &RoochAddress,
        coin_id: CoinID,
    ) -> Result<(), anyhow::Error> {
        // First, get the inner map associated with the address
        let inner_map = self.keys.entry(*address).or_insert_with(BTreeMap::new);

        // Remove or nullify the keypair for the specified coin in the inner map
        inner_map.remove(&coin_id);
        Ok(())
    }

    fn generate_session_key(
        &mut self,
        address: &RoochAddress,
    ) -> Result<AuthenticationKey, anyhow::Error> {
        //TODO define derivation_path for session key
        let (_address, kp, _coin_id, _phrase) =
            generate_new_key_pair::<RoochAddress, RoochKeyPair>(CoinID::Rooch, None, None)?;
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

        let auth = authenticator::Authenticator::rooch(signature);
        Ok(RoochTransaction::new(msg, auth))
    }
}

impl
    AccountKeystore<
        EthereumAddress,
        Secp256k1RecoverablePublicKey,
        Secp256k1RecoverableKeyPair,
        Secp256k1RecoverableSignature,
        EthereumTransaction,
        EthereumTransactionData,
    > for BaseKeyStore<EthereumAddress, Secp256k1RecoverableKeyPair>
{
    fn get_key_pair_by_coin_id(
        &self,
        address: &EthereumAddress,
        coin_id: CoinID,
    ) -> Result<&Secp256k1RecoverableKeyPair, signature::Error> {
        if let Some(inner_map) = self.keys.get(address) {
            if let Some(keypair) = inner_map.get(&coin_id) {
                Ok(keypair)
            } else {
                Err(signature::Error::from_source(format!(
                    "CoinID not found for address: [{:?}]",
                    coin_id
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
        address: &EthereumAddress,
        msg: &[u8],
        coin_id: CoinID,
    ) -> Result<Secp256k1RecoverableSignature, signature::Error> {
        let key_pair = self.get_key_pair_by_coin_id(address, coin_id)?;
        Ok(key_pair.sign_recoverable_with_hash::<Keccak256>(msg))
    }

    fn sign_secure<T>(
        &self,
        address: &EthereumAddress,
        msg: &T,
        coin_id: CoinID,
    ) -> Result<Secp256k1RecoverableSignature, signature::Error>
    where
        T: Serialize,
    {
        let key_pair = self.get_key_pair_by_coin_id(address, coin_id)?;
        // Serialize the message into a byte slice
        let message_bytes = serde_json::to_vec(msg).unwrap();
        Ok(key_pair.sign_recoverable(message_bytes.as_slice()))
    }

    fn sign_transaction(
        &self,
        _address: &EthereumAddress,
        msg: EthereumTransactionData,
        _coin_id: CoinID,
    ) -> Result<EthereumTransaction, signature::Error> {
        let signature = EthereumTransactionData::into_signature(&msg).unwrap();
        let auth = authenticator::Authenticator::ethereum(signature);
        Ok(EthereumTransaction::new(msg, auth))
    }

    fn add_key_pair_by_coin_id(
        &mut self,
        keypair: Secp256k1RecoverableKeyPair,
        coin_id: CoinID,
    ) -> Result<(), anyhow::Error> {
        let address = EthereumAddress::from(keypair.public.clone());
        self.keys
            .entry(address)
            .or_insert_with(BTreeMap::new)
            .insert(coin_id, keypair);
        Ok(())
    }

    fn get_public_key_by_coin_id(
        &self,
        coin_id: CoinID,
    ) -> Result<Secp256k1RecoverablePublicKey, anyhow::Error> {
        for inner_map in self.keys.values() {
            if let Some(keypair) = inner_map.get(&coin_id) {
                return Ok(keypair.public.clone());
            }
        }
        Err(anyhow!("Cannot find key for coin id: [{:?}]", coin_id))
    }

    fn get_address_public_keys(&self) -> Vec<(EthereumAddress, Secp256k1RecoverablePublicKey)> {
        let mut result = Vec::new();
        for (address, inner_map) in &self.keys {
            for keypair in inner_map.values() {
                let public_key = keypair.public.clone();
                result.push((*address, public_key));
            }
        }
        result
    }

    fn get_key_pairs(
        &self,
        address: &EthereumAddress,
    ) -> Result<Vec<&Secp256k1RecoverableKeyPair>, anyhow::Error> {
        match self.keys.get(address) {
            Some(key_map) => {
                // Collect references to RoochKeyPair objects from the inner map into a Vec.
                let key_pairs: Vec<&Secp256k1RecoverableKeyPair> = key_map.values().collect();
                Ok(key_pairs)
            }
            None => Err(anyhow!("Cannot find key for address: [{address}]")),
        }
    }

    fn update_key_pair_by_coin_id(
        &mut self,
        address: &EthereumAddress,
        keypair: Secp256k1RecoverableKeyPair,
        coin_id: CoinID,
    ) -> Result<(), anyhow::Error> {
        // First, get the inner map associated with the address
        let inner_map = self.keys.entry(*address).or_insert_with(BTreeMap::new);

        // Insert or update the keypair for the specified coin in the inner map
        inner_map.insert(coin_id, keypair);
        Ok(())
    }

    fn nullify_key_pair_by_coin_id(
        &mut self,
        address: &EthereumAddress,
        coin_id: CoinID,
    ) -> Result<(), anyhow::Error> {
        // First, get the inner map associated with the address
        let inner_map = self.keys.entry(*address).or_insert_with(BTreeMap::new);

        // Remove or nullify the keypair for the specified coin in the inner map
        inner_map.remove(&coin_id);
        Ok(())
    }

    fn generate_session_key(
        &mut self,
        address: &EthereumAddress,
    ) -> Result<AuthenticationKey, anyhow::Error> {
        //TODO define derivation_path for session key
        let (_, kp, _coin_id, _phrase) = generate_new_key_pair::<
            EthereumAddress,
            Secp256k1RecoverableKeyPair,
        >(CoinID::Ether, None, None)?;
        let authentication_key_bytes = address.0.as_bytes().to_vec();
        let authentication_key = AuthenticationKey::new(authentication_key_bytes);
        let inner_map = self
            .session_keys
            .entry(*address)
            .or_insert_with(BTreeMap::new);
        inner_map.insert(authentication_key.clone(), kp);
        Ok(authentication_key)
    }

    fn sign_transaction_via_session_key(
        &self,
        address: &EthereumAddress,
        msg: EthereumTransactionData,
        authentication_key: &AuthenticationKey,
    ) -> Result<EthereumTransaction, signature::Error> {
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

        let signature = kp.sign_recoverable_with_hash::<Keccak256>(msg.0.hash().as_bytes());

        let auth = authenticator::Authenticator::ethereum(signature);
        Ok(EthereumTransaction::new(msg, auth))
    }
}

#[derive(Default, Serialize, Deserialize)]
pub struct FileBasedKeystore<K: Ord, V> {
    keystore: BaseKeyStore<K, V>,
    path: Option<PathBuf>,
}

impl
    AccountKeystore<
        RoochAddress,
        PublicKey,
        RoochKeyPair,
        Signature,
        RoochTransaction,
        RoochTransactionData,
    > for FileBasedKeystore<RoochAddress, RoochKeyPair>
{
    fn get_key_pair_by_coin_id(
        &self,
        address: &RoochAddress,
        coin_id: CoinID,
    ) -> Result<&RoochKeyPair, signature::Error> {
        self.keystore.get_key_pair_by_coin_id(address, coin_id)
    }

    fn sign_hashed(
        &self,
        address: &RoochAddress,
        msg: &[u8],
        coin_id: CoinID,
    ) -> Result<Signature, signature::Error> {
        self.keystore.sign_hashed(address, msg, coin_id)
    }

    fn sign_secure<T>(
        &self,
        address: &RoochAddress,
        msg: &T,
        coin_id: CoinID,
    ) -> Result<Signature, signature::Error>
    where
        T: Serialize,
    {
        self.keystore.sign_secure(address, msg, coin_id)
    }

    fn sign_transaction(
        &self,
        address: &RoochAddress,
        msg: RoochTransactionData,
        coin_id: CoinID,
    ) -> Result<RoochTransaction, signature::Error> {
        self.keystore.sign_transaction(address, msg, coin_id)
    }

    fn add_key_pair_by_coin_id(
        &mut self,
        keypair: RoochKeyPair,
        coin_id: CoinID,
    ) -> Result<(), anyhow::Error> {
        self.keystore.add_key_pair_by_coin_id(keypair, coin_id)?;
        //TODO should check test env at here?
        if std::env::var_os("TEST_ENV").is_none() {
            self.save()?;
        }
        Ok(())
    }

    fn get_public_key_by_coin_id(&self, coin_id: CoinID) -> Result<PublicKey, anyhow::Error> {
        self.keystore.get_public_key_by_coin_id(coin_id)
    }

    fn get_address_public_keys(&self) -> Vec<(RoochAddress, PublicKey)> {
        self.keystore.get_address_public_keys()
    }

    fn get_key_pairs(&self, address: &RoochAddress) -> Result<Vec<&RoochKeyPair>, anyhow::Error> {
        self.keystore.get_key_pairs(address)
    }

    fn update_key_pair_by_coin_id(
        &mut self,
        address: &RoochAddress,
        keypair: RoochKeyPair,
        coin_id: CoinID,
    ) -> Result<(), anyhow::Error> {
        self.keystore
            .update_key_pair_by_coin_id(address, keypair, coin_id)?;
        //TODO should check test env at here?
        if std::env::var_os("TEST_ENV").is_none() {
            self.save()?;
        }
        Ok(())
    }

    fn nullify_key_pair_by_coin_id(
        &mut self,
        address: &RoochAddress,
        coin_id: CoinID,
    ) -> Result<(), anyhow::Error> {
        self.keystore
            .nullify_key_pair_by_coin_id(address, coin_id)?;
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

impl
    AccountKeystore<
        EthereumAddress,
        Secp256k1RecoverablePublicKey,
        Secp256k1RecoverableKeyPair,
        Secp256k1RecoverableSignature,
        EthereumTransaction,
        EthereumTransactionData,
    > for FileBasedKeystore<EthereumAddress, Secp256k1RecoverableKeyPair>
{
    fn get_key_pair_by_coin_id(
        &self,
        address: &EthereumAddress,
        coin_id: CoinID,
    ) -> Result<&Secp256k1RecoverableKeyPair, signature::Error> {
        self.keystore.get_key_pair_by_coin_id(address, coin_id)
    }

    fn sign_hashed(
        &self,
        address: &EthereumAddress,
        msg: &[u8],
        coin_id: CoinID,
    ) -> Result<Secp256k1RecoverableSignature, signature::Error> {
        self.keystore.sign_hashed(address, msg, coin_id)
    }

    fn sign_secure<T>(
        &self,
        address: &EthereumAddress,
        msg: &T,
        coin_id: CoinID,
    ) -> Result<Secp256k1RecoverableSignature, signature::Error>
    where
        T: Serialize,
    {
        self.keystore.sign_secure(address, msg, coin_id)
    }

    fn sign_transaction(
        &self,
        address: &EthereumAddress,
        msg: EthereumTransactionData,
        coin_id: CoinID,
    ) -> Result<EthereumTransaction, signature::Error> {
        self.keystore.sign_transaction(address, msg, coin_id)
    }

    fn add_key_pair_by_coin_id(
        &mut self,
        keypair: Secp256k1RecoverableKeyPair,
        coin_id: CoinID,
    ) -> Result<(), anyhow::Error> {
        self.keystore.add_key_pair_by_coin_id(keypair, coin_id)?;
        //TODO should check test env at here?
        if std::env::var_os("TEST_ENV").is_none() {
            self.save()?;
        }
        Ok(())
    }

    fn get_public_key_by_coin_id(
        &self,
        coin_id: CoinID,
    ) -> Result<Secp256k1RecoverablePublicKey, anyhow::Error> {
        self.keystore.get_public_key_by_coin_id(coin_id)
    }

    fn get_address_public_keys(&self) -> Vec<(EthereumAddress, Secp256k1RecoverablePublicKey)> {
        self.keystore.get_address_public_keys()
    }

    fn get_key_pairs(
        &self,
        address: &EthereumAddress,
    ) -> Result<Vec<&Secp256k1RecoverableKeyPair>, anyhow::Error> {
        self.keystore.get_key_pairs(address)
    }

    fn update_key_pair_by_coin_id(
        &mut self,
        address: &EthereumAddress,
        keypair: Secp256k1RecoverableKeyPair,
        coin_id: CoinID,
    ) -> Result<(), anyhow::Error> {
        self.keystore
            .update_key_pair_by_coin_id(address, keypair, coin_id)?;
        //TODO should check test env at here?
        if std::env::var_os("TEST_ENV").is_none() {
            self.save()?;
        }
        Ok(())
    }

    fn nullify_key_pair_by_coin_id(
        &mut self,
        address: &EthereumAddress,
        coin_id: CoinID,
    ) -> Result<(), anyhow::Error> {
        self.keystore
            .nullify_key_pair_by_coin_id(address, coin_id)?;
        //TODO should check test env at here?
        if std::env::var_os("TEST_ENV").is_none() {
            self.save()?;
        }
        Ok(())
    }

    fn generate_session_key(
        &mut self,
        address: &EthereumAddress,
    ) -> Result<AuthenticationKey, anyhow::Error> {
        let auth_key = self.keystore.generate_session_key(address)?;
        self.save()?;
        Ok(auth_key)
    }

    fn sign_transaction_via_session_key(
        &self,
        address: &EthereumAddress,
        msg: EthereumTransactionData,
        authentication_key: &AuthenticationKey,
    ) -> Result<EthereumTransaction, signature::Error> {
        self.keystore
            .sign_transaction_via_session_key(address, msg, authentication_key)
    }
}

impl FileBasedKeystore<RoochAddress, RoochKeyPair> {
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

impl FileBasedKeystore<EthereumAddress, Secp256k1RecoverableKeyPair> {
    pub fn new(path: &PathBuf) -> Result<Self, anyhow::Error> {
        let keystore = if path.exists() {
            let reader = BufReader::new(File::open(path).map_err(|e| {
                anyhow!(
                    "Can't open FileBasedKeystore from Ethereum path {:?}: {}",
                    path,
                    e
                )
            })?);
            serde_json::from_reader(reader).map_err(|e| {
                anyhow!(
                    "Can't deserialize FileBasedKeystore from Ethereum path {:?}: {}",
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

    pub fn key_pairs(&self) -> Vec<&Secp256k1RecoverableKeyPair> {
        self.keystore
            .keys
            .values()
            .flat_map(|inner_map| inner_map.values())
            .collect()
    }
}

#[derive(Default, Serialize, Deserialize)]
pub struct InMemKeystore<K: Ord, V> {
    keystore: BaseKeyStore<K, V>,
}

impl
    AccountKeystore<
        RoochAddress,
        PublicKey,
        RoochKeyPair,
        Signature,
        RoochTransaction,
        RoochTransactionData,
    > for InMemKeystore<RoochAddress, RoochKeyPair>
{
    fn sign_secure<T>(
        &self,
        address: &RoochAddress,
        msg: &T,
        coin_id: CoinID,
    ) -> Result<Signature, signature::Error>
    where
        T: Serialize,
    {
        self.keystore.sign_secure(address, msg, coin_id)
    }

    fn sign_transaction(
        &self,
        address: &RoochAddress,
        msg: RoochTransactionData,
        coin_id: CoinID,
    ) -> Result<RoochTransaction, signature::Error> {
        self.keystore.sign_transaction(address, msg, coin_id)
    }

    fn add_key_pair_by_coin_id(
        &mut self,
        keypair: RoochKeyPair,
        coin_id: CoinID,
    ) -> Result<(), anyhow::Error> {
        self.keystore.add_key_pair_by_coin_id(keypair, coin_id)
    }

    fn get_public_key_by_coin_id(&self, coin_id: CoinID) -> Result<PublicKey, anyhow::Error> {
        self.keystore.get_public_key_by_coin_id(coin_id)
    }

    fn get_address_public_keys(&self) -> Vec<(RoochAddress, PublicKey)> {
        self.keystore.get_address_public_keys()
    }

    fn get_key_pairs(&self, address: &RoochAddress) -> Result<Vec<&RoochKeyPair>, anyhow::Error> {
        self.keystore.get_key_pairs(address)
    }

    fn update_key_pair_by_coin_id(
        &mut self,
        address: &RoochAddress,
        keypair: RoochKeyPair,
        coin_id: CoinID,
    ) -> Result<(), anyhow::Error> {
        self.keystore
            .update_key_pair_by_coin_id(address, keypair, coin_id)
    }

    fn nullify_key_pair_by_coin_id(
        &mut self,
        address: &RoochAddress,
        coin_id: CoinID,
    ) -> Result<(), anyhow::Error> {
        self.keystore.nullify_key_pair_by_coin_id(address, coin_id)
    }

    fn get_key_pair_by_coin_id(
        &self,
        address: &RoochAddress,
        coin_id: CoinID,
    ) -> Result<&RoochKeyPair, signature::Error> {
        self.keystore.get_key_pair_by_coin_id(address, coin_id)
    }

    fn sign_hashed(
        &self,
        address: &RoochAddress,
        msg: &[u8],
        coin_id: CoinID,
    ) -> Result<Signature, signature::Error> {
        self.keystore.sign_hashed(address, msg, coin_id)
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

impl
    AccountKeystore<
        EthereumAddress,
        Secp256k1RecoverablePublicKey,
        Secp256k1RecoverableKeyPair,
        Secp256k1RecoverableSignature,
        EthereumTransaction,
        EthereumTransactionData,
    > for InMemKeystore<EthereumAddress, Secp256k1RecoverableKeyPair>
{
    fn sign_secure<T>(
        &self,
        address: &EthereumAddress,
        msg: &T,
        coin_id: CoinID,
    ) -> Result<Secp256k1RecoverableSignature, signature::Error>
    where
        T: Serialize,
    {
        self.keystore.sign_secure(address, msg, coin_id)
    }

    fn sign_transaction(
        &self,
        address: &EthereumAddress,
        msg: EthereumTransactionData,
        coin_id: CoinID,
    ) -> Result<EthereumTransaction, signature::Error> {
        self.keystore.sign_transaction(address, msg, coin_id)
    }

    fn add_key_pair_by_coin_id(
        &mut self,
        keypair: Secp256k1RecoverableKeyPair,
        coin_id: CoinID,
    ) -> Result<(), anyhow::Error> {
        self.keystore.add_key_pair_by_coin_id(keypair, coin_id)
    }

    fn get_public_key_by_coin_id(
        &self,
        coin_id: CoinID,
    ) -> Result<Secp256k1RecoverablePublicKey, anyhow::Error> {
        self.keystore.get_public_key_by_coin_id(coin_id)
    }

    fn get_address_public_keys(&self) -> Vec<(EthereumAddress, Secp256k1RecoverablePublicKey)> {
        self.keystore.get_address_public_keys()
    }

    fn get_key_pairs(
        &self,
        address: &EthereumAddress,
    ) -> Result<Vec<&Secp256k1RecoverableKeyPair>, anyhow::Error> {
        self.keystore.get_key_pairs(address)
    }

    fn update_key_pair_by_coin_id(
        &mut self,
        address: &EthereumAddress,
        keypair: Secp256k1RecoverableKeyPair,
        coin_id: CoinID,
    ) -> Result<(), anyhow::Error> {
        self.keystore
            .update_key_pair_by_coin_id(address, keypair, coin_id)
    }

    fn nullify_key_pair_by_coin_id(
        &mut self,
        address: &EthereumAddress,
        coin_id: CoinID,
    ) -> Result<(), anyhow::Error> {
        self.keystore.nullify_key_pair_by_coin_id(address, coin_id)
    }

    fn get_key_pair_by_coin_id(
        &self,
        address: &EthereumAddress,
        coin_id: CoinID,
    ) -> Result<&Secp256k1RecoverableKeyPair, signature::Error> {
        self.keystore.get_key_pair_by_coin_id(address, coin_id)
    }

    fn sign_hashed(
        &self,
        address: &EthereumAddress,
        msg: &[u8],
        coin_id: CoinID,
    ) -> Result<Secp256k1RecoverableSignature, signature::Error> {
        self.keystore.sign_hashed(address, msg, coin_id)
    }

    fn generate_session_key(
        &mut self,
        address: &EthereumAddress,
    ) -> Result<AuthenticationKey, anyhow::Error> {
        self.keystore.generate_session_key(address)
    }

    fn sign_transaction_via_session_key(
        &self,
        address: &EthereumAddress,
        msg: EthereumTransactionData,
        authentication_key: &AuthenticationKey,
    ) -> Result<EthereumTransaction, signature::Error> {
        self.keystore
            .sign_transaction_via_session_key(address, msg, authentication_key)
    }
}

impl InMemKeystore<RoochAddress, RoochKeyPair> {
    pub fn new_insecure_for_tests(initial_key_number: usize) -> Self {
        let mut rng = StdRng::from_seed([0; 32]);
        let keys = (0..initial_key_number)
            .map(|_| get_rooch_key_pair_from_rng(&mut rng))
            .map(|(ad, k)| {
                (
                    ad,
                    BTreeMap::from_iter(vec![(CoinID::Rooch, RoochKeyPair::Ed25519(k))]),
                )
            })
            .collect::<BTreeMap<RoochAddress, BTreeMap<CoinID, RoochKeyPair>>>();

        Self {
            keystore: BaseKeyStore::new(keys),
        }
    }
}

impl InMemKeystore<EthereumAddress, Secp256k1RecoverableKeyPair> {
    pub fn new_insecure_for_tests(initial_key_number: usize) -> Self {
        let mut rng = StdRng::from_seed([0; 32]);
        let keys = (0..initial_key_number)
            .map(|_| get_ethereum_key_pair_from_rng(&mut rng))
            .map(|(ad, k)| (ad, BTreeMap::from_iter(vec![(CoinID::Ether, k)])))
            .collect::<BTreeMap<EthereumAddress, BTreeMap<CoinID, Secp256k1RecoverableKeyPair>>>();

        Self {
            keystore: BaseKeyStore::new(keys),
        }
    }
}

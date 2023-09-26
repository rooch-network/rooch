// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::{
    key_derive::{generate_new_key_pair, CoinOperations},
    keypair::KeyPairType,
};
use anyhow::anyhow;
use bip32::DerivationPath;
use bip39::{Language, Mnemonic, Seed};
use enum_dispatch::enum_dispatch;
use fastcrypto::{
    ed25519::Ed25519PrivateKey,
    hash::Keccak256,
    // TODO replace Secp256k1RecoverableKeyPair and Secp256k1RecoverablePublicKey with native ethereum key pair and pub key
    secp256k1::recoverable::{
        Secp256k1RecoverableKeyPair, Secp256k1RecoverablePrivateKey, Secp256k1RecoverablePublicKey,
    },
    traits::{RecoverableSigner, ToFromBytes},
};
use rand::{rngs::StdRng, SeedableRng};
use rooch_types::{
    address::{EthereumAddress, RoochAddress},
    authentication_key::AuthenticationKey,
    crypto::{
        get_ethereum_key_pair_from_rng, get_rooch_key_pair_from_rng, PublicKey, RoochKeyPair,
        Signature,
    },
    transaction::{
        authenticator,
        ethereum::EthereumTransactionData,
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

#[derive(Serialize, Deserialize, Debug)]
#[enum_dispatch(AccountKeystore)]
pub enum Keystore<K: Ord, V> {
    File(FileBasedKeystore<K, V>),
    InMem(InMemKeystore<K, V>),
}

#[enum_dispatch]
pub trait AccountKeystore<Addr: Copy, PubKey, KeyPair, Sig, TransactionData, PrivKey>:
    Send + Sync
{
    type Transaction;

    fn add_key_pair_by_key_pair_type(
        &mut self,
        key_pair: KeyPair,
        key_pair_type: KeyPairType,
    ) -> Result<(), anyhow::Error>;
    fn get_address_public_keys(&self) -> Vec<(Addr, PubKey)>;
    fn get_public_key_by_key_pair_type(
        &self,
        key_pair_type: KeyPairType,
    ) -> Result<PubKey, anyhow::Error>;
    fn get_key_pairs(&self, address: &Addr) -> Result<Vec<&KeyPair>, anyhow::Error>;
    fn get_key_pair_by_key_pair_type(
        &self,
        address: &Addr,
        key_pair_type: KeyPairType,
    ) -> Result<&KeyPair, signature::Error>;
    fn update_key_pair_by_key_pair_type(
        &mut self,
        address: &Addr,
        key_pair: KeyPair,
        key_pair_type: KeyPairType,
    ) -> Result<(), anyhow::Error>;
    fn nullify_key_pair_by_key_pair_type(
        &mut self,
        address: &Addr,
        key_pair_type: KeyPairType,
    ) -> Result<(), anyhow::Error>;

    fn sign_hashed(
        &self,
        address: &Addr,
        msg: &[u8],
        key_pair_type: KeyPairType,
    ) -> Result<Sig, signature::Error>;

    fn sign_transaction(
        &self,
        address: &Addr,
        msg: TransactionData,
        key_pair_type: KeyPairType,
    ) -> Result<Self::Transaction, signature::Error>;

    fn sign_secure<T>(
        &self,
        address: &Addr,
        msg: &T,
        key_pair_type: KeyPairType,
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
        key_pair_type: KeyPairType,
        derivation_path: Option<DerivationPath>,
        word_length: Option<String>,
        password: Option<String>,
    ) -> Result<(Addr, String, KeyPairType, String, Vec<u8>, Vec<u8>, Vec<u8>), anyhow::Error>
    where
        KeyPairType: CoinOperations<Addr, KeyPair, PrivKey>,
    {
        let (address, kp, key_pair_type, hashed_password, nonce, ciphertext, tag, phrase) =
            generate_new_key_pair::<Addr, KeyPair, PrivKey>(
                key_pair_type,
                derivation_path,
                word_length,
                password,
            )?;
        self.add_key_pair_by_key_pair_type(kp, key_pair_type)?;
        Ok((
            address,
            phrase,
            key_pair_type,
            hashed_password,
            nonce,
            ciphertext,
            tag,
        ))
    }

    fn import_from_mnemonic(
        &mut self,
        phrase: &str,
        key_pair_type: KeyPairType,
        derivation_path: Option<DerivationPath>,
        password: Option<String>,
    ) -> Result<(Addr, String, Vec<u8>, Vec<u8>, Vec<u8>), anyhow::Error>
    where
        KeyPairType: CoinOperations<Addr, KeyPair, PrivKey>,
    {
        let mnemonic = Mnemonic::from_phrase(phrase, Language::English)?;
        let seed = Seed::new(&mnemonic, "");

        let sk =
            key_pair_type.derive_private_key_from_path(seed.as_bytes(), derivation_path.clone())?;

        let (nonce, ciphertext, tag) = key_pair_type
            .encrypt_private_key(sk, password.clone())
            .expect("Encryption failed for private key");

        let sk_clone =
            key_pair_type.derive_private_key_from_path(seed.as_bytes(), derivation_path)?;

        let hashed_password = key_pair_type
            .encrypt_password(sk_clone, password)
            .expect("Encryption failed for password");

        let (address, key_pair) =
            key_pair_type.derive_key_pair_from_ciphertext(ciphertext.clone())?;

        self.add_key_pair_by_key_pair_type(key_pair, key_pair_type)?;
        Ok((address, hashed_password, nonce, ciphertext, tag))
    }

    fn update_address_with_key_pair_from_key_pair_type(
        &mut self,
        _address: &Addr,
        phrase: String,
        key_pair_type: KeyPairType,
        derivation_path: Option<DerivationPath>,
        password: Option<String>,
    ) -> Result<(KeyPair, String, Vec<u8>, Vec<u8>, Vec<u8>), anyhow::Error>
    where
        KeyPairType: CoinOperations<Addr, KeyPair, PrivKey>,
    {
        let mnemonic = Mnemonic::from_phrase(&phrase, Language::English)?;
        let seed = Seed::new(&mnemonic, "");

        let sk =
            key_pair_type.derive_private_key_from_path(seed.as_bytes(), derivation_path.clone())?;

        let (nonce, ciphertext, tag) = key_pair_type
            .encrypt_private_key(sk, password.clone())
            .expect("Encryption failed for private key");

        let sk_clone =
            key_pair_type.derive_private_key_from_path(seed.as_bytes(), derivation_path)?;

        let hashed_password = key_pair_type
            .encrypt_password(sk_clone, password)
            .expect("Encryption failed for password");

        let (address, key_pair) =
            key_pair_type.derive_key_pair_from_ciphertext(ciphertext.clone())?;

        self.update_key_pair_by_key_pair_type(&address, key_pair, key_pair_type)?;

        let (_, key_pair_clone) =
            key_pair_type.derive_key_pair_from_ciphertext(ciphertext.clone())?;

        Ok((key_pair_clone, hashed_password, nonce, ciphertext, tag))
    }

    fn nullify_address_with_key_pair_from_key_pair_type(
        &mut self,
        address: &Addr,
        key_pair_type: KeyPairType,
    ) -> Result<(), anyhow::Error> {
        self.nullify_key_pair_by_key_pair_type(address, key_pair_type)?;
        Ok(())
    }

    fn generate_session_key(&mut self, address: &Addr) -> Result<AuthenticationKey, anyhow::Error>;

    fn sign_transaction_via_session_key(
        &self,
        address: &Addr,
        msg: TransactionData,
        authentication_key: &AuthenticationKey,
    ) -> Result<Self::Transaction, signature::Error>;
}

impl
    AccountKeystore<
        RoochAddress,
        PublicKey,
        RoochKeyPair,
        Signature,
        RoochTransactionData,
        Ed25519PrivateKey,
    > for Keystore<RoochAddress, RoochKeyPair>
{
    type Transaction = RoochTransaction;

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

    fn add_key_pair_by_key_pair_type(
        &mut self,
        key_pair: RoochKeyPair,
        key_pair_type: KeyPairType,
    ) -> Result<(), anyhow::Error> {
        // Implement this method to add a key pair to the appropriate variant (File or InMem)
        match self {
            Keystore::File(file_keystore) => {
                file_keystore.add_key_pair_by_key_pair_type(key_pair, key_pair_type)
            }
            Keystore::InMem(inmem_keystore) => {
                inmem_keystore.add_key_pair_by_key_pair_type(key_pair, key_pair_type)
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

    fn get_public_key_by_key_pair_type(
        &self,
        key_pair_type: KeyPairType,
    ) -> Result<PublicKey, anyhow::Error> {
        // Implement this method to get the public key by coin ID from the appropriate variant (File or InMem)
        match self {
            Keystore::File(file_keystore) => {
                file_keystore.get_public_key_by_key_pair_type(key_pair_type)
            }
            Keystore::InMem(inmem_keystore) => {
                inmem_keystore.get_public_key_by_key_pair_type(key_pair_type)
            }
        }
    }

    fn get_key_pairs(&self, address: &RoochAddress) -> Result<Vec<&RoochKeyPair>, anyhow::Error> {
        // Implement this method to get key pairs for the given address from the appropriate variant (File or InMem)
        match self {
            Keystore::File(file_keystore) => file_keystore.get_key_pairs(address),
            Keystore::InMem(inmem_keystore) => inmem_keystore.get_key_pairs(address),
        }
    }

    fn get_key_pair_by_key_pair_type(
        &self,
        address: &RoochAddress,
        key_pair_type: KeyPairType,
    ) -> Result<&RoochKeyPair, signature::Error> {
        // Implement this method to get the key pair by coin ID from the appropriate variant (File or InMem)
        match self {
            Keystore::File(file_keystore) => {
                file_keystore.get_key_pair_by_key_pair_type(address, key_pair_type)
            }
            Keystore::InMem(inmem_keystore) => {
                inmem_keystore.get_key_pair_by_key_pair_type(address, key_pair_type)
            }
        }
    }

    fn update_key_pair_by_key_pair_type(
        &mut self,
        address: &RoochAddress,
        key_pair: RoochKeyPair,
        key_pair_type: KeyPairType,
    ) -> Result<(), anyhow::Error> {
        // Implement this method to update the key pair by coin ID for the appropriate variant (File or InMem)
        match self {
            Keystore::File(file_keystore) => {
                file_keystore.update_key_pair_by_key_pair_type(address, key_pair, key_pair_type)
            }
            Keystore::InMem(inmem_keystore) => {
                inmem_keystore.update_key_pair_by_key_pair_type(address, key_pair, key_pair_type)
            }
        }
    }

    fn nullify_key_pair_by_key_pair_type(
        &mut self,
        address: &RoochAddress,
        key_pair_type: KeyPairType,
    ) -> Result<(), anyhow::Error> {
        // Implement this method to nullify the key pair by coin ID for the appropriate variant (File or InMem)
        match self {
            Keystore::File(file_keystore) => {
                file_keystore.nullify_key_pair_by_key_pair_type(address, key_pair_type)
            }
            Keystore::InMem(inmem_keystore) => {
                inmem_keystore.nullify_key_pair_by_key_pair_type(address, key_pair_type)
            }
        }
    }

    fn sign_hashed(
        &self,
        address: &RoochAddress,
        msg: &[u8],
        key_pair_type: KeyPairType,
    ) -> Result<Signature, signature::Error> {
        // Implement this method to sign a hashed message for the appropriate variant (File or InMem)
        match self {
            Keystore::File(file_keystore) => file_keystore.sign_hashed(address, msg, key_pair_type),
            Keystore::InMem(inmem_keystore) => {
                inmem_keystore.sign_hashed(address, msg, key_pair_type)
            }
        }
    }

    fn sign_transaction(
        &self,
        address: &RoochAddress,
        msg: RoochTransactionData,
        key_pair_type: KeyPairType,
    ) -> Result<RoochTransaction, signature::Error> {
        // Implement this method to sign a transaction for the appropriate variant (File or InMem)
        match self {
            Keystore::File(file_keystore) => {
                file_keystore.sign_transaction(address, msg, key_pair_type)
            }
            Keystore::InMem(inmem_keystore) => {
                inmem_keystore.sign_transaction(address, msg, key_pair_type)
            }
        }
    }

    fn sign_secure<T>(
        &self,
        address: &RoochAddress,
        msg: &T,
        key_pair_type: KeyPairType,
    ) -> Result<Signature, signature::Error>
    where
        T: Serialize,
    {
        // Implement this method to sign a secure message for the appropriate variant (File or InMem)
        match self {
            Keystore::File(file_keystore) => file_keystore.sign_secure(address, msg, key_pair_type),
            Keystore::InMem(inmem_keystore) => {
                inmem_keystore.sign_secure(address, msg, key_pair_type)
            }
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
        ethers::types::Signature,
        EthereumTransactionData,
        Secp256k1RecoverablePrivateKey,
    > for Keystore<EthereumAddress, Secp256k1RecoverableKeyPair>
{
    type Transaction = (EthereumTransactionData, ethers::types::Signature);

    fn sign_transaction_via_session_key(
        &self,
        address: &EthereumAddress,
        msg: EthereumTransactionData,
        authentication_key: &AuthenticationKey,
    ) -> Result<(EthereumTransactionData, ethers::types::Signature), signature::Error> {
        match self {
            Keystore::File(file_keystore) => {
                file_keystore.sign_transaction_via_session_key(address, msg, authentication_key)
            }
            Keystore::InMem(inmem_keystore) => {
                inmem_keystore.sign_transaction_via_session_key(address, msg, authentication_key)
            }
        }
    }

    fn add_key_pair_by_key_pair_type(
        &mut self,
        key_pair: Secp256k1RecoverableKeyPair,
        key_pair_type: KeyPairType,
    ) -> Result<(), anyhow::Error> {
        // Implement this method to add a key pair to the appropriate variant (File or InMem)
        match self {
            Keystore::File(file_keystore) => {
                file_keystore.add_key_pair_by_key_pair_type(key_pair, key_pair_type)
            }
            Keystore::InMem(inmem_keystore) => {
                inmem_keystore.add_key_pair_by_key_pair_type(key_pair, key_pair_type)
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

    fn get_public_key_by_key_pair_type(
        &self,
        key_pair_type: KeyPairType,
    ) -> Result<Secp256k1RecoverablePublicKey, anyhow::Error> {
        // Implement this method to get the public key by coin ID from the appropriate variant (File or InMem)
        match self {
            Keystore::File(file_keystore) => {
                file_keystore.get_public_key_by_key_pair_type(key_pair_type)
            }
            Keystore::InMem(inmem_keystore) => {
                inmem_keystore.get_public_key_by_key_pair_type(key_pair_type)
            }
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

    fn get_key_pair_by_key_pair_type(
        &self,
        address: &EthereumAddress,
        key_pair_type: KeyPairType,
    ) -> Result<&Secp256k1RecoverableKeyPair, signature::Error> {
        // Implement this method to get a key pair by coin ID from the appropriate variant (File or InMem)
        match self {
            Keystore::File(file_keystore) => {
                file_keystore.get_key_pair_by_key_pair_type(address, key_pair_type)
            }
            Keystore::InMem(inmem_keystore) => {
                inmem_keystore.get_key_pair_by_key_pair_type(address, key_pair_type)
            }
        }
    }

    fn update_key_pair_by_key_pair_type(
        &mut self,
        address: &EthereumAddress,
        key_pair: Secp256k1RecoverableKeyPair,
        key_pair_type: KeyPairType,
    ) -> Result<(), anyhow::Error> {
        // Implement this method to update a key pair by coin ID in the appropriate variant (File or InMem)
        match self {
            Keystore::File(file_keystore) => {
                file_keystore.update_key_pair_by_key_pair_type(address, key_pair, key_pair_type)
            }
            Keystore::InMem(inmem_keystore) => {
                inmem_keystore.update_key_pair_by_key_pair_type(address, key_pair, key_pair_type)
            }
        }
    }

    fn nullify_key_pair_by_key_pair_type(
        &mut self,
        address: &EthereumAddress,
        key_pair_type: KeyPairType,
    ) -> Result<(), anyhow::Error> {
        // Implement this method to nullify a key pair by coin ID in the appropriate variant (File or InMem)
        match self {
            Keystore::File(file_keystore) => {
                file_keystore.nullify_key_pair_by_key_pair_type(address, key_pair_type)
            }
            Keystore::InMem(inmem_keystore) => {
                inmem_keystore.nullify_key_pair_by_key_pair_type(address, key_pair_type)
            }
        }
    }

    fn sign_hashed(
        &self,
        address: &EthereumAddress,
        msg: &[u8],
        key_pair_type: KeyPairType,
    ) -> Result<ethers::types::Signature, signature::Error> {
        // Implement this method to sign a hashed message with the key pair for the given address and coin ID
        match self {
            Keystore::File(file_keystore) => file_keystore.sign_hashed(address, msg, key_pair_type),
            Keystore::InMem(inmem_keystore) => {
                inmem_keystore.sign_hashed(address, msg, key_pair_type)
            }
        }
    }

    fn sign_transaction(
        &self,
        address: &EthereumAddress,
        msg: EthereumTransactionData,
        key_pair_type: KeyPairType,
    ) -> Result<(EthereumTransactionData, ethers::types::Signature), signature::Error> {
        // Implement this method to sign a transaction with the key pair for the given address and coin ID
        match self {
            Keystore::File(file_keystore) => {
                file_keystore.sign_transaction(address, msg, key_pair_type)
            }
            Keystore::InMem(inmem_keystore) => {
                inmem_keystore.sign_transaction(address, msg, key_pair_type)
            }
        }
    }

    fn sign_secure<T>(
        &self,
        address: &EthereumAddress,
        msg: &T,
        key_pair_type: KeyPairType,
    ) -> Result<ethers::types::Signature, signature::Error>
    where
        T: Serialize,
    {
        // Implement this method to sign a serializable message with the key pair for the given address and coin ID
        match self {
            Keystore::File(file_keystore) => file_keystore.sign_secure(address, msg, key_pair_type),
            Keystore::InMem(inmem_keystore) => {
                inmem_keystore.sign_secure(address, msg, key_pair_type)
            }
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

#[derive(Default, Debug, Serialize, Deserialize)]
#[serde_as]
pub(crate) struct BaseKeyStore<K, V>
where
    K: Ord,
{
    keys: BTreeMap<K, BTreeMap<KeyPairType, V>>,
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
    pub fn new(keys: BTreeMap<K, BTreeMap<KeyPairType, V>>) -> Self {
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
        RoochTransactionData,
        Ed25519PrivateKey,
    > for BaseKeyStore<RoochAddress, RoochKeyPair>
{
    type Transaction = RoochTransaction;

    fn get_key_pair_by_key_pair_type(
        &self,
        address: &RoochAddress,
        key_pair_type: KeyPairType,
    ) -> Result<&RoochKeyPair, signature::Error> {
        if let Some(inner_map) = self.keys.get(address) {
            if let Some(keypair) = inner_map.get(&key_pair_type) {
                Ok(keypair)
            } else {
                Err(signature::Error::from_source(format!(
                    "KeyPairType not found for address: [{:?}]",
                    key_pair_type
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
        key_pair_type: KeyPairType,
    ) -> Result<Signature, signature::Error> {
        Ok(Signature::new_hashed(
            msg,
            self.get_key_pair_by_key_pair_type(address, key_pair_type)?,
        ))
    }

    fn sign_secure<T>(
        &self,
        address: &RoochAddress,
        msg: &T,
        key_pair_type: KeyPairType,
    ) -> Result<Signature, signature::Error>
    where
        T: Serialize,
    {
        Ok(Signature::new_secure(
            msg,
            self.get_key_pair_by_key_pair_type(address, key_pair_type)?,
        ))
    }

    fn sign_transaction(
        &self,
        address: &RoochAddress,
        msg: RoochTransactionData,
        key_pair_type: KeyPairType,
    ) -> Result<RoochTransaction, signature::Error> {
        let kp = self
            .get_key_pair_by_key_pair_type(address, key_pair_type)
            .ok()
            .ok_or_else(|| {
                signature::Error::from_source(format!("Cannot find key for address: [{address}]"))
            })?;

        let signature = Signature::new_hashed(msg.hash().as_bytes(), kp);

        let auth = authenticator::Authenticator::rooch(signature);

        Ok(RoochTransaction::new(msg, auth))
    }

    fn add_key_pair_by_key_pair_type(
        &mut self,
        key_pair: RoochKeyPair,
        key_pair_type: KeyPairType,
    ) -> Result<(), anyhow::Error> {
        let address: RoochAddress = (&key_pair.public()).into();
        self.keys
            .entry(address)
            .or_insert_with(BTreeMap::new)
            .insert(key_pair_type, key_pair);
        Ok(())
    }

    fn get_public_key_by_key_pair_type(
        &self,
        key_pair_type: KeyPairType,
    ) -> Result<PublicKey, anyhow::Error> {
        for inner_map in self.keys.values() {
            if let Some(keypair) = inner_map.get(&key_pair_type) {
                return Ok(keypair.public());
            }
        }
        Err(anyhow!(
            "Cannot find key for coin id: [{:?}]",
            key_pair_type
        ))
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

    fn update_key_pair_by_key_pair_type(
        &mut self,
        address: &RoochAddress,
        key_pair: RoochKeyPair,
        key_pair_type: KeyPairType,
    ) -> Result<(), anyhow::Error> {
        // First, get the inner map associated with the address
        let inner_map = self.keys.entry(*address).or_insert_with(BTreeMap::new);

        // Insert or update the keypair for the specified coin in the inner map
        inner_map.insert(key_pair_type, key_pair);
        Ok(())
    }

    fn nullify_key_pair_by_key_pair_type(
        &mut self,
        address: &RoochAddress,
        key_pair_type: KeyPairType,
    ) -> Result<(), anyhow::Error> {
        // First, get the inner map associated with the address
        let inner_map = self.keys.entry(*address).or_insert_with(BTreeMap::new);

        // Remove or nullify the keypair for the specified coin in the inner map
        inner_map.remove(&key_pair_type);
        Ok(())
    }

    fn generate_session_key(
        &mut self,
        address: &RoochAddress,
    ) -> Result<AuthenticationKey, anyhow::Error> {
        //TODO define derivation_path for session key
        let (_address, kp, _key_pair_type, _, _, _, _, _phrase) =
            generate_new_key_pair::<RoochAddress, RoochKeyPair, Ed25519PrivateKey>(
                KeyPairType::RoochKeyPairType,
                None,
                None,
                None,
            )?;
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
        ethers::types::Signature,
        EthereumTransactionData,
        Secp256k1RecoverablePrivateKey,
    > for BaseKeyStore<EthereumAddress, Secp256k1RecoverableKeyPair>
{
    type Transaction = (EthereumTransactionData, ethers::types::Signature);

    fn get_key_pair_by_key_pair_type(
        &self,
        address: &EthereumAddress,
        key_pair_type: KeyPairType,
    ) -> Result<&Secp256k1RecoverableKeyPair, signature::Error> {
        if let Some(inner_map) = self.keys.get(address) {
            if let Some(keypair) = inner_map.get(&key_pair_type) {
                Ok(keypair)
            } else {
                Err(signature::Error::from_source(format!(
                    "KeyPairType not found for address: [{:?}]",
                    key_pair_type
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
        key_pair_type: KeyPairType,
    ) -> Result<ethers::types::Signature, signature::Error> {
        let key_pair = self.get_key_pair_by_key_pair_type(address, key_pair_type)?;
        let signature = key_pair.sign_recoverable_with_hash::<Keccak256>(msg);
        let ethereum_signature = ethers::types::Signature::try_from(signature.as_bytes()).unwrap();
        Ok(ethereum_signature)
    }

    fn sign_secure<T>(
        &self,
        address: &EthereumAddress,
        msg: &T,
        key_pair_type: KeyPairType,
    ) -> Result<ethers::types::Signature, signature::Error>
    where
        T: Serialize,
    {
        let key_pair = self.get_key_pair_by_key_pair_type(address, key_pair_type)?;
        // Serialize the message into a byte slice
        let message_bytes = serde_json::to_vec(msg).unwrap();
        let signature = key_pair.sign_recoverable(message_bytes.as_slice());
        let ethereum_signature = ethers::types::Signature::try_from(signature.as_bytes()).unwrap();
        Ok(ethereum_signature)
    }

    fn sign_transaction(
        &self,
        _address: &EthereumAddress,
        msg: EthereumTransactionData,
        _key_pair_type: KeyPairType,
    ) -> Result<(EthereumTransactionData, ethers::types::Signature), signature::Error> {
        let signature = EthereumTransactionData::into_signature(&msg).unwrap();
        Ok((msg, signature))
    }

    fn add_key_pair_by_key_pair_type(
        &mut self,
        key_pair: Secp256k1RecoverableKeyPair,
        key_pair_type: KeyPairType,
    ) -> Result<(), anyhow::Error> {
        let address = EthereumAddress::from(key_pair.public.clone());
        self.keys
            .entry(address)
            .or_insert_with(BTreeMap::new)
            .insert(key_pair_type, key_pair);
        Ok(())
    }

    fn get_public_key_by_key_pair_type(
        &self,
        key_pair_type: KeyPairType,
    ) -> Result<Secp256k1RecoverablePublicKey, anyhow::Error> {
        for inner_map in self.keys.values() {
            if let Some(keypair) = inner_map.get(&key_pair_type) {
                return Ok(keypair.public.clone());
            }
        }
        Err(anyhow!(
            "Cannot find key for coin id: [{:?}]",
            key_pair_type
        ))
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

    fn update_key_pair_by_key_pair_type(
        &mut self,
        address: &EthereumAddress,
        key_pair: Secp256k1RecoverableKeyPair,
        key_pair_type: KeyPairType,
    ) -> Result<(), anyhow::Error> {
        // First, get the inner map associated with the address
        let inner_map = self.keys.entry(*address).or_insert_with(BTreeMap::new);

        // Insert or update the keypair for the specified coin in the inner map
        inner_map.insert(key_pair_type, key_pair);
        Ok(())
    }

    fn nullify_key_pair_by_key_pair_type(
        &mut self,
        address: &EthereumAddress,
        key_pair_type: KeyPairType,
    ) -> Result<(), anyhow::Error> {
        // First, get the inner map associated with the address
        let inner_map = self.keys.entry(*address).or_insert_with(BTreeMap::new);

        // Remove or nullify the keypair for the specified coin in the inner map
        inner_map.remove(&key_pair_type);
        Ok(())
    }

    fn generate_session_key(
        &mut self,
        address: &EthereumAddress,
    ) -> Result<AuthenticationKey, anyhow::Error> {
        //TODO define derivation_path for session key
        let (_, kp, _key_pair_type, _, _, _, _, _phrase) =
            generate_new_key_pair::<
                EthereumAddress,
                Secp256k1RecoverableKeyPair,
                Secp256k1RecoverablePrivateKey,
            >(KeyPairType::EthereumKeyPairType, None, None, None)?;
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
    ) -> Result<(EthereumTransactionData, ethers::types::Signature), signature::Error> {
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
        let ethereum_signature = ethers::types::Signature::try_from(signature.as_bytes()).unwrap();
        Ok((msg, ethereum_signature))
    }
}

#[derive(Default, Serialize, Deserialize, Debug)]
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
        RoochTransactionData,
        Ed25519PrivateKey,
    > for FileBasedKeystore<RoochAddress, RoochKeyPair>
{
    type Transaction = RoochTransaction;

    fn get_key_pair_by_key_pair_type(
        &self,
        address: &RoochAddress,
        key_pair_type: KeyPairType,
    ) -> Result<&RoochKeyPair, signature::Error> {
        self.keystore
            .get_key_pair_by_key_pair_type(address, key_pair_type)
    }

    fn sign_hashed(
        &self,
        address: &RoochAddress,
        msg: &[u8],
        key_pair_type: KeyPairType,
    ) -> Result<Signature, signature::Error> {
        self.keystore.sign_hashed(address, msg, key_pair_type)
    }

    fn sign_secure<T>(
        &self,
        address: &RoochAddress,
        msg: &T,
        key_pair_type: KeyPairType,
    ) -> Result<Signature, signature::Error>
    where
        T: Serialize,
    {
        self.keystore.sign_secure(address, msg, key_pair_type)
    }

    fn sign_transaction(
        &self,
        address: &RoochAddress,
        msg: RoochTransactionData,
        key_pair_type: KeyPairType,
    ) -> Result<RoochTransaction, signature::Error> {
        self.keystore.sign_transaction(address, msg, key_pair_type)
    }

    fn add_key_pair_by_key_pair_type(
        &mut self,
        key_pair: RoochKeyPair,
        key_pair_type: KeyPairType,
    ) -> Result<(), anyhow::Error> {
        self.keystore
            .add_key_pair_by_key_pair_type(key_pair, key_pair_type)?;
        //TODO should check test env at here?
        if std::env::var_os("TEST_ENV").is_none() {
            self.save()?;
        }
        Ok(())
    }

    fn get_public_key_by_key_pair_type(
        &self,
        key_pair_type: KeyPairType,
    ) -> Result<PublicKey, anyhow::Error> {
        self.keystore.get_public_key_by_key_pair_type(key_pair_type)
    }

    fn get_address_public_keys(&self) -> Vec<(RoochAddress, PublicKey)> {
        self.keystore.get_address_public_keys()
    }

    fn get_key_pairs(&self, address: &RoochAddress) -> Result<Vec<&RoochKeyPair>, anyhow::Error> {
        self.keystore.get_key_pairs(address)
    }

    fn update_key_pair_by_key_pair_type(
        &mut self,
        address: &RoochAddress,
        key_pair: RoochKeyPair,
        key_pair_type: KeyPairType,
    ) -> Result<(), anyhow::Error> {
        self.keystore
            .update_key_pair_by_key_pair_type(address, key_pair, key_pair_type)?;
        //TODO should check test env at here?
        if std::env::var_os("TEST_ENV").is_none() {
            self.save()?;
        }
        Ok(())
    }

    fn nullify_key_pair_by_key_pair_type(
        &mut self,
        address: &RoochAddress,
        key_pair_type: KeyPairType,
    ) -> Result<(), anyhow::Error> {
        self.keystore
            .nullify_key_pair_by_key_pair_type(address, key_pair_type)?;
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
        ethers::types::Signature,
        EthereumTransactionData,
        Secp256k1RecoverablePrivateKey,
    > for FileBasedKeystore<EthereumAddress, Secp256k1RecoverableKeyPair>
{
    type Transaction = (EthereumTransactionData, ethers::types::Signature);

    fn get_key_pair_by_key_pair_type(
        &self,
        address: &EthereumAddress,
        key_pair_type: KeyPairType,
    ) -> Result<&Secp256k1RecoverableKeyPair, signature::Error> {
        self.keystore
            .get_key_pair_by_key_pair_type(address, key_pair_type)
    }

    fn sign_hashed(
        &self,
        address: &EthereumAddress,
        msg: &[u8],
        key_pair_type: KeyPairType,
    ) -> Result<ethers::types::Signature, signature::Error> {
        self.keystore.sign_hashed(address, msg, key_pair_type)
    }

    fn sign_secure<T>(
        &self,
        address: &EthereumAddress,
        msg: &T,
        key_pair_type: KeyPairType,
    ) -> Result<ethers::types::Signature, signature::Error>
    where
        T: Serialize,
    {
        self.keystore.sign_secure(address, msg, key_pair_type)
    }

    fn sign_transaction(
        &self,
        address: &EthereumAddress,
        msg: EthereumTransactionData,
        key_pair_type: KeyPairType,
    ) -> Result<(EthereumTransactionData, ethers::types::Signature), signature::Error> {
        self.keystore.sign_transaction(address, msg, key_pair_type)
    }

    fn add_key_pair_by_key_pair_type(
        &mut self,
        key_pair: Secp256k1RecoverableKeyPair,
        key_pair_type: KeyPairType,
    ) -> Result<(), anyhow::Error> {
        self.keystore
            .add_key_pair_by_key_pair_type(key_pair, key_pair_type)?;
        //TODO should check test env at here?
        if std::env::var_os("TEST_ENV").is_none() {
            self.save()?;
        }
        Ok(())
    }

    fn get_public_key_by_key_pair_type(
        &self,
        key_pair_type: KeyPairType,
    ) -> Result<Secp256k1RecoverablePublicKey, anyhow::Error> {
        self.keystore.get_public_key_by_key_pair_type(key_pair_type)
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

    fn update_key_pair_by_key_pair_type(
        &mut self,
        address: &EthereumAddress,
        key_pair: Secp256k1RecoverableKeyPair,
        key_pair_type: KeyPairType,
    ) -> Result<(), anyhow::Error> {
        self.keystore
            .update_key_pair_by_key_pair_type(address, key_pair, key_pair_type)?;
        //TODO should check test env at here?
        if std::env::var_os("TEST_ENV").is_none() {
            self.save()?;
        }
        Ok(())
    }

    fn nullify_key_pair_by_key_pair_type(
        &mut self,
        address: &EthereumAddress,
        key_pair_type: KeyPairType,
    ) -> Result<(), anyhow::Error> {
        self.keystore
            .nullify_key_pair_by_key_pair_type(address, key_pair_type)?;
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
    ) -> Result<(EthereumTransactionData, ethers::types::Signature), signature::Error> {
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

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct InMemKeystore<K: Ord, V> {
    keystore: BaseKeyStore<K, V>,
}

impl
    AccountKeystore<
        RoochAddress,
        PublicKey,
        RoochKeyPair,
        Signature,
        RoochTransactionData,
        Ed25519PrivateKey,
    > for InMemKeystore<RoochAddress, RoochKeyPair>
{
    type Transaction = RoochTransaction;

    fn sign_secure<T>(
        &self,
        address: &RoochAddress,
        msg: &T,
        key_pair_type: KeyPairType,
    ) -> Result<Signature, signature::Error>
    where
        T: Serialize,
    {
        self.keystore.sign_secure(address, msg, key_pair_type)
    }

    fn sign_transaction(
        &self,
        address: &RoochAddress,
        msg: RoochTransactionData,
        key_pair_type: KeyPairType,
    ) -> Result<RoochTransaction, signature::Error> {
        self.keystore.sign_transaction(address, msg, key_pair_type)
    }

    fn add_key_pair_by_key_pair_type(
        &mut self,
        key_pair: RoochKeyPair,
        key_pair_type: KeyPairType,
    ) -> Result<(), anyhow::Error> {
        self.keystore
            .add_key_pair_by_key_pair_type(key_pair, key_pair_type)
    }

    fn get_public_key_by_key_pair_type(
        &self,
        key_pair_type: KeyPairType,
    ) -> Result<PublicKey, anyhow::Error> {
        self.keystore.get_public_key_by_key_pair_type(key_pair_type)
    }

    fn get_address_public_keys(&self) -> Vec<(RoochAddress, PublicKey)> {
        self.keystore.get_address_public_keys()
    }

    fn get_key_pairs(&self, address: &RoochAddress) -> Result<Vec<&RoochKeyPair>, anyhow::Error> {
        self.keystore.get_key_pairs(address)
    }

    fn update_key_pair_by_key_pair_type(
        &mut self,
        address: &RoochAddress,
        key_pair: RoochKeyPair,
        key_pair_type: KeyPairType,
    ) -> Result<(), anyhow::Error> {
        self.keystore
            .update_key_pair_by_key_pair_type(address, key_pair, key_pair_type)
    }

    fn nullify_key_pair_by_key_pair_type(
        &mut self,
        address: &RoochAddress,
        key_pair_type: KeyPairType,
    ) -> Result<(), anyhow::Error> {
        self.keystore
            .nullify_key_pair_by_key_pair_type(address, key_pair_type)
    }

    fn get_key_pair_by_key_pair_type(
        &self,
        address: &RoochAddress,
        key_pair_type: KeyPairType,
    ) -> Result<&RoochKeyPair, signature::Error> {
        self.keystore
            .get_key_pair_by_key_pair_type(address, key_pair_type)
    }

    fn sign_hashed(
        &self,
        address: &RoochAddress,
        msg: &[u8],
        key_pair_type: KeyPairType,
    ) -> Result<Signature, signature::Error> {
        self.keystore.sign_hashed(address, msg, key_pair_type)
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
        ethers::types::Signature,
        EthereumTransactionData,
        Secp256k1RecoverablePrivateKey,
    > for InMemKeystore<EthereumAddress, Secp256k1RecoverableKeyPair>
{
    type Transaction = (EthereumTransactionData, ethers::types::Signature);

    fn sign_secure<T>(
        &self,
        address: &EthereumAddress,
        msg: &T,
        key_pair_type: KeyPairType,
    ) -> Result<ethers::types::Signature, signature::Error>
    where
        T: Serialize,
    {
        self.keystore.sign_secure(address, msg, key_pair_type)
    }

    fn sign_transaction(
        &self,
        address: &EthereumAddress,
        msg: EthereumTransactionData,
        key_pair_type: KeyPairType,
    ) -> Result<(EthereumTransactionData, ethers::types::Signature), signature::Error> {
        self.keystore.sign_transaction(address, msg, key_pair_type)
    }

    fn add_key_pair_by_key_pair_type(
        &mut self,
        key_pair: Secp256k1RecoverableKeyPair,
        key_pair_type: KeyPairType,
    ) -> Result<(), anyhow::Error> {
        self.keystore
            .add_key_pair_by_key_pair_type(key_pair, key_pair_type)
    }

    fn get_public_key_by_key_pair_type(
        &self,
        key_pair_type: KeyPairType,
    ) -> Result<Secp256k1RecoverablePublicKey, anyhow::Error> {
        self.keystore.get_public_key_by_key_pair_type(key_pair_type)
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

    fn update_key_pair_by_key_pair_type(
        &mut self,
        address: &EthereumAddress,
        key_pair: Secp256k1RecoverableKeyPair,
        key_pair_type: KeyPairType,
    ) -> Result<(), anyhow::Error> {
        self.keystore
            .update_key_pair_by_key_pair_type(address, key_pair, key_pair_type)
    }

    fn nullify_key_pair_by_key_pair_type(
        &mut self,
        address: &EthereumAddress,
        key_pair_type: KeyPairType,
    ) -> Result<(), anyhow::Error> {
        self.keystore
            .nullify_key_pair_by_key_pair_type(address, key_pair_type)
    }

    fn get_key_pair_by_key_pair_type(
        &self,
        address: &EthereumAddress,
        key_pair_type: KeyPairType,
    ) -> Result<&Secp256k1RecoverableKeyPair, signature::Error> {
        self.keystore
            .get_key_pair_by_key_pair_type(address, key_pair_type)
    }

    fn sign_hashed(
        &self,
        address: &EthereumAddress,
        msg: &[u8],
        key_pair_type: KeyPairType,
    ) -> Result<ethers::types::Signature, signature::Error> {
        self.keystore.sign_hashed(address, msg, key_pair_type)
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
    ) -> Result<(EthereumTransactionData, ethers::types::Signature), signature::Error> {
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
                    BTreeMap::from_iter(vec![(
                        KeyPairType::RoochKeyPairType,
                        RoochKeyPair::Ed25519(k),
                    )]),
                )
            })
            .collect::<BTreeMap<RoochAddress, BTreeMap<KeyPairType, RoochKeyPair>>>();

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
            .map(|(ad, k)| (ad, BTreeMap::from_iter(vec![(KeyPairType::EthereumKeyPairType, k)])))
            .collect::<BTreeMap<EthereumAddress, BTreeMap<KeyPairType, Secp256k1RecoverableKeyPair>>>();

        Self {
            keystore: BaseKeyStore::new(keys),
        }
    }
}

// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::key_derive::{
    derive_address_from_private_key, derive_private_key_from_path, encrypt_private_key,
    generate_new_key_pair, get_ethereum_key_pair_from_red, get_rooch_key_pair_from_red,
    KeyOperator, KeyStoreOperator,
};
use anyhow::anyhow;
use bip32::DerivationPath;
use bip39::{Language, Mnemonic, Seed};
use enum_dispatch::enum_dispatch;
use fastcrypto::{
    hash::Keccak256,
    // TODO replace Secp256k1RecoverableKeyPair and Secp256k1RecoverablePublicKey with native ethereum key pair and pub key
    secp256k1::recoverable::{Secp256k1RecoverableKeyPair, Secp256k1RecoverablePublicKey},
    traits::{RecoverableSigner, ToFromBytes},
};
use rooch_types::{
    address::{EthereumAddress, RoochAddress},
    authentication_key::AuthenticationKey,
    crypto::{PublicKey, RoochKeyPair, Signature},
    error::RoochError,
    key_struct::{EncryptionData, GeneratedKeyPair},
    keypair_type::KeyPairType,
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

pub struct ImportedMnemonic<Addr> {
    pub address: Addr,
    pub encryption: EncryptionData,
}

pub struct UpdatedAddress<KeyPair> {
    pub key_pair: KeyPair,
    pub encryption: EncryptionData,
}

#[derive(Serialize, Deserialize, Debug)]
#[enum_dispatch(AccountKeystore)]
pub enum Keystore<K: Ord> {
    File(FileBasedKeystore<K>),
    InMem(InMemKeystore<K>),
}

#[enum_dispatch]
pub trait AccountKeystore<Addr: Copy, PubKey, KeyPair, Sig, TransactionData>: Send + Sync {
    type Transaction;

    fn add_encryption_data_by_key_pair_type(
        &mut self,
        address: Addr,
        encryption: EncryptionData,
    ) -> Result<(), anyhow::Error>;
    fn get_address_public_keys(
        &self,
        password: Option<String>,
    ) -> Result<Vec<(Addr, PubKey)>, RoochError>;
    fn get_public_key_by_key_pair_type(
        &self,
        key_pair_type: KeyPairType,
        password: Option<String>,
    ) -> Result<PubKey, anyhow::Error>;
    fn get_key_pairs(
        &self,
        address: &Addr,
        password: Option<String>,
    ) -> Result<Vec<KeyPair>, anyhow::Error>;
    fn get_key_pair_by_type_password(
        &self,
        address: &Addr,
        key_pair_type: KeyPairType,
        password: Option<String>,
    ) -> Result<KeyPair, RoochError>;
    fn update_encryption_data_by_key_pair_type(
        &mut self,
        address: &Addr,
        key_pair_type: KeyPairType,
        encryption: EncryptionData,
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
        password: Option<String>,
    ) -> Result<Sig, RoochError>;

    fn sign_transaction(
        &self,
        address: &Addr,
        msg: TransactionData,
        key_pair_type: KeyPairType,
        password: Option<String>,
    ) -> Result<Self::Transaction, RoochError>;

    fn sign_secure<T>(
        &self,
        address: &Addr,
        msg: &T,
        key_pair_type: KeyPairType,
        password: Option<String>,
    ) -> Result<Sig, RoochError>
    where
        T: Serialize;

    fn addresses(&self) -> Vec<Addr>;

    fn generate_and_add_new_key(
        &mut self,
        derivation_path: Option<DerivationPath>,
        word_length: Option<String>,
        password: Option<String>,
    ) -> Result<GeneratedKeyPair, anyhow::Error> {
        let result = generate_new_key_pair(derivation_path, word_length, password)?;

        self.add_encryption_data_by_key_pair_type(
            result.address,
            result.result.encryption.clone(),
        )?;

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

        let (nonce, ciphertext, tag) = encrypt_private_key(sk.clone(), password.clone())
            .expect("Encryption failed for private key");

        let address = derive_address_from_private_key(sk)?;

        let encryption = EncryptionData {
            nonce,
            ciphertext,
            tag,
        };

        let result = ImportedMnemonic {
            address,
            encryption: encryption.clone(),
        };

        self.add_encryption_data_by_key_pair_type(result.address, encryption)?;

        Ok(result)
    }

    fn update_address_with_key_pair_from_key_pair_type(
        &mut self,
        _address: &Addr,
        phrase: String,
        key_pair_type: KeyPairType,
        derivation_path: Option<DerivationPath>,
        password: Option<String>,
    ) -> Result<UpdatedAddress<KeyPair>, anyhow::Error>
    where
        KeyPairType: KeyStoreOperator<Addr, KeyPair>,
    {
        let mnemonic = Mnemonic::from_phrase(&phrase, Language::English)?;
        let seed = Seed::new(&mnemonic, "");

        let sk =
            key_pair_type.derive_private_key_from_path(seed.as_bytes(), derivation_path.clone())?;

        let (nonce, ciphertext, tag) = key_pair_type
            .encrypt_private_key(sk.clone(), password.clone())
            .expect("Encryption failed for private key");

        let sk_clone =
            key_pair_type.derive_private_key_from_path(seed.as_bytes(), derivation_path)?;

        let address = key_pair_type.derive_address_from_private_key(sk)?;

        let encryption = EncryptionData {
            nonce,
            ciphertext,
            tag,
        };

        let kp = key_pair_type.retrieve_key_pair(&encryption, password)?;

        let result = UpdatedAddress {
            key_pair: kp,
            encryption: encryption.clone(),
        };

        self.update_encryption_data_by_key_pair_type(&address, key_pair_type, encryption)?;

        Ok(result)
    }

    fn nullify_address_with_key_pair_from_key_pair_type(
        &mut self,
        address: &Addr,
        key_pair_type: KeyPairType,
    ) -> Result<(), anyhow::Error> {
        self.nullify_key_pair_by_key_pair_type(address, key_pair_type)?;
        Ok(())
    }

    fn generate_session_key(
        &mut self,
        address: &Addr,
        password: Option<String>,
    ) -> Result<AuthenticationKey, anyhow::Error>;

    fn sign_transaction_via_session_key(
        &self,
        address: &Addr,
        msg: TransactionData,
        authentication_key: &AuthenticationKey,
        password: Option<String>,
    ) -> Result<Self::Transaction, signature::Error>;
}

impl AccountKeystore<RoochAddress, PublicKey, RoochKeyPair, Signature, RoochTransactionData>
    for Keystore<RoochAddress>
{
    type Transaction = RoochTransaction;

    fn sign_transaction_via_session_key(
        &self,
        address: &RoochAddress,
        msg: RoochTransactionData,
        authentication_key: &AuthenticationKey,
        password: Option<String>,
    ) -> Result<RoochTransaction, signature::Error> {
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

    fn add_encryption_data_by_key_pair_type(
        &mut self,
        address: RoochAddress,
        key_pair_type: KeyPairType,
        encryption: EncryptionData,
    ) -> Result<(), anyhow::Error> {
        // Implement this method to add a key pair to the appropriate variant (File or InMem)
        match self {
            Keystore::File(file_keystore) => file_keystore.add_encryption_data_by_key_pair_type(
                address,
                key_pair_type,
                encryption,
            ),
            Keystore::InMem(inmem_keystore) => inmem_keystore.add_encryption_data_by_key_pair_type(
                address,
                key_pair_type,
                encryption,
            ),
        }
    }

    fn get_address_public_keys(
        &self,
        password: Option<String>,
    ) -> Result<Vec<(RoochAddress, PublicKey)>, RoochError> {
        // Implement this method to collect public keys from the appropriate variant (File or InMem)
        match self {
            Keystore::File(file_keystore) => file_keystore.get_address_public_keys(password),
            Keystore::InMem(inmem_keystore) => inmem_keystore.get_address_public_keys(password),
        }
    }

    fn get_public_key_by_key_pair_type(
        &self,
        key_pair_type: KeyPairType,
        password: Option<String>,
    ) -> Result<PublicKey, anyhow::Error> {
        // Implement this method to get the public key by coin ID from the appropriate variant (File or InMem)
        match self {
            Keystore::File(file_keystore) => {
                file_keystore.get_public_key_by_key_pair_type(key_pair_type, password)
            }
            Keystore::InMem(inmem_keystore) => {
                inmem_keystore.get_public_key_by_key_pair_type(key_pair_type, password)
            }
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

    fn get_key_pair_by_type_password(
        &self,
        address: &RoochAddress,
        key_pair_type: KeyPairType,
        password: Option<String>,
    ) -> Result<RoochKeyPair, RoochError> {
        // Implement this method to get the key pair by coin ID from the appropriate variant (File or InMem)
        match self {
            Keystore::File(file_keystore) => {
                file_keystore.get_key_pair_by_type_password(address, key_pair_type, password)
            }
            Keystore::InMem(inmem_keystore) => {
                inmem_keystore.get_key_pair_by_type_password(address, key_pair_type, password)
            }
        }
    }

    fn update_encryption_data_by_key_pair_type(
        &mut self,
        address: &RoochAddress,
        key_pair_type: KeyPairType,
        encryption: EncryptionData,
    ) -> Result<(), anyhow::Error> {
        // Implement this method to update the key pair by coin ID for the appropriate variant (File or InMem)
        match self {
            Keystore::File(file_keystore) => file_keystore.update_encryption_data_by_key_pair_type(
                address,
                key_pair_type,
                encryption,
            ),
            Keystore::InMem(inmem_keystore) => inmem_keystore
                .update_encryption_data_by_key_pair_type(address, key_pair_type, encryption),
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
        password: Option<String>,
    ) -> Result<Signature, RoochError> {
        // Implement this method to sign a hashed message for the appropriate variant (File or InMem)
        match self {
            Keystore::File(file_keystore) => {
                file_keystore.sign_hashed(address, msg, key_pair_type, password)
            }
            Keystore::InMem(inmem_keystore) => {
                inmem_keystore.sign_hashed(address, msg, key_pair_type, password)
            }
        }
    }

    fn sign_transaction(
        &self,
        address: &RoochAddress,
        msg: RoochTransactionData,
        key_pair_type: KeyPairType,
        password: Option<String>,
    ) -> Result<RoochTransaction, RoochError> {
        // Implement this method to sign a transaction for the appropriate variant (File or InMem)
        match self {
            Keystore::File(file_keystore) => {
                file_keystore.sign_transaction(address, msg, key_pair_type, password)
            }
            Keystore::InMem(inmem_keystore) => {
                inmem_keystore.sign_transaction(address, msg, key_pair_type, password)
            }
        }
    }

    fn sign_secure<T>(
        &self,
        address: &RoochAddress,
        msg: &T,
        key_pair_type: KeyPairType,
        password: Option<String>,
    ) -> Result<Signature, RoochError>
    where
        T: Serialize,
    {
        // Implement this method to sign a secure message for the appropriate variant (File or InMem)
        match self {
            Keystore::File(file_keystore) => {
                file_keystore.sign_secure(address, msg, key_pair_type, password)
            }
            Keystore::InMem(inmem_keystore) => {
                inmem_keystore.sign_secure(address, msg, key_pair_type, password)
            }
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
}

impl
    AccountKeystore<
        EthereumAddress,
        Secp256k1RecoverablePublicKey,
        Secp256k1RecoverableKeyPair,
        ethers::types::Signature,
        EthereumTransactionData,
    > for Keystore<EthereumAddress>
{
    type Transaction = (EthereumTransactionData, ethers::types::Signature);

    fn sign_transaction_via_session_key(
        &self,
        address: &EthereumAddress,
        msg: EthereumTransactionData,
        authentication_key: &AuthenticationKey,
        password: Option<String>,
    ) -> Result<(EthereumTransactionData, ethers::types::Signature), signature::Error> {
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

    fn add_encryption_data_by_key_pair_type(
        &mut self,
        address: EthereumAddress,
        key_pair_type: KeyPairType,
        encryption: EncryptionData,
    ) -> Result<(), anyhow::Error> {
        // Implement this method to add a key pair to the appropriate variant (File or InMem)
        match self {
            Keystore::File(file_keystore) => file_keystore.add_encryption_data_by_key_pair_type(
                address,
                key_pair_type,
                encryption,
            ),
            Keystore::InMem(inmem_keystore) => inmem_keystore.add_encryption_data_by_key_pair_type(
                address,
                key_pair_type,
                encryption,
            ),
        }
    }

    fn get_address_public_keys(
        &self,
        password: Option<String>,
    ) -> Result<Vec<(EthereumAddress, Secp256k1RecoverablePublicKey)>, RoochError> {
        // Implement this method to collect public keys from the appropriate variant (File or InMem)
        match self {
            Keystore::File(file_keystore) => file_keystore.get_address_public_keys(password),
            Keystore::InMem(inmem_keystore) => inmem_keystore.get_address_public_keys(password),
        }
    }

    fn get_public_key_by_key_pair_type(
        &self,
        key_pair_type: KeyPairType,
        password: Option<String>,
    ) -> Result<Secp256k1RecoverablePublicKey, anyhow::Error> {
        // Implement this method to get the public key by coin ID from the appropriate variant (File or InMem)
        match self {
            Keystore::File(file_keystore) => {
                file_keystore.get_public_key_by_key_pair_type(key_pair_type, password)
            }
            Keystore::InMem(inmem_keystore) => {
                inmem_keystore.get_public_key_by_key_pair_type(key_pair_type, password)
            }
        }
    }

    fn get_key_pairs(
        &self,
        address: &EthereumAddress,
        password: Option<String>,
    ) -> Result<Vec<Secp256k1RecoverableKeyPair>, anyhow::Error> {
        // Implement this method to get key pairs for the given address from the appropriate variant (File or InMem)
        match self {
            Keystore::File(file_keystore) => file_keystore.get_key_pairs(address, password),
            Keystore::InMem(inmem_keystore) => inmem_keystore.get_key_pairs(address, password),
        }
    }

    fn get_key_pair_by_type_password(
        &self,
        address: &EthereumAddress,
        key_pair_type: KeyPairType,
        password: Option<String>,
    ) -> Result<Secp256k1RecoverableKeyPair, RoochError> {
        // Implement this method to get a key pair by coin ID from the appropriate variant (File or InMem)
        match self {
            Keystore::File(file_keystore) => {
                file_keystore.get_key_pair_by_type_password(address, key_pair_type, password)
            }
            Keystore::InMem(inmem_keystore) => {
                inmem_keystore.get_key_pair_by_type_password(address, key_pair_type, password)
            }
        }
    }

    fn update_encryption_data_by_key_pair_type(
        &mut self,
        address: &EthereumAddress,
        key_pair_type: KeyPairType,
        encryption: EncryptionData,
    ) -> Result<(), anyhow::Error> {
        // Implement this method to update a key pair by coin ID in the appropriate variant (File or InMem)
        match self {
            Keystore::File(file_keystore) => file_keystore.update_encryption_data_by_key_pair_type(
                address,
                key_pair_type,
                encryption,
            ),
            Keystore::InMem(inmem_keystore) => inmem_keystore
                .update_encryption_data_by_key_pair_type(address, key_pair_type, encryption),
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
        password: Option<String>,
    ) -> Result<ethers::types::Signature, RoochError> {
        // Implement this method to sign a hashed message with the key pair for the given address and coin ID
        match self {
            Keystore::File(file_keystore) => {
                file_keystore.sign_hashed(address, msg, key_pair_type, password)
            }
            Keystore::InMem(inmem_keystore) => {
                inmem_keystore.sign_hashed(address, msg, key_pair_type, password)
            }
        }
    }

    fn sign_transaction(
        &self,
        address: &EthereumAddress,
        msg: EthereumTransactionData,
        key_pair_type: KeyPairType,
        password: Option<String>,
    ) -> Result<(EthereumTransactionData, ethers::types::Signature), RoochError> {
        // Implement this method to sign a transaction with the key pair for the given address and coin ID
        match self {
            Keystore::File(file_keystore) => {
                file_keystore.sign_transaction(address, msg, key_pair_type, password)
            }
            Keystore::InMem(inmem_keystore) => {
                inmem_keystore.sign_transaction(address, msg, key_pair_type, password)
            }
        }
    }

    fn sign_secure<T>(
        &self,
        address: &EthereumAddress,
        msg: &T,
        key_pair_type: KeyPairType,
        password: Option<String>,
    ) -> Result<ethers::types::Signature, RoochError>
    where
        T: Serialize,
    {
        // Implement this method to sign a serializable message with the key pair for the given address and coin ID
        match self {
            Keystore::File(file_keystore) => {
                file_keystore.sign_secure(address, msg, key_pair_type, password)
            }
            Keystore::InMem(inmem_keystore) => {
                inmem_keystore.sign_secure(address, msg, key_pair_type, password)
            }
        }
    }

    fn generate_session_key(
        &mut self,
        address: &EthereumAddress,
        password: Option<String>,
    ) -> Result<AuthenticationKey, anyhow::Error> {
        // Implement this method to generate a session key for the given address
        match self {
            Keystore::File(file_keystore) => file_keystore.generate_session_key(address, password),
            Keystore::InMem(inmem_keystore) => {
                inmem_keystore.generate_session_key(address, password)
            }
        }
    }

    fn addresses(&self) -> Vec<EthereumAddress> {
        match self {
            Keystore::File(file_keystore) => file_keystore.addresses(),
            Keystore::InMem(inmem_keystore) => inmem_keystore.addresses(),
        }
    }
}

impl Display for Keystore<RoochAddress> {
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

impl Display for Keystore<EthereumAddress> {
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
pub(crate) struct BaseKeyStore<K>
where
    K: Ord,
{
    keys: BTreeMap<K, EncryptionData>,
    /// RoochAddress -> BTreeMap<AuthenticationKey, RoochKeyPair>
    /// EthereumAddress -> BTreeMap<AuthenticationKey, Secp256k1RecoverableKeyPair>
    #[serde_as(as = "BTreeMap<DisplayFromStr, BTreeMap<DisplayFromStr, _>>")]
    session_keys: BTreeMap<K, BTreeMap<AuthenticationKey, EncryptionData>>,
}

impl<K> BaseKeyStore<K>
where
    K: Serialize + Deserialize<'static> + Ord,
{
    pub fn new(keys: BTreeMap<K, BTreeMap<KeyPairType, EncryptionData>>) -> Self {
        Self {
            keys,
            session_keys: BTreeMap::new(),
        }
    }
}

impl AccountKeystore<RoochAddress, PublicKey, RoochKeyPair, Signature, RoochTransactionData>
    for BaseKeyStore<RoochAddress>
{
    type Transaction = RoochTransaction;

    fn get_key_pair_by_type_password(
        &self,
        address: &RoochAddress,
        key_pair_type: KeyPairType,
        password: Option<String>,
    ) -> Result<RoochKeyPair, RoochError> {
        if let Some(inner_map) = self.keys.get(address) {
            if let Some(encryption) = inner_map.get(&key_pair_type) {
                let keypair: RoochKeyPair =
                    key_pair_type.retrieve_key_pair(encryption, password)?;
                Ok(keypair)
            } else {
                Err(RoochError::KeyConversionError(format!(
                    "KeyPairType not found for address: [{:?}]",
                    key_pair_type
                )))
            }
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
        key_pair_type: KeyPairType,
        password: Option<String>,
    ) -> Result<Signature, RoochError> {
        Ok(Signature::new_hashed(
            msg,
            &self.get_key_pair_by_type_password(address, key_pair_type, password)?,
        ))
    }

    fn sign_secure<T>(
        &self,
        address: &RoochAddress,
        msg: &T,
        key_pair_type: KeyPairType,
        password: Option<String>,
    ) -> Result<Signature, RoochError>
    where
        T: Serialize,
    {
        Ok(Signature::new_secure(
            msg,
            &self.get_key_pair_by_type_password(address, key_pair_type, password)?,
        ))
    }

    fn sign_transaction(
        &self,
        address: &RoochAddress,
        msg: RoochTransactionData,
        key_pair_type: KeyPairType,
        password: Option<String>,
    ) -> Result<RoochTransaction, RoochError> {
        let kp = self
            .get_key_pair_by_type_password(address, key_pair_type, password)
            .ok()
            .ok_or_else(|| {
                RoochError::SignMessageError(format!("Cannot find key for address: [{address}]"))
            })?;

        let signature = Signature::new_hashed(msg.hash().as_bytes(), &kp);

        let auth = authenticator::Authenticator::rooch(signature);

        Ok(RoochTransaction::new(msg, auth))
    }

    fn add_encryption_data_by_key_pair_type(
        &mut self,
        address: RoochAddress,
        key_pair_type: KeyPairType,
        encryption: EncryptionData,
    ) -> Result<(), anyhow::Error> {
        self.keys
            .entry(address)
            .or_insert_with(BTreeMap::new)
            .insert(key_pair_type, encryption);
        Ok(())
    }

    fn get_public_key_by_key_pair_type(
        &self,
        key_pair_type: KeyPairType,
        password: Option<String>,
    ) -> Result<PublicKey, anyhow::Error> {
        for inner_map in self.keys.values() {
            if let Some(encryption) = inner_map.get(&key_pair_type) {
                let keypair: RoochKeyPair =
                    key_pair_type.retrieve_key_pair(encryption, password)?;
                return Ok(keypair.public());
            }
        }
        Err(anyhow!(
            "Cannot find key for coin id: [{:?}]",
            key_pair_type
        ))
    }

    fn get_address_public_keys(
        &self,
        password: Option<String>,
    ) -> Result<Vec<(RoochAddress, PublicKey)>, RoochError> {
        let mut result = Vec::new();
        for (address, inner_map) in &self.keys {
            for key_pair_type in inner_map.keys() {
                for encryption in inner_map.values() {
                    let keypair: RoochKeyPair =
                        key_pair_type.retrieve_key_pair(encryption, password.clone())?;
                    let public_key = keypair.public();
                    result.push((*address, public_key));
                }
            }
        }
        Ok(result)
    }

    fn get_key_pairs(
        &self,
        address: &RoochAddress,
        password: Option<String>,
    ) -> Result<Vec<RoochKeyPair>, anyhow::Error> {
        match self.keys.get(address) {
            Some(key_map) => {
                // Collect references to RoochKeyPair objects from the inner map into a Vec.
                let key_pairs: Vec<RoochKeyPair> = key_map
                    .iter()
                    .map(|(key_pair_type, encryption)| {
                        key_pair_type.retrieve_key_pair(encryption, password.clone())
                    })
                    .collect::<Result<_, _>>()?;

                Ok(key_pairs)
            }
            None => Err(anyhow!("Cannot find key for address: [{address}]")),
        }
    }

    fn update_encryption_data_by_key_pair_type(
        &mut self,
        address: &RoochAddress,
        key_pair_type: KeyPairType,
        encryption: EncryptionData,
    ) -> Result<(), anyhow::Error> {
        // First, get the inner map associated with the address
        let inner_map = self.keys.entry(*address).or_insert_with(BTreeMap::new);

        // Insert or update the keypair for the specified coin in the inner map
        inner_map.insert(key_pair_type, encryption);
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
        password: Option<String>,
    ) -> Result<AuthenticationKey, anyhow::Error> {
        //TODO define derivation_path for session key
        let result = generate_new_key_pair::<RoochAddress, RoochKeyPair>(
            KeyPairType::RoochKeyPairType,
            None,
            None,
            password.clone(),
        )?;
        let kp: RoochKeyPair =
            KeyPairType::RoochKeyPairType.retrieve_key_pair(&result.result.encryption, password)?;
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

        let kp: RoochKeyPair = KeyPairType::RoochKeyPairType
            .retrieve_key_pair(encryption, password)
            .map_err(signature::Error::from_source)?;

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
}

impl
    AccountKeystore<
        EthereumAddress,
        Secp256k1RecoverablePublicKey,
        Secp256k1RecoverableKeyPair,
        ethers::types::Signature,
        EthereumTransactionData,
    > for BaseKeyStore<EthereumAddress>
{
    type Transaction = (EthereumTransactionData, ethers::types::Signature);

    fn get_key_pair_by_type_password(
        &self,
        address: &EthereumAddress,
        key_pair_type: KeyPairType,
        password: Option<String>,
    ) -> Result<Secp256k1RecoverableKeyPair, RoochError> {
        if let Some(inner_map) = self.keys.get(address) {
            if let Some(encryption) = inner_map.get(&key_pair_type) {
                let keypair: Secp256k1RecoverableKeyPair =
                    key_pair_type.retrieve_key_pair(encryption, password)?;
                Ok(keypair)
            } else {
                Err(RoochError::KeyConversionError(format!(
                    "KeyPairType not found for address: [{:?}]",
                    key_pair_type
                )))
            }
        } else {
            Err(RoochError::SignMessageError(format!(
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
        password: Option<String>,
    ) -> Result<ethers::types::Signature, RoochError> {
        let key_pair = self.get_key_pair_by_type_password(address, key_pair_type, password)?;
        let signature = key_pair.sign_recoverable_with_hash::<Keccak256>(msg);
        let ethereum_signature = ethers::types::Signature::try_from(signature.as_bytes()).unwrap();
        Ok(ethereum_signature)
    }

    fn sign_secure<T>(
        &self,
        address: &EthereumAddress,
        msg: &T,
        key_pair_type: KeyPairType,
        password: Option<String>,
    ) -> Result<ethers::types::Signature, RoochError>
    where
        T: Serialize,
    {
        let key_pair = self.get_key_pair_by_type_password(address, key_pair_type, password)?;
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
        _password: Option<String>,
    ) -> Result<(EthereumTransactionData, ethers::types::Signature), RoochError> {
        let signature = EthereumTransactionData::into_signature(&msg).unwrap();
        Ok((msg, signature))
    }

    fn add_encryption_data_by_key_pair_type(
        &mut self,
        address: EthereumAddress,
        key_pair_type: KeyPairType,
        encryption: EncryptionData,
    ) -> Result<(), anyhow::Error> {
        self.keys
            .entry(address)
            .or_insert_with(BTreeMap::new)
            .insert(key_pair_type, encryption);
        Ok(())
    }

    fn get_public_key_by_key_pair_type(
        &self,
        key_pair_type: KeyPairType,
        password: Option<String>,
    ) -> Result<Secp256k1RecoverablePublicKey, anyhow::Error> {
        for inner_map in self.keys.values() {
            if let Some(encryption) = inner_map.get(&key_pair_type) {
                let keypair: Secp256k1RecoverableKeyPair =
                    key_pair_type.retrieve_key_pair(encryption, password)?;
                return Ok(keypair.public);
            }
        }
        Err(anyhow!(
            "Cannot find key for coin id: [{:?}]",
            key_pair_type
        ))
    }

    fn get_address_public_keys(
        &self,
        password: Option<String>,
    ) -> Result<Vec<(EthereumAddress, Secp256k1RecoverablePublicKey)>, RoochError> {
        let mut result = Vec::new();
        for (address, inner_map) in &self.keys {
            for key_pair_type in inner_map.keys() {
                for encryption in inner_map.values() {
                    let keypair: Secp256k1RecoverableKeyPair =
                        key_pair_type.retrieve_key_pair(encryption, password.clone())?;
                    let public_key = keypair.public.clone();
                    result.push((*address, public_key));
                }
            }
        }
        Ok(result)
    }

    fn get_key_pairs(
        &self,
        address: &EthereumAddress,
        password: Option<String>,
    ) -> Result<Vec<Secp256k1RecoverableKeyPair>, anyhow::Error> {
        match self.keys.get(address) {
            Some(key_map) => {
                // Collect references to Secp256k1RecoverableKeyPair objects from the inner map into a Vec.
                let key_pairs: Vec<Secp256k1RecoverableKeyPair> = key_map
                    .iter()
                    .map(|(key_pair_type, encryption)| {
                        key_pair_type.retrieve_key_pair(encryption, password.clone())
                    })
                    .collect::<Result<_, _>>()?;

                Ok(key_pairs)
            }
            None => Err(anyhow!("Cannot find key for address: [{address}]")),
        }
    }

    fn update_encryption_data_by_key_pair_type(
        &mut self,
        address: &EthereumAddress,
        key_pair_type: KeyPairType,
        encryption: EncryptionData,
    ) -> Result<(), anyhow::Error> {
        // First, get the inner map associated with the address
        let inner_map = self.keys.entry(*address).or_insert_with(BTreeMap::new);

        // Insert or update the keypair for the specified coin in the inner map
        inner_map.insert(key_pair_type, encryption);
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
        password: Option<String>,
    ) -> Result<AuthenticationKey, anyhow::Error> {
        //TODO define derivation_path for session key
        let result = generate_new_key_pair::<EthereumAddress, Secp256k1RecoverableKeyPair>(
            KeyPairType::EthereumKeyPairType,
            None,
            None,
            password,
        )?;
        let authentication_key_bytes = address.0.as_bytes().to_vec();
        let authentication_key = AuthenticationKey::new(authentication_key_bytes);
        let inner_map = self
            .session_keys
            .entry(*address)
            .or_insert_with(BTreeMap::new);
        inner_map.insert(authentication_key.clone(), result.result.encryption);
        Ok(authentication_key)
    }

    fn sign_transaction_via_session_key(
        &self,
        address: &EthereumAddress,
        msg: EthereumTransactionData,
        authentication_key: &AuthenticationKey,
        password: Option<String>,
    ) -> Result<(EthereumTransactionData, ethers::types::Signature), signature::Error> {
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

        let kp: Secp256k1RecoverableKeyPair = KeyPairType::EthereumKeyPairType
            .retrieve_key_pair(encryption, password)
            .map_err(signature::Error::from_source)?;

        let signature = kp.sign_recoverable_with_hash::<Keccak256>(msg.0.hash().as_bytes());
        let ethereum_signature = ethers::types::Signature::try_from(signature.as_bytes()).unwrap();
        Ok((msg, ethereum_signature))
    }

    fn addresses(&self) -> Vec<EthereumAddress> {
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
}

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct FileBasedKeystore<K: Ord> {
    keystore: BaseKeyStore<K>,
    path: Option<PathBuf>,
}

impl AccountKeystore<RoochAddress, PublicKey, RoochKeyPair, Signature, RoochTransactionData>
    for FileBasedKeystore<RoochAddress>
{
    type Transaction = RoochTransaction;

    fn get_key_pair_by_type_password(
        &self,
        address: &RoochAddress,
        key_pair_type: KeyPairType,
        password: Option<String>,
    ) -> Result<RoochKeyPair, RoochError> {
        self.keystore
            .get_key_pair_by_type_password(address, key_pair_type, password)
    }

    fn sign_hashed(
        &self,
        address: &RoochAddress,
        msg: &[u8],
        key_pair_type: KeyPairType,
        password: Option<String>,
    ) -> Result<Signature, RoochError> {
        self.keystore
            .sign_hashed(address, msg, key_pair_type, password)
    }

    fn sign_secure<T>(
        &self,
        address: &RoochAddress,
        msg: &T,
        key_pair_type: KeyPairType,
        password: Option<String>,
    ) -> Result<Signature, RoochError>
    where
        T: Serialize,
    {
        self.keystore
            .sign_secure(address, msg, key_pair_type, password)
    }

    fn sign_transaction(
        &self,
        address: &RoochAddress,
        msg: RoochTransactionData,
        key_pair_type: KeyPairType,
        password: Option<String>,
    ) -> Result<RoochTransaction, RoochError> {
        self.keystore
            .sign_transaction(address, msg, key_pair_type, password)
    }

    fn add_encryption_data_by_key_pair_type(
        &mut self,
        address: RoochAddress,
        key_pair_type: KeyPairType,
        encryption: EncryptionData,
    ) -> Result<(), anyhow::Error> {
        self.keystore
            .add_encryption_data_by_key_pair_type(address, key_pair_type, encryption)?;
        //TODO should check test env at here?
        if std::env::var_os("TEST_ENV").is_none() {
            self.save()?;
        }
        Ok(())
    }

    fn get_public_key_by_key_pair_type(
        &self,
        key_pair_type: KeyPairType,
        password: Option<String>,
    ) -> Result<PublicKey, anyhow::Error> {
        self.keystore
            .get_public_key_by_key_pair_type(key_pair_type, password)
    }

    fn get_address_public_keys(
        &self,
        password: Option<String>,
    ) -> Result<Vec<(RoochAddress, PublicKey)>, RoochError> {
        self.keystore.get_address_public_keys(password)
    }

    fn get_key_pairs(
        &self,
        address: &RoochAddress,
        password: Option<String>,
    ) -> Result<Vec<RoochKeyPair>, anyhow::Error> {
        self.keystore.get_key_pairs(address, password)
    }

    fn update_encryption_data_by_key_pair_type(
        &mut self,
        address: &RoochAddress,
        key_pair_type: KeyPairType,
        encryption: EncryptionData,
    ) -> Result<(), anyhow::Error> {
        self.keystore.update_encryption_data_by_key_pair_type(
            address,
            key_pair_type,
            encryption,
        )?;
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
    ) -> Result<RoochTransaction, signature::Error> {
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
}

impl
    AccountKeystore<
        EthereumAddress,
        Secp256k1RecoverablePublicKey,
        Secp256k1RecoverableKeyPair,
        ethers::types::Signature,
        EthereumTransactionData,
    > for FileBasedKeystore<EthereumAddress>
{
    type Transaction = (EthereumTransactionData, ethers::types::Signature);

    fn get_key_pair_by_type_password(
        &self,
        address: &EthereumAddress,
        key_pair_type: KeyPairType,
        password: Option<String>,
    ) -> Result<Secp256k1RecoverableKeyPair, RoochError> {
        self.keystore
            .get_key_pair_by_type_password(address, key_pair_type, password)
    }

    fn sign_hashed(
        &self,
        address: &EthereumAddress,
        msg: &[u8],
        key_pair_type: KeyPairType,
        password: Option<String>,
    ) -> Result<ethers::types::Signature, RoochError> {
        self.keystore
            .sign_hashed(address, msg, key_pair_type, password)
    }

    fn sign_secure<T>(
        &self,
        address: &EthereumAddress,
        msg: &T,
        key_pair_type: KeyPairType,
        password: Option<String>,
    ) -> Result<ethers::types::Signature, RoochError>
    where
        T: Serialize,
    {
        self.keystore
            .sign_secure(address, msg, key_pair_type, password)
    }

    fn sign_transaction(
        &self,
        address: &EthereumAddress,
        msg: EthereumTransactionData,
        key_pair_type: KeyPairType,
        password: Option<String>,
    ) -> Result<(EthereumTransactionData, ethers::types::Signature), RoochError> {
        self.keystore
            .sign_transaction(address, msg, key_pair_type, password)
    }

    fn add_encryption_data_by_key_pair_type(
        &mut self,
        address: EthereumAddress,
        key_pair_type: KeyPairType,
        encryption: EncryptionData,
    ) -> Result<(), anyhow::Error> {
        self.keystore
            .add_encryption_data_by_key_pair_type(address, key_pair_type, encryption)?;
        //TODO should check test env at here?
        if std::env::var_os("TEST_ENV").is_none() {
            self.save()?;
        }
        Ok(())
    }

    fn get_public_key_by_key_pair_type(
        &self,
        key_pair_type: KeyPairType,
        password: Option<String>,
    ) -> Result<Secp256k1RecoverablePublicKey, anyhow::Error> {
        self.keystore
            .get_public_key_by_key_pair_type(key_pair_type, password)
    }

    fn get_address_public_keys(
        &self,
        password: Option<String>,
    ) -> Result<Vec<(EthereumAddress, Secp256k1RecoverablePublicKey)>, RoochError> {
        self.keystore.get_address_public_keys(password)
    }

    fn get_key_pairs(
        &self,
        address: &EthereumAddress,
        password: Option<String>,
    ) -> Result<Vec<Secp256k1RecoverableKeyPair>, anyhow::Error> {
        self.keystore.get_key_pairs(address, password)
    }

    fn update_encryption_data_by_key_pair_type(
        &mut self,
        address: &EthereumAddress,
        key_pair_type: KeyPairType,
        encryption: EncryptionData,
    ) -> Result<(), anyhow::Error> {
        self.keystore.update_encryption_data_by_key_pair_type(
            address,
            key_pair_type,
            encryption,
        )?;
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
        password: Option<String>,
    ) -> Result<AuthenticationKey, anyhow::Error> {
        let auth_key = self.keystore.generate_session_key(address, password)?;
        self.save()?;
        Ok(auth_key)
    }

    fn sign_transaction_via_session_key(
        &self,
        address: &EthereumAddress,
        msg: EthereumTransactionData,
        authentication_key: &AuthenticationKey,
        password: Option<String>,
    ) -> Result<(EthereumTransactionData, ethers::types::Signature), signature::Error> {
        self.keystore
            .sign_transaction_via_session_key(address, msg, authentication_key, password)
    }

    fn addresses(&self) -> Vec<EthereumAddress> {
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
}

impl FileBasedKeystore<RoochAddress> {
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
            .flat_map(|key_map| {
                // Iterate over key-value pairs (KeyPairType, EncryptionData)
                key_map.iter().flat_map(|(key_pair_type, encryption)| {
                    // Transform EncryptionData into RoochKeyPair using your conversion function.
                    Some(key_pair_type.retrieve_key_pair(encryption, password.clone()))
                })
            })
            .collect::<Result<_, _>>()?;

        Ok(key_pairs)
    }
}

impl FileBasedKeystore<EthereumAddress> {
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

    pub fn key_pairs(
        &self,
        _address: &RoochAddress,
        password: Option<String>,
    ) -> Result<Vec<Secp256k1RecoverableKeyPair>, anyhow::Error> {
        // Collect references to Secp256k1RecoverableKeyPair objects from all inner maps.
        let key_pairs: Vec<Secp256k1RecoverableKeyPair> = self
            .keystore
            .keys
            .values() // Get inner maps
            .flat_map(|key_map| {
                // Iterate over key-value pairs (KeyPairType, EncryptionData)
                key_map.iter().flat_map(|(key_pair_type, encryption)| {
                    // Transform EncryptionData into Secp256k1RecoverableKeyPair using your conversion function.
                    Some(key_pair_type.retrieve_key_pair(encryption, password.clone()))
                })
            })
            .collect::<Result<_, _>>()?;

        Ok(key_pairs)
    }
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct InMemKeystore<K: Ord> {
    keystore: BaseKeyStore<K>,
}

impl AccountKeystore<RoochAddress, PublicKey, RoochKeyPair, Signature, RoochTransactionData>
    for InMemKeystore<RoochAddress>
{
    type Transaction = RoochTransaction;

    fn sign_secure<T>(
        &self,
        address: &RoochAddress,
        msg: &T,
        key_pair_type: KeyPairType,
        password: Option<String>,
    ) -> Result<Signature, RoochError>
    where
        T: Serialize,
    {
        self.keystore
            .sign_secure(address, msg, key_pair_type, password)
    }

    fn sign_transaction(
        &self,
        address: &RoochAddress,
        msg: RoochTransactionData,
        key_pair_type: KeyPairType,
        password: Option<String>,
    ) -> Result<RoochTransaction, RoochError> {
        self.keystore
            .sign_transaction(address, msg, key_pair_type, password)
    }

    fn add_encryption_data_by_key_pair_type(
        &mut self,
        address: RoochAddress,
        key_pair_type: KeyPairType,
        encryption: EncryptionData,
    ) -> Result<(), anyhow::Error> {
        self.keystore
            .add_encryption_data_by_key_pair_type(address, key_pair_type, encryption)
    }

    fn get_public_key_by_key_pair_type(
        &self,
        key_pair_type: KeyPairType,
        password: Option<String>,
    ) -> Result<PublicKey, anyhow::Error> {
        self.keystore
            .get_public_key_by_key_pair_type(key_pair_type, password)
    }

    fn get_address_public_keys(
        &self,
        password: Option<String>,
    ) -> Result<Vec<(RoochAddress, PublicKey)>, RoochError> {
        self.keystore.get_address_public_keys(password)
    }

    fn get_key_pairs(
        &self,
        address: &RoochAddress,
        password: Option<String>,
    ) -> Result<Vec<RoochKeyPair>, anyhow::Error> {
        self.keystore.get_key_pairs(address, password)
    }

    fn update_encryption_data_by_key_pair_type(
        &mut self,
        address: &RoochAddress,
        key_pair_type: KeyPairType,
        encryption: EncryptionData,
    ) -> Result<(), anyhow::Error> {
        self.keystore
            .update_encryption_data_by_key_pair_type(address, key_pair_type, encryption)
    }

    fn nullify_key_pair_by_key_pair_type(
        &mut self,
        address: &RoochAddress,
        key_pair_type: KeyPairType,
    ) -> Result<(), anyhow::Error> {
        self.keystore
            .nullify_key_pair_by_key_pair_type(address, key_pair_type)
    }

    fn get_key_pair_by_type_password(
        &self,
        address: &RoochAddress,
        key_pair_type: KeyPairType,
        password: Option<String>,
    ) -> Result<RoochKeyPair, RoochError> {
        self.keystore
            .get_key_pair_by_type_password(address, key_pair_type, password)
    }

    fn sign_hashed(
        &self,
        address: &RoochAddress,
        msg: &[u8],
        key_pair_type: KeyPairType,
        password: Option<String>,
    ) -> Result<Signature, RoochError> {
        self.keystore
            .sign_hashed(address, msg, key_pair_type, password)
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
}

impl
    AccountKeystore<
        EthereumAddress,
        Secp256k1RecoverablePublicKey,
        Secp256k1RecoverableKeyPair,
        ethers::types::Signature,
        EthereumTransactionData,
    > for InMemKeystore<EthereumAddress>
{
    type Transaction = (EthereumTransactionData, ethers::types::Signature);

    fn sign_secure<T>(
        &self,
        address: &EthereumAddress,
        msg: &T,
        key_pair_type: KeyPairType,
        password: Option<String>,
    ) -> Result<ethers::types::Signature, RoochError>
    where
        T: Serialize,
    {
        self.keystore
            .sign_secure(address, msg, key_pair_type, password)
    }

    fn sign_transaction(
        &self,
        address: &EthereumAddress,
        msg: EthereumTransactionData,
        key_pair_type: KeyPairType,
        password: Option<String>,
    ) -> Result<(EthereumTransactionData, ethers::types::Signature), RoochError> {
        self.keystore
            .sign_transaction(address, msg, key_pair_type, password)
    }

    fn add_encryption_data_by_key_pair_type(
        &mut self,
        address: EthereumAddress,
        key_pair_type: KeyPairType,
        encryption: EncryptionData,
    ) -> Result<(), anyhow::Error> {
        self.keystore
            .add_encryption_data_by_key_pair_type(address, key_pair_type, encryption)
    }

    fn get_public_key_by_key_pair_type(
        &self,
        key_pair_type: KeyPairType,
        password: Option<String>,
    ) -> Result<Secp256k1RecoverablePublicKey, anyhow::Error> {
        self.keystore
            .get_public_key_by_key_pair_type(key_pair_type, password)
    }

    fn get_address_public_keys(
        &self,
        password: Option<String>,
    ) -> Result<Vec<(EthereumAddress, Secp256k1RecoverablePublicKey)>, RoochError> {
        self.keystore.get_address_public_keys(password)
    }

    fn get_key_pairs(
        &self,
        address: &EthereumAddress,
        password: Option<String>,
    ) -> Result<Vec<Secp256k1RecoverableKeyPair>, anyhow::Error> {
        self.keystore.get_key_pairs(address, password)
    }

    fn update_encryption_data_by_key_pair_type(
        &mut self,
        address: &EthereumAddress,
        key_pair_type: KeyPairType,
        encryption: EncryptionData,
    ) -> Result<(), anyhow::Error> {
        self.keystore
            .update_encryption_data_by_key_pair_type(address, key_pair_type, encryption)
    }

    fn nullify_key_pair_by_key_pair_type(
        &mut self,
        address: &EthereumAddress,
        key_pair_type: KeyPairType,
    ) -> Result<(), anyhow::Error> {
        self.keystore
            .nullify_key_pair_by_key_pair_type(address, key_pair_type)
    }

    fn get_key_pair_by_type_password(
        &self,
        address: &EthereumAddress,
        key_pair_type: KeyPairType,
        password: Option<String>,
    ) -> Result<Secp256k1RecoverableKeyPair, RoochError> {
        self.keystore
            .get_key_pair_by_type_password(address, key_pair_type, password)
    }

    fn sign_hashed(
        &self,
        address: &EthereumAddress,
        msg: &[u8],
        key_pair_type: KeyPairType,
        password: Option<String>,
    ) -> Result<ethers::types::Signature, RoochError> {
        self.keystore
            .sign_hashed(address, msg, key_pair_type, password)
    }

    fn generate_session_key(
        &mut self,
        address: &EthereumAddress,
        password: Option<String>,
    ) -> Result<AuthenticationKey, anyhow::Error> {
        self.keystore.generate_session_key(address, password)
    }

    fn sign_transaction_via_session_key(
        &self,
        address: &EthereumAddress,
        msg: EthereumTransactionData,
        authentication_key: &AuthenticationKey,
        password: Option<String>,
    ) -> Result<(EthereumTransactionData, ethers::types::Signature), signature::Error> {
        self.keystore
            .sign_transaction_via_session_key(address, msg, authentication_key, password)
    }

    fn addresses(&self) -> Vec<EthereumAddress> {
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
}

impl InMemKeystore<RoochAddress> {
    pub fn new_insecure_for_tests(initial_key_number: usize) -> Self {
        let keys = (0..initial_key_number)
            .map(|_| get_rooch_key_pair_from_red())
            .map(|(addr, data)| {
                (
                    addr,
                    BTreeMap::from_iter(vec![(KeyPairType::RoochKeyPairType, data)]),
                )
            })
            .collect::<BTreeMap<RoochAddress, BTreeMap<KeyPairType, EncryptionData>>>();

        Self {
            keystore: BaseKeyStore::new(keys),
        }
    }
}
impl InMemKeystore<EthereumAddress> {
    pub fn new_insecure_for_tests(initial_key_number: usize) -> Self {
        let keys = (0..initial_key_number)
            .map(|_| get_ethereum_key_pair_from_red())
            .map(|(addr, data)| {
                (
                    addr,
                    BTreeMap::from_iter(vec![(KeyPairType::EthereumKeyPairType, data)]),
                )
            })
            .collect::<BTreeMap<EthereumAddress, BTreeMap<KeyPairType, EncryptionData>>>();

        Self {
            keystore: BaseKeyStore::new(keys),
        }
    }
}

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
    crypto::{
        get_key_pair_from_rng, BuiltinScheme, EncodeDecodeBase64, PublicKey, RoochKeyPair,
        Signature,
    },
    transaction::{
        authenticator,
        rooch::{RoochTransaction, RoochTransactionData},
    },
};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
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
    fn add_key(&mut self, keypair: RoochKeyPair) -> Result<(), anyhow::Error>;
    fn keys(&self) -> Vec<PublicKey>;
    fn get_key(&self, address: &RoochAddress) -> Result<&RoochKeyPair, anyhow::Error>;

    fn sign_hashed(
        &self,
        address: &RoochAddress,
        msg: &[u8],
    ) -> Result<Signature, signature::Error>;

    fn sign_transaction(
        &self,
        address: &RoochAddress,
        msg: RoochTransactionData,
    ) -> Result<RoochTransaction, signature::Error>;

    fn sign_secure<T>(
        &self,
        address: &RoochAddress,
        msg: &T,
    ) -> Result<Signature, signature::Error>
    where
        T: Serialize;

    fn addresses(&self) -> Vec<RoochAddress> {
        self.keys().iter().map(|k| k.into()).collect()
    }

    fn generate_and_add_new_key(
        &mut self,
        key_scheme: BuiltinScheme,
        derivation_path: Option<DerivationPath>,
        word_length: Option<String>,
    ) -> Result<(RoochAddress, String, BuiltinScheme), anyhow::Error> {
        let (address, kp, scheme, phrase) =
            generate_new_key(key_scheme, derivation_path, word_length)?;
        self.add_key(kp)?;
        Ok((address, phrase, scheme))
    }

    fn import_from_mnemonic(
        &mut self,
        phrase: &str,
        key_scheme: BuiltinScheme,
        derivation_path: Option<DerivationPath>,
    ) -> Result<RoochAddress, anyhow::Error> {
        let mnemonic = Mnemonic::from_phrase(phrase, Language::English)
            .map_err(|e| anyhow::anyhow!("Invalid mnemonic phrase: {:?}", e))?;
        let seed = Seed::new(&mnemonic, "");
        match derive_key_pair_from_path(seed.as_bytes(), derivation_path, &key_scheme) {
            Ok((address, kp)) => {
                self.add_key(kp)?;
                Ok(address)
            }
            Err(e) => Err(anyhow!("error getting keypair {:?}", e)),
        }
    }
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

#[derive(Default)]
pub struct FileBasedKeystore {
    keys: BTreeMap<RoochAddress, RoochKeyPair>,
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
    fn sign_hashed(
        &self,
        address: &RoochAddress,
        msg: &[u8],
    ) -> Result<Signature, signature::Error> {
        Ok(Signature::new_hashed(
            msg,
            self.keys.get(address).ok_or_else(|| {
                signature::Error::from_source(format!("Cannot find key for address: [{address}]"))
            })?,
        ))
    }

    fn sign_secure<T>(&self, address: &RoochAddress, msg: &T) -> Result<Signature, signature::Error>
    where
        T: Serialize,
    {
        Ok(Signature::new_secure(
            msg,
            self.keys.get(address).ok_or_else(|| {
                signature::Error::from_source(format!("Cannot find key for address: [{address}]"))
            })?,
        ))
    }

    fn sign_transaction(
        &self,
        address: &RoochAddress,
        msg: RoochTransactionData,
    ) -> Result<RoochTransaction, signature::Error> {
        let pk = self.get_key(address).ok().ok_or_else(|| {
            signature::Error::from_source(format!("Cannot find key for address: [{address}]"))
        })?;

        let signature = Signature::new_hashed(msg.hash().as_bytes(), pk);

        let auth = match pk.public().scheme() {
            BuiltinScheme::Ed25519 => authenticator::Authenticator::ed25519(signature),
            BuiltinScheme::Ecdsa => todo!(),
            BuiltinScheme::MultiEd25519 => todo!(),
            BuiltinScheme::Schnorr => authenticator::Authenticator::schnorr(signature),
        };

        Ok(RoochTransaction::new(msg, auth))
    }

    fn add_key(&mut self, keypair: RoochKeyPair) -> Result<(), anyhow::Error> {
        match std::env::var_os("TEST_ENV") {
            Some(_) => {}
            None => {
                let address: RoochAddress = (&keypair.public()).into();
                self.keys.insert(address, keypair);
                self.save()?;
            }
        }
        Ok(())
    }

    fn keys(&self) -> Vec<PublicKey> {
        self.keys.values().map(|key| key.public()).collect()
    }

    fn get_key(&self, address: &RoochAddress) -> Result<&RoochKeyPair, anyhow::Error> {
        match self.keys.get(address) {
            Some(key) => Ok(key),
            None => Err(anyhow!("Cannot find key for address: [{address}]")),
        }
    }
}

impl FileBasedKeystore {
    pub fn new(path: &PathBuf) -> Result<Self, anyhow::Error> {
        let keys = if path.exists() {
            let reader = BufReader::new(
                File::open(path)
                    .map_err(|e| anyhow!("Can't open FileBasedKeystore from {:?}: {e}", path))?,
            );
            let kp_strings: Vec<String> = serde_json::from_reader(reader)
                .map_err(|e| anyhow!("Can't deserialize FileBasedKeystore from {:?}: {e}", path))?;
            kp_strings
                .iter()
                .map(|kpstr| {
                    let key = RoochKeyPair::decode_base64(kpstr);
                    key.map(|k| (Into::<RoochAddress>::into(&k.public()), k))
                })
                .collect::<Result<BTreeMap<_, _>, _>>()
                .map_err(|e| anyhow::anyhow!("Invalid Keypair file {:#?} {:?}", e, path))?
        } else {
            BTreeMap::new()
        };

        Ok(Self {
            keys,
            path: Some(path.to_path_buf()),
        })
    }

    pub fn set_path(&mut self, path: &Path) {
        self.path = Some(path.to_path_buf());
    }

    pub fn save(&self) -> Result<(), anyhow::Error> {
        if let Some(path) = &self.path {
            let store = serde_json::to_string_pretty(
                &self
                    .keys
                    .values()
                    .map(EncodeDecodeBase64::encode_base64)
                    .collect::<Vec<_>>(),
            )
            .unwrap();
            fs::write(path, store)?
        }
        Ok(())
    }

    pub fn key_pairs(&self) -> Vec<&RoochKeyPair> {
        self.keys.values().collect()
    }
}

#[derive(Default, Serialize, Deserialize)]
pub struct InMemKeystore {
    keys: BTreeMap<RoochAddress, RoochKeyPair>,
}

impl AccountKeystore for InMemKeystore {
    fn sign_hashed(
        &self,
        address: &RoochAddress,
        msg: &[u8],
    ) -> Result<Signature, signature::Error> {
        Ok(Signature::new_hashed(
            msg,
            self.keys.get(address).ok_or_else(|| {
                signature::Error::from_source(format!("Cannot find key for address: [{address}]"))
            })?,
        ))
    }

    fn sign_transaction(
        &self,
        address: &RoochAddress,
        msg: RoochTransactionData,
    ) -> Result<RoochTransaction, signature::Error> {
        let pk = self.get_key(address).ok().ok_or_else(|| {
            signature::Error::from_source(format!("Cannot find key for address: [{address}]"))
        })?;

        let signature = Signature::new_hashed(msg.hash().as_bytes(), pk);

        let auth = match pk.public().scheme() {
            BuiltinScheme::Ed25519 => authenticator::Authenticator::ed25519(signature),
            BuiltinScheme::Ecdsa => todo!(),
            BuiltinScheme::MultiEd25519 => todo!(),
            BuiltinScheme::Schnorr => authenticator::Authenticator::schnorr(signature),
        };

        Ok(RoochTransaction::new(msg, auth))
    }

    fn sign_secure<T>(&self, address: &RoochAddress, msg: &T) -> Result<Signature, signature::Error>
    where
        T: Serialize,
    {
        Ok(Signature::new_secure(
            msg,
            self.keys.get(address).ok_or_else(|| {
                signature::Error::from_source(format!("Cannot find key for address: [{address}]"))
            })?,
        ))
    }

    fn add_key(&mut self, keypair: RoochKeyPair) -> Result<(), anyhow::Error> {
        let address: RoochAddress = (&keypair.public()).into();
        self.keys.insert(address, keypair);
        Ok(())
    }

    fn keys(&self) -> Vec<PublicKey> {
        self.keys.values().map(|key| key.public()).collect()
    }

    fn get_key(&self, address: &RoochAddress) -> Result<&RoochKeyPair, anyhow::Error> {
        match self.keys.get(address) {
            Some(key) => Ok(key),
            None => Err(anyhow!("Cannot find key for address: [{address}]")),
        }
    }
}

impl InMemKeystore {
    pub fn new_insecure_for_tests(initial_key_number: usize) -> Self {
        let mut rng = StdRng::from_seed([0; 32]);
        let keys = (0..initial_key_number)
            .map(|_| get_key_pair_from_rng(&mut rng))
            .map(|(ad, k)| (ad, RoochKeyPair::Ed25519(k)))
            .collect::<BTreeMap<RoochAddress, RoochKeyPair>>();

        Self { keys }
    }
}

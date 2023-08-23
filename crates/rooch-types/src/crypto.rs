// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::{
    address::RoochAddress,
    authentication_key::AuthenticationKey,
    error::{RoochError, RoochResult},
    framework::{
        bitcoin_validator::BitcoinValidatorModule, ethereum_validator::EthereumValidatorModule,
        native_validator::NativeValidatorModule, nostr_validator::NostrValidatorModule,
    },
};
use clap::ArgEnum;
use derive_more::{AsMut, AsRef, From};
pub use enum_dispatch::enum_dispatch;
use eyre::eyre;
use fastcrypto::encoding::{Base64, Encoding};
use fastcrypto::error::FastCryptoError;
use fastcrypto::hash::{Blake2b256, HashFunction};
pub use fastcrypto::traits::KeyPair as KeypairTraits;
pub use fastcrypto::traits::Signer;
pub use fastcrypto::traits::{
    AggregateAuthenticator, Authenticator, EncodeDecodeBase64, RecoverableSignature,
    RecoverableSigner, SigningKey, ToFromBytes, VerifyingKey,
};
use fastcrypto::{
    ed25519::{
        Ed25519KeyPair, Ed25519PublicKey, Ed25519PublicKeyAsBytes, Ed25519Signature,
        Ed25519SignatureAsBytes,
    },
    secp256k1::{
        recoverable::{
            Secp256k1RecoverableKeyPair, Secp256k1RecoverablePublicKey,
            Secp256k1RecoverablePublicKeyAsBytes, Secp256k1RecoverableSignature,
            Secp256k1RecoverableSignatureAsBytes,
        },
        schnorr::{
            SchnorrKeyPair, SchnorrPublicKey, SchnorrPublicKeyAsBytes, SchnorrSignature,
            SchnorrSignatureAsBytes,
        },
        // TODO wrap it to ecdsa
        Secp256k1KeyPair,
        Secp256k1PublicKey,
        Secp256k1PublicKeyAsBytes,
        Secp256k1Signature,
        Secp256k1SignatureAsBytes,
    },
};
use moveos_types::{h256::H256, serde::Readable, transaction::MoveAction};
use rand::{rngs::StdRng, SeedableRng};
use schemars::JsonSchema;
use serde::ser::Serializer;
use serde::{Deserialize, Deserializer, Serialize};
use serde_with::{serde_as, Bytes};
use std::{hash::Hash, str::FromStr};
use strum_macros::{Display, EnumString};

pub type DefaultHash = Blake2b256;

/// A `Authenticator` is an an abstraction of a account authenticator.
/// It is a part of `AccountAbstraction`

/// The Authenticator scheme which builtin Rooch
#[derive(
    Copy,
    Clone,
    Debug,
    EnumString,
    PartialEq,
    Eq,
    ArgEnum,
    Display,
    Ord,
    PartialOrd,
    Serialize,
    Deserialize,
)]
#[strum(serialize_all = "lowercase")]
pub enum BuiltinScheme {
    Ed25519,
    MultiEd25519,
    Ecdsa,
    EcdsaRecoverable,
    Schnorr,
}

impl BuiltinScheme {
    const ED25519_FLAG: u8 = 0x00;
    const MULTIED25519_FLAG: u8 = 0x01;
    const ECDSA_FLAG: u8 = 0x02;
    const ECDSARECOVERABLE_FLAG: u8 = 0x03;
    const SCHNORR_FLAG: u8 = 0x04;

    pub fn flag(&self) -> u8 {
        match self {
            BuiltinScheme::Ed25519 => Self::ED25519_FLAG,
            BuiltinScheme::MultiEd25519 => Self::MULTIED25519_FLAG,
            BuiltinScheme::Ecdsa => Self::ECDSA_FLAG,
            BuiltinScheme::EcdsaRecoverable => Self::ECDSARECOVERABLE_FLAG,
            BuiltinScheme::Schnorr => Self::SCHNORR_FLAG,
        }
    }

    pub fn from_flag(flag: &str) -> Result<BuiltinScheme, RoochError> {
        let byte_int = flag
            .parse::<u8>()
            .map_err(|_| RoochError::KeyConversionError("Invalid key scheme".to_owned()))?;
        Self::from_flag_byte(byte_int)
    }

    pub fn from_flag_byte(byte_int: u8) -> Result<BuiltinScheme, RoochError> {
        match byte_int {
            Self::ED25519_FLAG => Ok(BuiltinScheme::Ed25519),
            Self::MULTIED25519_FLAG => Ok(BuiltinScheme::MultiEd25519),
            Self::ECDSA_FLAG => Ok(BuiltinScheme::Ecdsa),
            Self::ECDSARECOVERABLE_FLAG => Ok(BuiltinScheme::EcdsaRecoverable),
            Self::SCHNORR_FLAG => Ok(BuiltinScheme::Schnorr),
            _ => Err(RoochError::KeyConversionError(
                "Invalid key scheme".to_owned(),
            )),
        }
    }

    pub fn create_rotate_authentication_key_action(
        &self,
        public_key: Vec<u8>,
        decimal_prefix_or_version: Option<u8>,
    ) -> Result<MoveAction, RoochError> {
        let action = match self {
            BuiltinScheme::Ed25519 => {
                NativeValidatorModule::rotate_authentication_key_action(public_key)
            }
            BuiltinScheme::MultiEd25519 => todo!(),
            BuiltinScheme::Ecdsa => {
                let decimal_prefix_or_version = decimal_prefix_or_version.ok_or_else(|| RoochError::RotateAuthenticationKeyError("Error decoding the decimal prefix or the script version. Use -t or --address-type to indicate an address to use for Bitcoin under the ecdsa scheme.".to_owned()))?;
                BitcoinValidatorModule::rotate_authentication_key_action(
                    public_key,
                    decimal_prefix_or_version,
                )
            }
            BuiltinScheme::EcdsaRecoverable => {
                EthereumValidatorModule::rotate_authentication_key_action(public_key)
            }
            BuiltinScheme::Schnorr => {
                NostrValidatorModule::rotate_authentication_key_action(public_key)
            }
        };
        Ok(action)
    }

    pub fn create_remove_authentication_key_action(&self) -> Result<MoveAction, RoochError> {
        let action = match self {
            BuiltinScheme::Ed25519 => NativeValidatorModule::remove_authentication_key_action(),
            BuiltinScheme::MultiEd25519 => todo!(),
            BuiltinScheme::Ecdsa => BitcoinValidatorModule::remove_authentication_key_action(),
            BuiltinScheme::EcdsaRecoverable => {
                EthereumValidatorModule::remove_authentication_key_action()
            }
            BuiltinScheme::Schnorr => NostrValidatorModule::remove_authentication_key_action(),
        };
        Ok(action)
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, From, PartialEq, Eq)]
pub enum RoochKeyPair {
    Ed25519(Ed25519KeyPair),
    Ecdsa(Secp256k1KeyPair),
    EcdsaRecoverable(Secp256k1RecoverableKeyPair),
    Schnorr(SchnorrKeyPair),
}

impl RoochKeyPair {
    pub fn public(&self) -> PublicKey {
        match self {
            RoochKeyPair::Ed25519(kp) => PublicKey::Ed25519(kp.public().into()),
            RoochKeyPair::Ecdsa(kp) => PublicKey::Ecdsa(kp.public().into()),
            RoochKeyPair::EcdsaRecoverable(kp) => PublicKey::EcdsaRecoverable(kp.public().into()),
            RoochKeyPair::Schnorr(kp) => PublicKey::Schnorr(kp.public().into()),
        }
    }

    pub fn authentication_key(&self) -> AuthenticationKey {
        self.public().authentication_key()
    }
}

impl Signer<Signature> for RoochKeyPair {
    fn sign(&self, msg: &[u8]) -> Signature {
        match self {
            RoochKeyPair::Ed25519(kp) => kp.sign(msg),
            RoochKeyPair::Ecdsa(kp) => kp.sign(msg),
            RoochKeyPair::EcdsaRecoverable(kp) => kp.sign(msg),
            RoochKeyPair::Schnorr(kp) => kp.sign(msg),
        }
    }
}

impl FromStr for RoochKeyPair {
    type Err = eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let kp = Self::decode_base64(s).map_err(|e| eyre!("{}", e.to_string()))?;
        Ok(kp)
    }
}

impl EncodeDecodeBase64 for RoochKeyPair {
    /// Encode a RoochKeyPair as `flag || privkey` in Base64. Note that the pubkey is not encoded.
    fn encode_base64(&self) -> String {
        let mut bytes: Vec<u8> = Vec::new();
        bytes.push(self.public().flag());

        match self {
            RoochKeyPair::Ed25519(kp) => {
                bytes.extend_from_slice(kp.as_bytes());
            }
            RoochKeyPair::Ecdsa(kp) => {
                bytes.extend_from_slice(kp.as_bytes());
            }
            RoochKeyPair::EcdsaRecoverable(kp) => {
                bytes.extend_from_slice(kp.as_bytes());
            }
            RoochKeyPair::Schnorr(kp) => {
                bytes.extend_from_slice(kp.as_bytes());
            }
        }
        Base64::encode(&bytes[..])
    }

    /// Decode a RoochKeyPair from `flag || privkey` in Base64. The public key is computed directly from the private key bytes.
    fn decode_base64(value: &str) -> Result<Self, eyre::Report> {
        let bytes = Base64::decode(value).map_err(|e| eyre!("{}", e.to_string()))?;
        match BuiltinScheme::from_flag_byte(*bytes.first().ok_or_else(|| eyre!("Invalid length"))?)
        {
            Ok(x) => match x {
                BuiltinScheme::Ed25519 => Ok(RoochKeyPair::Ed25519(Ed25519KeyPair::from_bytes(
                    bytes.get(1..).ok_or_else(|| eyre!("Invalid length"))?,
                )?)),
                BuiltinScheme::Ecdsa => Ok(RoochKeyPair::Ecdsa(Secp256k1KeyPair::from_bytes(
                    bytes.get(1..).ok_or_else(|| eyre!("Invalid length"))?,
                )?)),
                BuiltinScheme::EcdsaRecoverable => Ok(RoochKeyPair::EcdsaRecoverable(
                    Secp256k1RecoverableKeyPair::from_bytes(
                        bytes.get(1..).ok_or_else(|| eyre!("Invalid length"))?,
                    )?,
                )),
                BuiltinScheme::Schnorr => Ok(RoochKeyPair::Schnorr(SchnorrKeyPair::from_bytes(
                    bytes.get(1..).ok_or_else(|| eyre!("Invalid length"))?,
                )?)),
                _ => Err(eyre!("Invalid flag byte")),
            },
            _ => Err(eyre!("Invalid bytes")),
        }
    }
}

impl Serialize for RoochKeyPair {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = self.encode_base64();
        serializer.serialize_str(&s)
    }
}

impl<'de> Deserialize<'de> for RoochKeyPair {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error;
        let s = String::deserialize(deserializer)?;
        <RoochKeyPair as EncodeDecodeBase64>::decode_base64(&s)
            .map_err(|e| Error::custom(e.to_string()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, JsonSchema)]
pub enum PublicKey {
    Ed25519(Ed25519PublicKeyAsBytes),
    Ecdsa(Secp256k1PublicKeyAsBytes),
    EcdsaRecoverable(Secp256k1RecoverablePublicKeyAsBytes),
    Schnorr(SchnorrPublicKeyAsBytes),
}

impl AsRef<[u8]> for PublicKey {
    fn as_ref(&self) -> &[u8] {
        match self {
            PublicKey::Ed25519(pk) => &pk.0,
            PublicKey::Ecdsa(pk) => &pk.0,
            PublicKey::EcdsaRecoverable(pk) => &pk.0,
            PublicKey::Schnorr(pk) => &pk.0,
        }
    }
}

impl EncodeDecodeBase64 for PublicKey {
    fn encode_base64(&self) -> String {
        let mut bytes: Vec<u8> = Vec::new();
        bytes.extend_from_slice(&[self.flag()]);
        bytes.extend_from_slice(self.as_ref());
        Base64::encode(&bytes[..])
    }

    fn decode_base64(value: &str) -> Result<Self, eyre::Report> {
        let bytes = Base64::decode(value).map_err(|e| eyre!("{}", e.to_string()))?;
        match bytes.first() {
            Some(x) => {
                if x == &BuiltinScheme::Ed25519.flag() {
                    let pk: Ed25519PublicKey = Ed25519PublicKey::from_bytes(
                        bytes.get(1..).ok_or_else(|| eyre!("Invalid length"))?,
                    )?;
                    Ok(PublicKey::Ed25519((&pk).into()))
                } else if x == &BuiltinScheme::Ecdsa.flag() {
                    let pk = Secp256k1PublicKey::from_bytes(
                        bytes.get(1..).ok_or_else(|| eyre!("Invalid length"))?,
                    )?;
                    Ok(PublicKey::Ecdsa((&pk).into()))
                } else if x == &BuiltinScheme::EcdsaRecoverable.flag() {
                    let pk = Secp256k1RecoverablePublicKey::from_bytes(
                        bytes.get(1..).ok_or_else(|| eyre!("Invalid length"))?,
                    )?;
                    Ok(PublicKey::Ecdsa((&pk).into()))
                } else if x == &BuiltinScheme::Schnorr.flag() {
                    let pk: SchnorrPublicKey = SchnorrPublicKey::from_bytes(
                        bytes.get(1..).ok_or_else(|| eyre!("Invalid length"))?,
                    )?;
                    Ok(PublicKey::Schnorr((&pk).into()))
                } else {
                    Err(eyre!("Invalid flag byte"))
                }
            }
            _ => Err(eyre!("Invalid bytes")),
        }
    }
}

impl Serialize for PublicKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = self.encode_base64();
        serializer.serialize_str(&s)
    }
}

impl<'de> Deserialize<'de> for PublicKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error;
        let s = String::deserialize(deserializer)?;
        <PublicKey as EncodeDecodeBase64>::decode_base64(&s)
            .map_err(|e| Error::custom(e.to_string()))
    }
}

impl PublicKey {
    pub fn flag(&self) -> u8 {
        match self {
            PublicKey::Ed25519(_) => Ed25519RoochSignature::SCHEME.flag(),
            PublicKey::Ecdsa(_) => EcdsaRoochSignature::SCHEME.flag(),
            PublicKey::EcdsaRecoverable(_) => EcdsaRecoverableRoochSignature::SCHEME.flag(),
            PublicKey::Schnorr(_) => SchnorrRoochSignature::SCHEME.flag(),
        }
    }
    pub fn try_from_bytes(
        scheme: BuiltinScheme,
        key_bytes: &[u8],
    ) -> Result<PublicKey, eyre::Report> {
        match scheme {
            BuiltinScheme::Ed25519 => Ok(PublicKey::Ed25519(
                (&Ed25519PublicKey::from_bytes(key_bytes)?).into(),
            )),
            BuiltinScheme::Ecdsa => Ok(PublicKey::Ecdsa(
                (&Secp256k1PublicKey::from_bytes(key_bytes)?).into(),
            )),
            BuiltinScheme::EcdsaRecoverable => Ok(PublicKey::EcdsaRecoverable(
                (&Secp256k1RecoverablePublicKey::from_bytes(key_bytes)?).into(),
            )),
            BuiltinScheme::Schnorr => Ok(PublicKey::Schnorr(
                (&SchnorrPublicKey::from_bytes(key_bytes)?).into(),
            )),
            _ => Err(eyre!("Unsupported scheme")),
        }
    }
    pub fn scheme(&self) -> BuiltinScheme {
        match self {
            PublicKey::Ed25519(_) => Ed25519RoochSignature::SCHEME,
            PublicKey::Ecdsa(_) => EcdsaRoochSignature::SCHEME,
            PublicKey::EcdsaRecoverable(_) => EcdsaRecoverableRoochSignature::SCHEME,
            PublicKey::Schnorr(_) => SchnorrRoochSignature::SCHEME,
        }
    }

    pub fn authentication_key(&self) -> AuthenticationKey {
        self.into()
    }

    pub fn address(&self) -> RoochAddress {
        self.into()
    }
}

pub trait RoochPublicKey: VerifyingKey {
    const SIGNATURE_SCHEME: BuiltinScheme;
}

impl RoochPublicKey for Ed25519PublicKey {
    const SIGNATURE_SCHEME: BuiltinScheme = BuiltinScheme::Ed25519;
}

impl RoochPublicKey for Secp256k1PublicKey {
    const SIGNATURE_SCHEME: BuiltinScheme = BuiltinScheme::Ecdsa;
}

impl RoochPublicKey for Secp256k1RecoverablePublicKey {
    const SIGNATURE_SCHEME: BuiltinScheme = BuiltinScheme::EcdsaRecoverable;
}

impl RoochPublicKey for SchnorrPublicKey {
    const SIGNATURE_SCHEME: BuiltinScheme = BuiltinScheme::Schnorr;
}

impl<T: RoochPublicKey> From<&T> for RoochAddress {
    fn from(pk: &T) -> Self {
        let mut hasher = DefaultHash::default();
        hasher.update([T::SIGNATURE_SCHEME.flag()]);
        hasher.update(pk);
        let g_arr = hasher.finalize();
        RoochAddress(H256(g_arr.digest))
    }
}

/// The address is the hash of the public key
impl From<&PublicKey> for RoochAddress {
    fn from(pk: &PublicKey) -> Self {
        let mut hasher = DefaultHash::default();
        hasher.update([pk.flag()]);
        hasher.update(pk);
        let g_arr = hasher.finalize();
        RoochAddress(H256(g_arr.digest))
    }
}

///The authentication key is the hash of the public key
/// The address and authentication key are the same for now
impl From<&PublicKey> for AuthenticationKey {
    fn from(pk: &PublicKey) -> Self {
        let mut hasher = DefaultHash::default();
        hasher.update([pk.flag()]);
        hasher.update(pk);
        let g_arr = hasher.finalize();
        AuthenticationKey::new(g_arr.digest.to_vec())
    }
}

//
// Account Signatures
//
// This struct exists due to the limitations of the `enum_dispatch` library.
//
pub trait RoochSignatureInner: Sized + ToFromBytes + PartialEq + Eq + Hash {
    type Sig: Authenticator<PubKey = Self::PubKey>;
    type PubKey: VerifyingKey<Sig = Self::Sig> + RoochPublicKey;
    type KeyPair: KeypairTraits<PubKey = Self::PubKey, Sig = Self::Sig>;

    const LENGTH: usize = Self::Sig::LENGTH + Self::PubKey::LENGTH + 1;
    const SCHEME: BuiltinScheme = Self::PubKey::SIGNATURE_SCHEME;

    fn get_verification_inputs(
        &self,
        author: RoochAddress,
    ) -> RoochResult<(Self::Sig, Self::PubKey)> {
        // Is this signature emitted by the expected author?
        let bytes = self.public_key_bytes();
        let pk = Self::PubKey::from_bytes(bytes)
            .map_err(|_| RoochError::KeyConversionError("Invalid public key".to_owned()))?;

        let received_addr = RoochAddress::from(&pk);
        if received_addr != author {
            return Err(RoochError::IncorrectSigner {
                error: format!("Signature get_verification_inputs() failure. Author is {author}, received address is {received_addr}")
            });
        }

        // deserialize the signature
        let signature = Self::Sig::from_bytes(self.signature_bytes()).map_err(|_| {
            RoochError::InvalidSignature {
                error: "Fail to get pubkey and sig".to_owned(),
            }
        })?;

        Ok((signature, pk))
    }

    fn new(kp: &Self::KeyPair, message: &[u8]) -> Self {
        let sig = Signer::sign(kp, message);

        let mut signature_bytes: Vec<u8> = Vec::new();
        signature_bytes
            .extend_from_slice(&[<Self::PubKey as RoochPublicKey>::SIGNATURE_SCHEME.flag()]);

        signature_bytes.extend_from_slice(sig.as_ref());
        signature_bytes.extend_from_slice(kp.public().as_ref());
        Self::from_bytes(&signature_bytes[..])
            .expect("Serialized signature did not have expected size")
    }
}

// Enums for signature scheme signatures
#[enum_dispatch]
#[derive(Clone, JsonSchema, Debug, PartialEq, Eq, Hash)]
pub enum Signature {
    Ed25519RoochSignature,
    EcdsaRoochSignature,
    EcdsaRecoverableRoochSignature,
    SchnorrRoochSignature,
}

impl Serialize for Signature {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let bytes = self.as_ref();

        if serializer.is_human_readable() {
            let s = Base64::encode(bytes);
            serializer.serialize_str(&s)
        } else {
            serializer.serialize_bytes(bytes)
        }
    }
}

impl<'de> Deserialize<'de> for Signature {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error;

        let bytes = if deserializer.is_human_readable() {
            let s = String::deserialize(deserializer)?;
            Base64::decode(&s).map_err(|e| Error::custom(e.to_string()))?
        } else {
            let data: Vec<u8> = Vec::deserialize(deserializer)?;
            data
        };

        Self::from_bytes(&bytes).map_err(|e| Error::custom(e.to_string()))
    }
}

impl Signature {
    /// The messaged passed in is already hashed form.
    pub fn new_hashed(hashed_msg: &[u8], secret: &dyn Signer<Signature>) -> Self {
        Signer::sign(secret, hashed_msg)
    }

    pub fn new_secure<T>(value: &T, secret: &dyn Signer<Signature>) -> Self
    where
        T: Serialize,
    {
        let mut hasher = DefaultHash::default();
        hasher.update(&bcs::to_bytes(&value).expect("Message serialization should not fail"));
        Signer::sign(secret, &hasher.finalize().digest)
    }

    /// Parse [enum CompressedSignature] from trait Signature `flag || sig || pk`.
    /// This is useful for the MultiSig to combine partial signature into a MultiSig public key.
    pub fn to_compressed(&self) -> Result<CompressedSignature, RoochError> {
        let bytes = self.signature_bytes();
        match self.scheme() {
            BuiltinScheme::Ed25519 => Ok(CompressedSignature::Ed25519(
                (&Ed25519Signature::from_bytes(bytes).map_err(|_| {
                    RoochError::InvalidSignature {
                        error: "Cannot parse sig".to_owned(),
                    }
                })?)
                    .into(),
            )),
            BuiltinScheme::Ecdsa => Ok(CompressedSignature::Ecdsa(
                (&Secp256k1Signature::from_bytes(bytes).map_err(|_| {
                    RoochError::InvalidSignature {
                        error: "Cannot parse sig".to_owned(),
                    }
                })?)
                    .into(),
            )),
            BuiltinScheme::EcdsaRecoverable => Ok(CompressedSignature::EcdsaRecoverable(
                (&Secp256k1RecoverableSignature::from_bytes(bytes).map_err(|_| {
                    RoochError::InvalidSignature {
                        error: "Cannot parse sig".to_owned(),
                    }
                })?)
                    .into(),
            )),
            BuiltinScheme::Schnorr => Ok(CompressedSignature::Schnorr(
                (&SchnorrSignature::from_bytes(bytes).map_err(|_| {
                    RoochError::InvalidSignature {
                        error: "Cannot parse sig".to_owned(),
                    }
                })?)
                    .into(),
            )),
            _ => Err(RoochError::UnsupportedFeatureError {
                error: "Unsupported signature scheme in MultiSig".to_owned(),
            }),
        }
    }

    /// Parse [struct PublicKey] from trait RoochSignature `flag || sig || pk`.
    /// This is useful for the MultiSig to construct the bitmap in [struct MultiPublicKey].
    pub fn to_public_key(&self) -> Result<PublicKey, RoochError> {
        let bytes = self.public_key_bytes();
        match self.scheme() {
            BuiltinScheme::Ed25519 => Ok(PublicKey::Ed25519(
                (&Ed25519PublicKey::from_bytes(bytes)
                    .map_err(|_| RoochError::KeyConversionError("Cannot parse pk".to_owned()))?)
                    .into(),
            )),
            BuiltinScheme::Ecdsa => Ok(PublicKey::Ecdsa(
                (&Secp256k1PublicKey::from_bytes(bytes)
                    .map_err(|_| RoochError::KeyConversionError("Cannot parse pk".to_owned()))?)
                    .into(),
            )),
            BuiltinScheme::EcdsaRecoverable => Ok(PublicKey::EcdsaRecoverable(
                (&Secp256k1RecoverablePublicKey::from_bytes(bytes)
                    .map_err(|_| RoochError::KeyConversionError("Cannot parse pk".to_owned()))?)
                    .into(),
            )),
            BuiltinScheme::Schnorr => Ok(PublicKey::Schnorr(
                (&SchnorrPublicKey::from_bytes(bytes)
                    .map_err(|_| RoochError::KeyConversionError("Cannot parse pk".to_owned()))?)
                    .into(),
            )),
            _ => Err(RoochError::UnsupportedFeatureError {
                error: "Unsupported signature scheme in MultiSig".to_owned(),
            }),
        }
    }
}

impl AsRef<[u8]> for Signature {
    fn as_ref(&self) -> &[u8] {
        match self {
            Signature::Ed25519RoochSignature(sig) => sig.as_ref(),
            Signature::EcdsaRoochSignature(sig) => sig.as_ref(),
            Signature::EcdsaRecoverableRoochSignature(sig) => sig.as_ref(),
            Signature::SchnorrRoochSignature(sig) => sig.as_ref(),
        }
    }
}
impl AsMut<[u8]> for Signature {
    fn as_mut(&mut self) -> &mut [u8] {
        match self {
            Signature::Ed25519RoochSignature(sig) => sig.as_mut(),
            Signature::EcdsaRoochSignature(sig) => sig.as_mut(),
            Signature::EcdsaRecoverableRoochSignature(sig) => sig.as_mut(),
            Signature::SchnorrRoochSignature(sig) => sig.as_mut(),
        }
    }
}

impl ToFromBytes for Signature {
    fn from_bytes(bytes: &[u8]) -> Result<Self, FastCryptoError> {
        match bytes.first() {
            Some(x) => {
                if x == &Ed25519RoochSignature::SCHEME.flag() {
                    Ok(<Ed25519RoochSignature as ToFromBytes>::from_bytes(bytes)?.into())
                } else if x == &EcdsaRoochSignature::SCHEME.flag() {
                    Ok(<EcdsaRoochSignature as ToFromBytes>::from_bytes(bytes)?.into())
                } else if x == &EcdsaRecoverableRoochSignature::SCHEME.flag() {
                    Ok(<EcdsaRecoverableRoochSignature as ToFromBytes>::from_bytes(bytes)?.into())
                } else if x == &SchnorrRoochSignature::SCHEME.flag() {
                    Ok(<SchnorrRoochSignature as ToFromBytes>::from_bytes(bytes)?.into())
                } else {
                    Err(FastCryptoError::InvalidInput)
                }
            }
            _ => Err(FastCryptoError::InvalidInput),
        }
    }
}

/// Unlike [enum Signature], [enum CompressedSignature] does not contain public key.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
pub enum CompressedSignature {
    Ed25519(Ed25519SignatureAsBytes),
    Ecdsa(Secp256k1SignatureAsBytes),
    EcdsaRecoverable(Secp256k1RecoverableSignatureAsBytes),
    Schnorr(SchnorrSignatureAsBytes),
}

impl AsRef<[u8]> for CompressedSignature {
    fn as_ref(&self) -> &[u8] {
        match self {
            CompressedSignature::Ed25519(sig) => &sig.0,
            CompressedSignature::Ecdsa(sig) => &sig.0,
            CompressedSignature::EcdsaRecoverable(sig) => &sig.0,
            CompressedSignature::Schnorr(sig) => &sig.0,
        }
    }
}

#[enum_dispatch(Signature)]
pub trait RoochSignature: Sized + ToFromBytes {
    fn signature_bytes(&self) -> &[u8];
    fn public_key_bytes(&self) -> &[u8];
    fn scheme(&self) -> BuiltinScheme;

    fn verify_secure<T>(&self, value: &T, author: RoochAddress) -> RoochResult<()>
    where
        T: Serialize;
}

impl<S: RoochSignatureInner + Sized> RoochSignature for S {
    fn signature_bytes(&self) -> &[u8] {
        // Access array slice is safe because the array bytes is initialized as
        // flag || signature || pubkey with its defined length.
        &self.as_ref()[1..1 + S::Sig::LENGTH]
    }

    fn public_key_bytes(&self) -> &[u8] {
        // Access array slice is safe because the array bytes is initialized as
        // flag || signature || pubkey with its defined length.
        &self.as_ref()[S::Sig::LENGTH + 1..]
    }

    fn scheme(&self) -> BuiltinScheme {
        S::PubKey::SIGNATURE_SCHEME
    }

    fn verify_secure<T>(&self, value: &T, author: RoochAddress) -> Result<(), RoochError>
    where
        T: Serialize,
    {
        let mut hasher = DefaultHash::default();
        hasher.update(&bcs::to_bytes(&value).expect("Message serialization should not fail"));
        let digest = hasher.finalize().digest;

        let (sig, pk) = &self.get_verification_inputs(author)?;
        pk.verify(&digest, sig)
            .map_err(|e| RoochError::InvalidSignature {
                error: format!("Fail to verify user sig {}", e),
            })
    }
}

//
// Ed25519 Rooch Signature port
//
#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash, AsRef, AsMut)]
#[as_ref(forward)]
#[as_mut(forward)]
pub struct Ed25519RoochSignature(
    #[schemars(with = "Base64")]
    #[serde_as(as = "Readable<Base64, Bytes>")]
    [u8; Ed25519PublicKey::LENGTH + Ed25519Signature::LENGTH + 1],
);

// Implementation useful for simplify testing when mock signature is needed
impl Default for Ed25519RoochSignature {
    fn default() -> Self {
        Self([0; Ed25519PublicKey::LENGTH + Ed25519Signature::LENGTH + 1])
    }
}

impl ToFromBytes for Ed25519RoochSignature {
    fn from_bytes(bytes: &[u8]) -> Result<Self, FastCryptoError> {
        if bytes.len() != Self::LENGTH {
            return Err(FastCryptoError::InputLengthWrong(Self::LENGTH));
        }
        let mut sig_bytes = [0; Self::LENGTH];
        sig_bytes.copy_from_slice(bytes);
        Ok(Self(sig_bytes))
    }
}

impl Signer<Signature> for Ed25519KeyPair {
    fn sign(&self, msg: &[u8]) -> Signature {
        Ed25519RoochSignature::new(self, msg).into()
    }
}

impl RoochSignatureInner for Ed25519RoochSignature {
    type Sig = Ed25519Signature;
    type PubKey = Ed25519PublicKey;
    type KeyPair = Ed25519KeyPair;
    const LENGTH: usize = Ed25519PublicKey::LENGTH + Ed25519Signature::LENGTH + 1;
}

//
// Ecdsa Rooch Signature port
//
#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash, AsRef, AsMut)]
#[as_ref(forward)]
#[as_mut(forward)]
pub struct EcdsaRoochSignature(
    #[schemars(with = "Base64")]
    #[serde_as(as = "Readable<Base64, Bytes>")]
    [u8; Secp256k1PublicKey::LENGTH + Secp256k1Signature::LENGTH + 1],
);

impl RoochSignatureInner for EcdsaRoochSignature {
    type Sig = Secp256k1Signature;
    type PubKey = Secp256k1PublicKey;
    type KeyPair = Secp256k1KeyPair;
    const LENGTH: usize = Secp256k1PublicKey::LENGTH + Secp256k1Signature::LENGTH + 1;
}

impl ToFromBytes for EcdsaRoochSignature {
    fn from_bytes(bytes: &[u8]) -> Result<Self, FastCryptoError> {
        if bytes.len() != Self::LENGTH {
            return Err(FastCryptoError::InputLengthWrong(Self::LENGTH));
        }
        let mut sig_bytes = [0; Self::LENGTH];
        sig_bytes.copy_from_slice(bytes);
        Ok(Self(sig_bytes))
    }
}

impl Signer<Signature> for Secp256k1KeyPair {
    fn sign(&self, msg: &[u8]) -> Signature {
        EcdsaRoochSignature::new(self, msg).into()
    }
}

//
// EcdsaRecoverable Rooch Signature port
//
#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash, AsRef, AsMut)]
#[as_ref(forward)]
#[as_mut(forward)]
pub struct EcdsaRecoverableRoochSignature(
    #[schemars(with = "Base64")]
    #[serde_as(as = "Readable<Base64, Bytes>")]
    [u8; Secp256k1RecoverablePublicKey::LENGTH + Secp256k1RecoverableSignature::LENGTH + 1],
);

impl RoochSignatureInner for EcdsaRecoverableRoochSignature {
    type Sig = Secp256k1RecoverableSignature;
    type PubKey = Secp256k1RecoverablePublicKey;
    type KeyPair = Secp256k1RecoverableKeyPair;
    const LENGTH: usize =
        Secp256k1RecoverablePublicKey::LENGTH + Secp256k1RecoverableSignature::LENGTH + 1;
}

impl ToFromBytes for EcdsaRecoverableRoochSignature {
    fn from_bytes(bytes: &[u8]) -> Result<Self, FastCryptoError> {
        if bytes.len() != Self::LENGTH {
            return Err(FastCryptoError::InputLengthWrong(Self::LENGTH));
        }
        let mut sig_bytes = [0; Self::LENGTH];
        sig_bytes.copy_from_slice(bytes);
        Ok(Self(sig_bytes))
    }
}

impl Signer<Signature> for Secp256k1RecoverableKeyPair {
    fn sign(&self, msg: &[u8]) -> Signature {
        EcdsaRecoverableRoochSignature::new(self, msg).into()
    }
}

//
// Schnorr Rooch Signature port
//
#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash, AsRef, AsMut)]
#[as_ref(forward)]
#[as_mut(forward)]
pub struct SchnorrRoochSignature(
    #[schemars(with = "Base64")]
    #[serde_as(as = "Readable<Base64, Bytes>")]
    [u8; SchnorrPublicKey::LENGTH + SchnorrSignature::LENGTH + 1],
);

// Implementation useful for simplify testing when mock signature is needed
impl Default for SchnorrRoochSignature {
    fn default() -> Self {
        Self([0; SchnorrPublicKey::LENGTH + SchnorrSignature::LENGTH + 1])
    }
}

impl ToFromBytes for SchnorrRoochSignature {
    fn from_bytes(bytes: &[u8]) -> Result<Self, FastCryptoError> {
        if bytes.len() != Self::LENGTH {
            return Err(FastCryptoError::InputLengthWrong(Self::LENGTH));
        }
        let mut sig_bytes = [0; Self::LENGTH];
        sig_bytes.copy_from_slice(bytes);
        Ok(Self(sig_bytes))
    }
}

impl Signer<Signature> for SchnorrKeyPair {
    fn sign(&self, msg: &[u8]) -> Signature {
        SchnorrRoochSignature::new(self, msg).into()
    }
}

impl RoochSignatureInner for SchnorrRoochSignature {
    type Sig = SchnorrSignature;
    type PubKey = SchnorrPublicKey;
    type KeyPair = SchnorrKeyPair;
    const LENGTH: usize = SchnorrPublicKey::LENGTH + SchnorrSignature::LENGTH + 1;
}

/// Generate a keypair from the specified RNG (useful for testing with seedable rngs).
pub fn get_key_pair_from_rng<KP: KeypairTraits, R>(csprng: &mut R) -> (RoochAddress, KP)
where
    R: rand::CryptoRng + rand::RngCore,
    <KP as KeypairTraits>::PubKey: RoochPublicKey,
{
    let kp = KP::generate(&mut StdRng::from_rng(csprng).unwrap());
    (kp.public().into(), kp)
}

#[cfg(test)]
mod tests {
    use crate::address::RoochAddress;
    use bitcoin::{
        secp256k1::{All, Secp256k1},
        Address, PublicKey,
    };
    use ethers::utils::keccak256;
    use fastcrypto::{
        ed25519::{Ed25519KeyPair, Ed25519PrivateKey},
        secp256k1::schnorr::{SchnorrKeyPair, SchnorrPrivateKey},
        secp256k1::{
            recoverable::{Secp256k1RecoverableKeyPair, Secp256k1RecoverablePrivateKey},
            Secp256k1KeyPair, Secp256k1PrivateKey,
        },
        traits::{KeyPair, ToFromBytes},
    };
    use once_cell::sync::Lazy;

    // this test ensure the Rooch native public key to address keep the same as the old version
    // we should also keep the Rooch native public key to address algorithm the same as the move version
    #[test]
    fn test_native_public_key_to_address() {
        let private_key = Ed25519PrivateKey::from_bytes(&[0u8; 32]).unwrap();
        let keypair: Ed25519KeyPair = private_key.into();
        let address: RoochAddress = keypair.public().into();
        assert_eq!(
            address.to_string(),
            "0x7a1378aafadef8ce743b72e8b248295c8f61c102c94040161146ea4d51a182b6"
        );
    }

    // this test is to ensure that the ECDSA algorithm works for Bitcoin public key to address
    #[test]
    fn test_bitcoin_public_key_to_address() {
        pub static SECP256K1: Lazy<Secp256k1<All>> = Lazy::new(Secp256k1::new);
        let private_key = Secp256k1PrivateKey::from_bytes(&[1u8; 32]).unwrap(); // use 1u8.
        let keypair: Secp256k1KeyPair = private_key.into();
        let general_pk = keypair.public().pubkey;
        let pk = PublicKey::new(general_pk);
        let network = bitcoin::Network::Bitcoin;
        let p2pkh_address = Address::p2pkh(&pk, network);
        let script_pubkey = p2pkh_address.script_pubkey();
        let redeem_script = script_pubkey.as_script();
        let p2pkh_address_str = p2pkh_address.to_string();
        let p2sh_address_str = Address::p2sh(&redeem_script, network)
            .expect("Creating a pay to script hash P2SH address from a script should succeed")
            .to_string();
        let p2wpkh_address_str = Address::p2wpkh(&pk, network)
            .expect("Creating a witness pay to public key address from a public key should succeed")
            .to_string();
        let p2wsh_address_str = Address::p2wsh(&redeem_script, network).to_string();
        let p2tr_address_str =
            Address::p2tr(&SECP256K1, general_pk.x_only_public_key().0, None, network).to_string();

        assert_eq!(p2pkh_address_str, "1C6Rc3w25VHud3dLDamutaqfKWqhrLRTaD");
        assert_eq!(p2sh_address_str, "3DedZ8SErqfunkjqnv8Pta1MKgEuHi22W5");
        assert_eq!(
            p2wpkh_address_str,
            "bc1q0xcqpzrky6eff2g52qdye53xkk9jxkvrh6yhyw"
        );
        assert_eq!(
            p2wsh_address_str,
            "bc1qdudnf8tla4fyptt3n9y9985tq64lqwzr37d4ywpqfzfhtt638glsqaednx"
        );
        assert_eq!(
            p2tr_address_str,
            "bc1p33wm0auhr9kkahzd6l0kqj85af4cswn276hsxg6zpz85xe2r0y8syx4e5t"
        );
    }

    // this test is to ensure that the ECDSA recoverable algorithm works for Ethereum public key to address
    #[test]
    fn test_ethereum_public_key_to_address() {
        let private_key = Secp256k1RecoverablePrivateKey::from_bytes(&[1u8; 32]).unwrap(); // use 1u8.
        let keypair: Secp256k1RecoverableKeyPair = private_key.into();
        let public_key = keypair.public();
        let uncompressed = public_key.pubkey.serialize_uncompressed();
        let uncompressed_64 = uncompressed[1..65].to_vec();
        let hashed = keccak256(uncompressed_64);
        let address_bytes = hashed[12..32].to_vec();
        let address_str = format!("0x{}", hex::encode(address_bytes)); // Include "0x" prefix
        let expected_address = "0x1a642f0e3c3af545e7acbd38b07251b3990914f1";
        assert_eq!(address_str, expected_address);
    }

    // this test is to ensure that the Schnorr algorithm works for Nostr public key to address
    #[test]
    fn test_nostr_public_key_to_address() {
        let private_key = SchnorrPrivateKey::from_bytes(&[1u8; 32]).unwrap(); // ensure not leave 0, use 1u8
        let keypair: SchnorrKeyPair = private_key.into();
        let address: RoochAddress = keypair.public().into();
        assert_eq!(
            address.to_string(),
            "0xa519b36bbecc294726bbfd962ab46ca4e09baacca7cd90d5d2da2331afb363e6"
        );
    }
}

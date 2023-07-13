// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::{
    address::RoochAddress,
    error::{RoochError, RoochResult},
};
use derive_more::{AsMut, AsRef, From};
pub use enum_dispatch::enum_dispatch;
use eyre::eyre;
use fastcrypto::{encoding::{Base64, Encoding}, traits::AllowedRng};
use fastcrypto::hash::{Blake2b256, HashFunction};
use fastcrypto::error::FastCryptoError;
pub use fastcrypto::traits::{
    EncodeDecodeBase64, ToFromBytes,
};
use ed25519_dalek::{Keypair as Ed25519KeyPair, PublicKey as Ed25519PublicKey, Signature as Ed25519Signature, PUBLIC_KEY_LENGTH, SIGNATURE_LENGTH};
use secp256k1::{Secp256k1, KeyPair as Secp256k1KeyPair, PublicKey as Secp256k1PublicKey, schnorr::Signature as SchnorrSignature, ecdsa::Signature as ECDSASignature, XOnlyPublicKey};
use secp256k1::constants::{SCHNORR_PUBLIC_KEY_SIZE, SCHNORR_SIGNATURE_SIZE, PUBLIC_KEY_SIZE, COMPACT_SIGNATURE_SIZE};
use moveos_types::{h256::H256, serde::Readable};
use rand::{rngs::StdRng, SeedableRng};
use schemars::JsonSchema;
use serde::{ser::Serializer, de::DeserializeOwned};
use serde::{Deserialize, Deserializer, Serialize};
use serde_with::{serde_as, Bytes};
use std::{hash::Hash, str::FromStr, fmt::Display, borrow::Borrow};
use strum_macros::EnumString;
// pub use dyn_clone::DynClone;
// pub use traitobject::traitobject;
// pub use dyn_trait::dyn_trait;
// use anyhow::anyhow;
// use std::any::Any;

pub type DefaultHash = Blake2b256;

/// A `Authenticator` is an an abstraction of a account authenticator.
/// It is a part of `AccountAbstraction`

/// The Authenticator scheme which builtin Rooch
#[derive(Copy, Clone, Debug, EnumString, strum_macros::Display)]
#[strum(serialize_all = "lowercase")]
pub enum BuiltinScheme {
    Ed25519,
    MultiEd25519,
    Ecdsa,
    Schnorr,
}

impl BuiltinScheme {
    pub fn flag(&self) -> u8 {
        match self {
            BuiltinScheme::Ed25519 => 0x00,
            BuiltinScheme::MultiEd25519 => 0x01,
            BuiltinScheme::Ecdsa => 0x02,
            BuiltinScheme::Schnorr => 0x03,
        }
    }

    pub fn from_flag(flag: &str) -> Result<BuiltinScheme, RoochError> {
        let byte_int = flag
            .parse::<u8>()
            .map_err(|_| RoochError::KeyConversionError("Invalid key scheme".to_owned()))?;
        Self::from_flag_byte(&byte_int)
    }

    pub fn from_flag_byte(byte_int: &u8) -> Result<BuiltinScheme, RoochError> {
        match byte_int {
            0x00 => Ok(BuiltinScheme::Ed25519),
            0x01 => Ok(BuiltinScheme::MultiEd25519),
            0x02 => Ok(BuiltinScheme::Ecdsa),
            0x03 => Ok(BuiltinScheme::Schnorr),
            _ => Err(RoochError::KeyConversionError(
                "Invalid key scheme".to_owned(),
            )),
        }
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
pub enum RoochKeyPair {
    Ed25519(Ed25519KeyPair),
    Ecdsa(Secp256k1KeyPair),
    Schnorr(Secp256k1KeyPair),
}

impl RoochKeyPair {
    pub fn public(&self) -> PublicKey {
        match self {
            RoochKeyPair::Ed25519(kp) => PublicKey::Ed25519(kp.public().into()),
            RoochKeyPair::Ecdsa(kp) => PublicKey::Ecdsa(kp.public().into()),
            RoochKeyPair::Schnorr(kp) => PublicKey::Schnorr(kp.x_only_public_key().0.into()),
        }
    }
}

impl Signer<Signature> for RoochKeyPair {
    fn sign(&self, msg: &[u8]) -> Signature {
        match self {
            RoochKeyPair::Ed25519(kp) => kp.sign(msg),
            RoochKeyPair::Ecdsa(kp) => kp.sign(msg),
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
                bytes.extend_from_slice(&kp.to_bytes());
            }
            RoochKeyPair::Ecdsa(kp) => {
                bytes.extend_from_slice(kp.as_bytes());
            }
            RoochKeyPair::Schnorr(kp) => {
                bytes.extend_from_slice(&kp.secret_bytes());
            }
        }
        Base64::encode(&bytes[..])
    }

    /// Decode a RoochKeyPair from `flag || privkey` in Base64. The public key is computed directly from the private key bytes.
    fn decode_base64(value: &str) -> Result<Self, eyre::Report> {
        let bytes = Base64::decode(value).map_err(|e| eyre!("{}", e.to_string()))?;
        match BuiltinScheme::from_flag_byte(bytes.first().ok_or_else(|| eyre!("Invalid length"))?) {
            Ok(x) => match x {
                BuiltinScheme::Ed25519 => Ok(RoochKeyPair::Ed25519(Ed25519KeyPair::from_bytes(
                    bytes.get(1..).ok_or_else(|| eyre!("Invalid length"))?,
                )?)),
                BuiltinScheme::Ecdsa => Ok(RoochKeyPair::Ecdsa(Secp256k1KeyPair::from_bytes(
                    bytes.get(1..).ok_or_else(|| eyre!("Invalid length"))?,
                )?)),
                BuiltinScheme::Schnorr => Ok(RoochKeyPair::Schnorr(KeyPair::from_seckey_slice(
                    &Secp256k1::new(),
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

#[derive(Debug, Clone, PartialEq, Eq, From)]
pub enum PublicKey {
    Ed25519(Ed25519PublicKeyAsBytes),
    Ecdsa(Secp256k1PublicKeyAsBytes),
    Schnorr(XOnlyPublicKey),
}

impl AsRef<[u8]> for PublicKey {
    fn as_ref(&self) -> &[u8] {
        match self {
            PublicKey::Ed25519(pk) => &pk.0,
            PublicKey::Ecdsa(pk) => &pk.0,
            PublicKey::Schnorr(pk) => &pk.serialize(),
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
                } else if x == &BuiltinScheme::Schnorr.flag() {
                    let pk: XOnlyPublicKey = XOnlyPublicKey::from_slice(
                        bytes.get(1..).ok_or_else(|| eyre!("Invalid length"))?,
                    )?;
                    Ok(PublicKey::Schnorr(pk.into()))
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
        Ed25519RoochSignature::SCHEME.flag()
    }
    pub fn try_from_bytes(
        scheme: BuiltinScheme,
        key_bytes: &[u8],
    ) -> Result<PublicKey, eyre::Report> {
        match scheme {
            BuiltinScheme::Ed25519 => Ok(PublicKey::Ed25519(
                Ed25519PublicKey::from_bytes(key_bytes)?.into(),
            )),
            BuiltinScheme::Ecdsa => Ok(PublicKey::Ecdsa(
                (&Secp256k1PublicKey::from_bytes(key_bytes)?).into(),
            )),
            BuiltinScheme::Schnorr => Ok(PublicKey::Schnorr(
                XOnlyPublicKey::from_slice(key_bytes)?.into(),
            )),
            _ => Err(eyre!("Unsupported scheme")),
        }
    }

    pub fn scheme(&self) -> BuiltinScheme {
        match self {
            PublicKey::Ed25519(_) => Ed25519RoochSignature::SCHEME,
            PublicKey::Ecdsa(_) => EcdsaRoochSignature::SCHEME,
            PublicKey::Schnorr(_) => SchnorrRoochSignature::SCHEME,
        }
    }
}

pub trait RoochPublicKey {
    const SIGNATURE_SCHEME: BuiltinScheme;
}

impl RoochPublicKey for Ed25519PublicKey {
    const SIGNATURE_SCHEME: BuiltinScheme = BuiltinScheme::Ed25519;
}


impl RoochPublicKey for Secp256k1PublicKey {
    const SIGNATURE_SCHEME: BuiltinScheme = BuiltinScheme::Ecdsa;
}

impl RoochPublicKey for XOnlyPublicKey {
    const SIGNATURE_SCHEME: BuiltinScheme = BuiltinScheme::Schnorr;
}

impl<T: RoochPublicKey + std::convert::AsRef<[u8]>> From<&T> for RoochAddress {
    fn from(pk: &T) -> Self {
        let mut hasher = DefaultHash::default();
        hasher.update([T::SIGNATURE_SCHEME.flag()]);
        hasher.update(pk);
        let g_arr = hasher.finalize();
        RoochAddress(H256(g_arr.digest))
    }
}

impl From<&PublicKey> for RoochAddress {
    fn from(pk: &PublicKey) -> Self {
        let mut hasher = DefaultHash::default();
        hasher.update([pk.flag()]);
        hasher.update(pk);
        let g_arr = hasher.finalize();
        RoochAddress(H256(g_arr.digest))
    }
}

pub trait Authenticator:
    ToFromBytes + Display + Serialize + DeserializeOwned + Send + Sync + 'static + Clone
{
    type PubKey: VerifyingKey<Sig = Self>;
    type PrivKey: SigningKey<Sig = Self>;
    const LENGTH: usize;
}


pub trait VerifyingKey:
    Serialize
    + DeserializeOwned
    + std::hash::Hash
    + Display
    + Eq  // required to make some cached bytes representations explicit.
    + Ord // required to put keys in BTreeMap.
    + ToFromBytes
    + for<'a> From<&'a Self::PrivKey> // conversion PrivateKey -> PublicKey.
    + Send
    + Sync
    + 'static
    + Clone
{
    type PrivKey: SigningKey<PubKey=Self>;
    type Sig: Authenticator<PubKey=Self>;
    const LENGTH: usize;

    /// Use Self to verify that the provided signature for a given message bytestring is authentic.
    /// Returns Error if it is inauthentic, or otherwise returns ().
    fn verify(&self, msg: &[u8], signature: &Self::Sig) -> Result<(), FastCryptoError>;

    // Expected to be overridden by implementations
    /// Batch verification over the same message. Implementations of this method can be fast,
    /// assuming rogue key checks have already been performed.
    /// TODO: take as input a flag to denote if rogue key protection already took place.
    #[cfg(any(test, feature = "experimental"))]
    fn verify_batch_empty_fail(msg: &[u8], pks: &[Self], sigs: &[Self::Sig]) -> Result<(), eyre::Report> {
        if sigs.is_empty() {
            return Err(eyre!("Critical Error! This behaviour can signal something dangerous, and that someone may be trying to bypass signature verification through providing empty batches."));
        }
        if pks.len() != sigs.len() {
            return Err(eyre!("Mismatch between number of signatures and public keys provided"));
        }
        pks.iter()
            .zip(sigs)
            .try_for_each(|(pk, sig)| pk.verify(msg, sig))
            .map_err(|_| eyre!("Signature verification failed"))
    }

    // Expected to be overridden by implementations
    /// Batch verification over different messages. Implementations of this method can be fast,
    /// assuming rogue key checks have already been performed.
    /// TODO: take as input a flag to denote if rogue key protection already took place.
    #[cfg(any(test, feature = "experimental"))]
    fn verify_batch_empty_fail_different_msg<'a, M>(msgs: &[M], pks: &[Self], sigs: &[Self::Sig]) -> Result<(), eyre::Report> where M: Borrow<[u8]> + 'a {
        if sigs.is_empty() {
            return Err(eyre!("Critical Error! This behaviour can signal something dangerous, and that someone may be trying to bypass signature verification through providing empty batches."));
        }
        if pks.len() != sigs.len() || pks.len() != msgs.len() {
            return Err(eyre!("Mismatch between number of messages, signatures and public keys provided"));
        }
        pks.iter()
            .zip(sigs)
            .zip(msgs)
            .try_for_each(|((pk, sig), msg)| pk.verify(msg.borrow(), sig))
            .map_err(|_| eyre!("Signature verification failed"))
    }
}

pub trait SigningKey: ToFromBytes + Serialize + DeserializeOwned + Send + Sync + 'static {
    type PubKey: VerifyingKey<PrivKey = Self>;
    type Sig: Authenticator<PrivKey = Self>;
    const LENGTH: usize;
}

pub trait Signer<Sig> {
    /// Create a new signature over a message.
    fn sign(&self, msg: &[u8]) -> Sig;
}

pub trait KeypairTraits: Sized + From<Self::PrivKey> + Signer<Self::Sig> + EncodeDecodeBase64 + FromStr
{
    // Define the methods required for the KeypairTraits trait
    /// Trait impl'd by a public / private key pair in asymmetric cryptography.

    type PubKey: VerifyingKey<PrivKey = Self::PrivKey, Sig = Self::Sig>;
    type PrivKey: SigningKey<PubKey = Self::PubKey, Sig = Self::Sig>;
    type Sig: Authenticator<PubKey = Self::PubKey, PrivKey = Self::PrivKey>;

    /// Get the public key.
    fn public(&'_ self) -> &'_ Self::PubKey;
    /// Get the private key.
    fn private(self) -> Self::PrivKey;

    #[cfg(feature = "copy_key")]
    fn copy(&self) -> Self;

    /// Generate a new keypair using the given RNG.
    fn generate<R: AllowedRng>(rng: &mut R) -> Self;
}

// #[enum_dispatch(RoochSignatureInner)]
// pub enum RoochSignatureWrapper {
//     // Define the possible wrapper variants
// }

// impl RoochSignatureInner for RoochSignatureWrapper {
//     type Sig = dyn Authenticator<PubKey = Self::PubKey>;
//     type PubKey = dyn VerifyingKey<Sig = Self::Sig> + RoochPublicKey;
//     type KeyPair = dyn KeypairTraits<PubKey = Self::PubKey, Sig = Self::Sig>;

//     fn get_verification_inputs(&self, author: RoochAddress) -> RoochResult<(Self::Sig, Self::PubKey)> {
//         // Is this signature emitted by the expected author?
//         let bytes = self.public_key_bytes();
//         let pk = Self::PubKey::from_bytes(bytes)
//             .map_err(|_| RoochError::KeyConversionError("Invalid public key".to_owned()))?;
    
//         let received_addr = RoochAddress::from(&pk);
//         if received_addr != author {
//             return Err(RoochError::IncorrectSigner {
//                 error: format!("Signature get_verification_inputs() failure. Author is {}, received address is {}", author, received_addr)
//             });
//         }
    
//         // deserialize the signature
//         let signature = Self::Sig::from_bytes(self.signature_bytes()).map_err(|_| {
//             RoochError::InvalidSignature {
//                 error: "Fail to get pubkey and sig".to_owned(),
//             }
//         })?;
    
//         Ok((signature, pk))
//     }

//     fn new(kp: &Self::KeyPair, message: &[u8]) -> Self {
//         let sig = Signer::sign(kp, message);

//         let mut signature_bytes: Vec<u8> = Vec::new();
//         signature_bytes
//             .extend_from_slice(&[<Self::PubKey as RoochPublicKey>::SIGNATURE_SCHEME.flag()]);

//         signature_bytes.extend_from_slice(sig.as_ref());
//         signature_bytes.extend_from_slice(kp.public().as_ref());
//         Self::from_bytes(&signature_bytes[..])
//             .expect("Serialized signature did not have expected size")
//     }
// }

// impl<T> From<T> for RoochSignatureWrapper
// where
//     T: RoochSignatureInner,
// {
//     fn from(inner: T) -> Self {
//         RoochSignatureWrapper::Inner(inner)
//     }
// }

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
                error: format!("Signature get_verification_inputs() failure. Author is {}, received address is {}", author, received_addr)
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
            BuiltinScheme::Schnorr => Ok(CompressedSignature::Schnorr(
                SchnorrSignature::from_slice(bytes).map_err(|_| {
                    RoochError::InvalidSignature {
                        error: "Cannot parse sig".to_owned(),
                    }
                })?,
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
            BuiltinScheme::Schnorr => Ok(PublicKey::Schnorr(
                (XOnlyPublicKey::from_slice(bytes)
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
            Signature::SchnorrRoochSignature(sig) => sig.as_ref(),
        }
    }
}
impl AsMut<[u8]> for Signature {
    fn as_mut(&mut self) -> &mut [u8] {
        match self {
            Signature::Ed25519RoochSignature(sig) => sig.as_mut(),
            Signature::EcdsaRoochSignature(sig) => sig.as_mut(),
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CompressedSignature {
    Ed25519(Ed25519SignatureAsBytes),
    Ecdsa(Secp256k1SignatureAsBytes),
    Schnorr(SchnorrSignature),
}

impl AsRef<[u8]> for CompressedSignature {
    fn as_ref(&self) -> &[u8] {
        match self {
            CompressedSignature::Ed25519(sig) => &sig.0,
            CompressedSignature::Ecdsa(sig) => &sig.0,
            CompressedSignature::Schnorr(sig) => sig.as_ref(),
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
// Schnorr Rooch Signature port
//
#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash, AsRef, AsMut)]
#[as_ref(forward)]
#[as_mut(forward)]
pub struct SchnorrRoochSignature(
    #[schemars(with = "Base64")]
    #[serde_as(as = "Readable<Base64, Bytes>")]
    [u8; SCHNORR_PUBLIC_KEY_SIZE + SCHNORR_SIGNATURE_SIZE + 1],
);

// Implementation useful for simplify testing when mock signature is needed
impl Default for SchnorrRoochSignature {
    fn default() -> Self {
        Self([0; SCHNORR_PUBLIC_KEY_SIZE + SCHNORR_SIGNATURE_SIZE + 1])
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

impl RoochSignatureInner for SchnorrRoochSignature {
    type Sig = SchnorrSignature;
    type PubKey = XOnlyPublicKey;
    type KeyPair = Secp256k1KeyPair;
    const LENGTH: usize = SCHNORR_PUBLIC_KEY_SIZE + SCHNORR_SIGNATURE_SIZE + 1;
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
    [u8; PUBLIC_KEY_LENGTH + SIGNATURE_LENGTH + 1],
);

// Implementation useful for simplify testing when mock signature is needed
impl Default for Ed25519RoochSignature {
    fn default() -> Self {
        Self([0; PUBLIC_KEY_LENGTH + SIGNATURE_LENGTH + 1])
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
    [u8; PUBLIC_KEY_SIZE + COMPACT_SIGNATURE_SIZE + 1],
);

impl RoochSignatureInner for EcdsaRoochSignature {
    type Sig = Secp256k1Signature;
    type PubKey = Secp256k1PublicKey;
    type KeyPair = Secp256k1KeyPair;
    const LENGTH: usize = PUBLIC_KEY_SIZE + COMPACT_SIGNATURE_SIZE + 1;
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

// TODO decide keypair sig ECDSA or Schnorr?
impl Signer<Signature> for Secp256k1KeyPair {
    fn sign(&self, msg: &[u8]) -> Signature {
        EcdsaRoochSignature::new(self, msg).into()
    }
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
    use crate::address::{RoochAddress, NostrAddress, self};
    use fastcrypto::{
        ed25519::{Ed25519KeyPair, Ed25519PrivateKey},
        traits::{KeyPair, ToFromBytes},
    };
    use secp256k1::{SecretKey, KeyPair as Secp256k1KeyPair, Secp256k1};

    // this test ensure the public key to address keep the same as the old version
    // we should also keep the public key to address algorithm the same as the move version
    #[test]
    fn test_public_key_to_address() {
        let private_key = Ed25519PrivateKey::from_bytes(&[0u8; 32]).unwrap();
        let keypair: Ed25519KeyPair = private_key.into();
        //println!("public_key: {}", hex::encode(keypair.public().as_bytes()));
        let address: RoochAddress = keypair.public().into();
        //println!("address: {:?}", address);
        assert_eq!(
            address.to_string(),
            "0x7a1378aafadef8ce743b72e8b248295c8f61c102c94040161146ea4d51a182b6"
        );
    }

    #[test]
    fn test_x_only_public_key_to_address() {
        let secret_key: SecretKey = SecretKey::from_slice(&[0u8; 32]).unwrap();
        let secp = Secp256k1::new();
        let keypair: Secp256k1KeyPair = secret_key.keypair(&secp);
        println!("public_key: {}", hex::encode(keypair.x_only_public_key().0.serialize()));
        let address: NostrAddress = address::NostrAddress(keypair.x_only_public_key().0);
        println!("address: {:?}", address);
        assert_eq!(
            address.0.to_string(),
            "0x7a1378aafadef8ce743b72e8b248295c8f61c102c94040161146ea4d51a182b6"
        );
    }
}

// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::{
    address::RoochAddress,
    error::{RoochError, RoochResult},
};
use derive_more::{AsMut, AsRef, From};
use ed25519_dalek::{
    Keypair as Ed25519KeyPair, PublicKey as Ed25519PublicKey, Signature as Ed25519Signature,
    PUBLIC_KEY_LENGTH, SIGNATURE_LENGTH,
};
pub use enum_dispatch::enum_dispatch;
use eyre::eyre;
use fastcrypto::encoding::{Base64, Encoding};
use fastcrypto::error::FastCryptoError;
use fastcrypto::hash::{Blake2b256, HashFunction};
use moveos_types::{h256::H256, serde::Readable};
use rand::{rngs::StdRng, CryptoRng, RngCore, SeedableRng};
use schemars::JsonSchema;
use secp256k1::constants::{
    COMPACT_SIGNATURE_SIZE, PUBLIC_KEY_SIZE, SCHNORR_PUBLIC_KEY_SIZE, SCHNORR_SIGNATURE_SIZE,
};
use secp256k1::{
    ecdsa::Signature as ECDSASignature, schnorr::Signature as SchnorrSignature,
    KeyPair as Secp256k1KeyPair, Message, PublicKey as Secp256k1PublicKey, Secp256k1,
    XOnlyPublicKey,
};
use serde::{de::DeserializeOwned, ser::Serializer};
use serde::{Deserialize, Deserializer, Serialize};
use serde_with::{serde_as, Bytes};
use std::{fmt::Display, hash::Hash, str::FromStr};
use strum_macros::EnumString;

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
        BuiltinScheme::Ed25519.flag()
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
    + Eq
    + Ord
    + ToFromBytes
    + for<'a> From<&'a Self::PrivKey>
    + Send
    + Sync
    + 'static
    + Clone
{
    type PrivKey: SigningKey<PubKey = Self>;
    type Sig: Authenticator<PubKey = Self>;
    const LENGTH: usize;

    fn verify(&self, msg: &[u8], signature: &Self::Sig) -> Result<(), FastCryptoError>;

    #[cfg(any(test, feature = "experimental"))]
    fn verify_batch_empty_fail(
        msg: &[u8],
        pks: &[Self],
        sigs: &[Self::Sig],
    ) -> Result<(), eyre::Report> {
        if sigs.is_empty() {
            return Err(eyre!("Critical Error! This behaviour can signal something dangerous, and that someone may be trying to bypass signature verification through providing empty batches."));
        }
        if pks.len() != sigs.len() {
            return Err(eyre!(
                "Mismatch between number of signatures and public keys provided"
            ));
        }
        pks.iter()
            .zip(sigs)
            .try_for_each(|(pk, sig)| pk.verify(msg, sig))
            .map_err(|_| eyre!("Signature verification failed"))
    }

    #[cfg(any(test, feature = "experimental"))]
    fn verify_batch_empty_fail_different_msg<'a, M>(
        msgs: &[M],
        pks: &[Self],
        sigs: &[Self::Sig],
    ) -> Result<(), eyre::Report>
    where
        M: std::borrow::Borrow<[u8]> + 'a,
    {
        if sigs.is_empty() {
            return Err(eyre!("Critical Error! This behaviour can signal something dangerous, and that someone may be trying to bypass signature verification through providing empty batches."));
        }
        if pks.len() != sigs.len() || pks.len() != msgs.len() {
            return Err(eyre!(
                "Mismatch between number of messages, signatures and public keys provided"
            ));
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
    fn sign(&self, msg: &[u8]) -> Sig;
}

pub trait ToFromBytes: AsRef<[u8]> + Sized {
    /// Parse an object from its byte representation
    fn from_bytes(bytes: &[u8]) -> Result<Self, FastCryptoError>;

    /// Borrow a byte slice representing the serialized form of this object
    fn as_bytes(&self) -> &[u8] {
        self.as_ref()
    }
}

pub trait EncodeDecodeBase64: Sized {
    fn encode_base64(&self) -> String;
    fn decode_base64(value: &str) -> Result<Self, eyre::Report>;
}

pub trait AllowedRng: CryptoRng + RngCore {}

pub trait KeypairTraits:
    Sized + From<Self::PrivKey> + Signer<Self::Sig> + EncodeDecodeBase64 + FromStr
{
    type PubKey: VerifyingKey<PrivKey = Self::PrivKey, Sig = Self::Sig>;
    type PrivKey: SigningKey<PubKey = Self::PubKey, Sig = Self::Sig>;
    type Sig: Authenticator<PubKey = Self::PubKey, PrivKey = Self::PrivKey>;

    fn public(&'_ self) -> &'_ Self::PubKey;
    fn private(self) -> Self::PrivKey;

    #[cfg(feature = "copy_key")]
    fn copy(&self) -> Self;

    fn generate<R: AllowedRng>(rng: &mut R) -> Self;
}

pub trait RoochSignatureInner<Sig, PubKey, KeyPair>:
    Sized + ToFromBytes + PartialEq + Eq + Hash
{
    fn get_verification_inputs(&self, author: RoochAddress) -> RoochResult<(Sig, PubKey)>;
    fn new(kp: &KeyPair, message: &[u8]) -> Self;
}

// //
// // Account Signatures
// //
// // This struct exists due to the limitations of the `enum_dispatch` library.
// //
// pub trait RoochSignatureInner: Sized + ToFromBytes + PartialEq + Eq + Hash {
//     type Sig: Authenticator<PubKey = Self::PubKey>;
//     type PubKey: VerifyingKey<Sig = Self::Sig> + RoochPublicKey;
//     type KeyPair: KeypairTraits<PubKey = Self::PubKey, Sig = Self::Sig>;

//     const LENGTH: usize = Self::Sig::LENGTH + Self::PubKey::LENGTH + 1;
//     const SCHEME: BuiltinScheme = Self::PubKey::SIGNATURE_SCHEME;

//     fn get_verification_inputs(
//         &self,
//         author: RoochAddress,
//     ) -> RoochResult<(Self::Sig, Self::PubKey)> {
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
//             .extend_from_slice(&[<PubKey as RoochPublicKey>::SIGNATURE_SCHEME.flag()]);

//         signature_bytes.extend_from_slice(sig.as_ref());
//         signature_bytes.extend_from_slice(kp.public().as_ref());
//         Self::from_bytes(&signature_bytes[..])
//             .expect("Serialized signature did not have expected size")
//     }
// }

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
                Ed25519Signature::from_bytes(bytes)
                    .map_err(|_| RoochError::InvalidSignature {
                        error: "Cannot parse sig".to_owned(),
                    })?
                    .into(),
            )),
            BuiltinScheme::Ecdsa => Ok(CompressedSignature::Ecdsa(
                (&Secp256k1Signature::from_bytes(bytes).map_err(|_| {
                    RoochError::InvalidSignature {
                        error: "Cannot parse sig".to_owned(),
                    })?
                    .into(),
            )),
            BuiltinScheme::Schnorr => Ok(CompressedSignature::Schnorr(
                SchnorrSignature::from_slice(bytes).map_err(|_| RoochError::InvalidSignature {
                    error: "Cannot parse sig".to_owned(),
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
                XOnlyPublicKey::from_slice(bytes)
                    .map_err(|_| RoochError::KeyConversionError("Cannot parse pk".to_owned()))?,
            )
            .into()),
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
                if x == &BuiltinScheme::Ed25519.flag() {
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

impl RoochSignature for ECDSARoochSignature {
    fn signature_bytes(&self) -> &[u8] {
        // Access array slice is safe because the array bytes is initialized as
        // flag || signature || pubkey with its defined length.
        &self.as_bytes()[1..1 + PUBLIC_KEY_SIZE + COMPACT_SIGNATURE_SIZE]
    }

    fn public_key_bytes(&self) -> &[u8] {
        // Access array slice is safe because the array bytes is initialized as
        // flag || signature || pubkey with its defined length.
        &self.as_bytes()[PUBLIC_KEY_SIZE + COMPACT_SIGNATURE_SIZE + 1..]
    }

    fn scheme(&self) -> BuiltinScheme {
        BuiltinScheme::ECDSA
    }

    fn verify_secure<T>(&self, value: &T, author: RoochAddress) -> Result<(), RoochError>
    where
        T: Serialize,
    {
        let mut hasher = DefaultHash::default();
        hasher.update(&bcs::to_bytes(&value).expect("Message serialization should not fail"));
        let digest = hasher.finalize().digest;

        let (sig, pk) = &self.get_verification_inputs(author)?;
        let message = Message::from_slice(&digest).unwrap();
        Secp256k1::verify_ecdsa(&Secp256k1::new(), &message, sig, pk).map_err(|e| {
            RoochError::InvalidSignature {
                error: format!("Fail to verify user sig {}", e),
            }
        })
    }
}

impl RoochSignature for Ed25519RoochSignature {
    fn signature_bytes(&self) -> &[u8] {
        // Access array slice is safe because the array bytes is initialized as
        // flag || signature || pubkey with its defined length.
        &self.as_bytes()[1..1 + PUBLIC_KEY_LENGTH + SIGNATURE_LENGTH]
    }

    fn public_key_bytes(&self) -> &[u8] {
        // Access array slice is safe because the array bytes is initialized as
        // flag || signature || pubkey with its defined length.
        &self.as_bytes()[PUBLIC_KEY_LENGTH + SIGNATURE_LENGTH + 1..]
    }

    fn scheme(&self) -> BuiltinScheme {
        BuiltinScheme::Ed25519
    }

    fn verify_secure<T>(&self, value: &T, author: RoochAddress) -> Result<(), RoochError>
    where
        T: Serialize,
    {
        let mut hasher = DefaultHash::default();
        hasher.update(&bcs::to_bytes(&value).expect("Message serialization should not fail"));
        let digest = hasher.finalize().digest;

        let (sig, pk) = &self.get_verification_inputs(author)?;
        pk.verify_strict(&digest, sig)
            .map_err(|e| RoochError::InvalidSignature {
                error: format!("Fail to verify user sig {}", e),
            })
    }
}

impl RoochSignature for SchnorrRoochSignature {
    fn signature_bytes(&self) -> &[u8] {
        // Access array slice is safe because the array bytes is initialized as
        // flag || signature || pubkey with its defined length.
        &self.as_bytes()[1..1 + SCHNORR_PUBLIC_KEY_SIZE + SCHNORR_SIGNATURE_SIZE]
    }

    fn public_key_bytes(&self) -> &[u8] {
        // Access array slice is safe because the array bytes is initialized as
        // flag || signature || pubkey with its defined length.
        &self.as_bytes()[SCHNORR_PUBLIC_KEY_SIZE + SCHNORR_SIGNATURE_SIZE + 1..]
    }

    fn scheme(&self) -> BuiltinScheme {
        BuiltinScheme::Schnorr
    }

    fn verify_secure<T>(&self, value: &T, author: RoochAddress) -> Result<(), RoochError>
    where
        T: Serialize,
    {
        let mut hasher = DefaultHash::default();
        hasher.update(&bcs::to_bytes(&value).expect("Message serialization should not fail"));
        let digest = hasher.finalize().digest;

        let (sig, pk) = &self.get_verification_inputs(author)?;
        let message = Message::from_slice(&digest).unwrap();
        Secp256k1::verify_schnorr(&Secp256k1::new(), sig, &message, pk.into()).map_err(|e| {
            RoochError::InvalidSignature {
                error: format!("Fail to verify user sig {}", e),
            }
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
        if bytes.len() != SCHNORR_PUBLIC_KEY_SIZE + SCHNORR_SIGNATURE_SIZE + 1 {
            return Err(FastCryptoError::InputLengthWrong(
                SCHNORR_PUBLIC_KEY_SIZE + SCHNORR_SIGNATURE_SIZE + 1,
            ));
        }
        let mut sig_bytes = [0; SCHNORR_PUBLIC_KEY_SIZE + SCHNORR_SIGNATURE_SIZE + 1];
        sig_bytes.copy_from_slice(bytes);
        Ok(Self(sig_bytes))
    }
}

impl Signer<Signature> for Secp256k1KeyPair {
    fn sign(&self, msg: &[u8]) -> Signature {
        SchnorrRoochSignature::new(self, msg).into()
    }
}

impl RoochSignatureInner<SchnorrSignature, XOnlyPublicKey, Secp256k1KeyPair>
    for SchnorrRoochSignature
{
    fn get_verification_inputs(
        &self,
        author: RoochAddress,
    ) -> RoochResult<(SchnorrSignature, XOnlyPublicKey)> {
        // Is this signature emitted by the expected author?
        let bytes = self.public_key_bytes();

        let pk = XOnlyPublicKey::from_slice(bytes)
            .map_err(|_| RoochError::KeyConversionError("Invalid public key".to_owned()))?;

        let received_addr = RoochAddress::from(&pk);
        if received_addr != author {
            return Err(RoochError::IncorrectSigner {
                error: format!("Signature get_verification_inputs() failure. Author is {}, received address is {}", author, received_addr)
            });
        }

        // deserialize the signature
        let signature = SchnorrSignature::from_slice(self.signature_bytes()).map_err(|_| {
            RoochError::InvalidSignature {
                error: "Fail to get pubkey and sig".to_owned(),
            }
        })?;

        Ok((signature, pk))
    }

    fn new(kp: &Secp256k1KeyPair, message: &[u8]) -> Self {
        let sig: SchnorrSignature = SchnorrSignature::from_slice(message).unwrap().into();

        let mut signature_bytes: Vec<u8> = Vec::new();
        signature_bytes
            .extend_from_slice(&[<XOnlyPublicKey as RoochPublicKey>::SIGNATURE_SCHEME.flag()]);

        signature_bytes.extend_from_slice(sig.as_ref());
        signature_bytes.extend_from_slice(kp.x_only_public_key().0.serialize().as_ref());
        Self::from_bytes(&signature_bytes[..])
            .expect("Serialized signature did not have expected size")
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
        if bytes.len() != PUBLIC_KEY_LENGTH + SIGNATURE_LENGTH + 1 {
            return Err(FastCryptoError::InputLengthWrong(
                PUBLIC_KEY_LENGTH + SIGNATURE_LENGTH + 1,
            ));
        }
        let mut sig_bytes = [0; PUBLIC_KEY_LENGTH + SIGNATURE_LENGTH + 1];
        sig_bytes.copy_from_slice(bytes);
        Ok(Self(sig_bytes))
    }
}

impl Signer<Signature> for Ed25519KeyPair {
    fn sign(&self, msg: &[u8]) -> Signature {
        Ed25519RoochSignature::new(self, msg).into()
    }
}

impl RoochSignatureInner<Ed25519Signature, Ed25519PublicKey, Ed25519KeyPair>
    for Ed25519RoochSignature
{
    fn get_verification_inputs(
        &self,
        author: RoochAddress,
    ) -> RoochResult<(Ed25519Signature, Ed25519PublicKey)> {
        // Is this signature emitted by the expected author?
        let bytes = self.public_key_bytes();

        let pk = Ed25519PublicKey::from_bytes(bytes)
            .map_err(|_| RoochError::KeyConversionError("Invalid public key".to_owned()))?;

        let received_addr = RoochAddress::from(&pk);
        if received_addr != author {
            return Err(RoochError::IncorrectSigner {
                error: format!("Signature get_verification_inputs() failure. Author is {}, received address is {}", author, received_addr)
            });
        }

        // deserialize the signature
        let signature = Ed25519Signature::from_bytes(self.signature_bytes()).map_err(|_| {
            RoochError::InvalidSignature {
                error: "Fail to get pubkey and sig".to_owned(),
            }
        })?;

        Ok((signature, pk))
    }

    fn new(kp: &Ed25519KeyPair, message: &[u8]) -> Self {
        let sig: Ed25519Signature = Ed25519Signature::from_bytes(message).unwrap().into();

        let mut signature_bytes: Vec<u8> = Vec::new();
        signature_bytes
            .extend_from_slice(&[<Ed25519PublicKey as RoochPublicKey>::SIGNATURE_SCHEME.flag()]);

        signature_bytes.extend_from_slice(sig.to_bytes().as_ref());
        signature_bytes.extend_from_slice(kp.public.as_ref());
        Self::from_bytes(&signature_bytes[..])
            .expect("Serialized signature did not have expected size")
    }
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
        if bytes.len() != PUBLIC_KEY_SIZE + COMPACT_SIGNATURE_SIZE + 1 {
            return Err(FastCryptoError::InputLengthWrong(
                PUBLIC_KEY_SIZE + COMPACT_SIGNATURE_SIZE + 1,
            ));
        }
        let mut sig_bytes = [0; PUBLIC_KEY_SIZE + COMPACT_SIGNATURE_SIZE + 1];
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
    use crate::address::{self, NostrAddress, RoochAddress};
    use secp256k1::{KeyPair as Secp256k1KeyPair, Secp256k1, SecretKey};

    // this test ensure the public key to address keep the same as the old version
    // we should also keep the public key to address algorithm the same as the move version
    #[test]
    fn test_public_key_to_address() {
        let keypair = Secp256k1KeyPair::from_seckey_slice(&Secp256k1::new(), &[0u8; 32]).unwrap();
        //println!("public_key: {}", hex::encode(keypair.public().as_bytes()));
        let address: RoochAddress = keypair.public_key().into();
        //println!("address: {:?}", address);
        assert_eq!(
            address.to_string(),
            "0x7a1378aafadef8ce743b72e8b248295c8f61c102c94040161146ea4d51a182b6"
        );
    }

    #[test]
    fn test_x_only_public_key_to_nostr_address() {
        let secret_key: SecretKey = SecretKey::from_slice(&[0u8; 32]).unwrap();
        let secp = Secp256k1::new();
        let keypair: Secp256k1KeyPair = secret_key.keypair(&secp);
        println!(
            "public_key: {}",
            hex::encode(keypair.x_only_public_key().0.serialize())
        );
        let address: NostrAddress = address::NostrAddress(keypair.x_only_public_key().0);
        println!("address: {:?}", address);
        assert_eq!(
            address.0.to_string(),
            "0x7a1378aafadef8ce743b72e8b248295c8f61c102c94040161146ea4d51a182b6"
        );
    }
}

// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::address::RoochAddress;
use moveos_types::h256::H256;
use crate::{error::RoochError, rooch_serde::Readable};
use derive_more::{AsMut, AsRef, From};
use eyre::eyre;
use fastcrypto::ed25519::{Ed25519KeyPair, Ed25519PublicKey, Ed25519Signature};
use fastcrypto::encoding::Base64;
use fastcrypto::encoding::Encoding;
use fastcrypto::error::FastCryptoError;
use fastcrypto::hash::*;
pub use fastcrypto::traits::KeyPair as KeypairTraits;
pub use fastcrypto::traits::Signer;
pub use fastcrypto::traits::{
    AggregateAuthenticator, Authenticator, EncodeDecodeBase64, SigningKey, ToFromBytes,
    VerifyingKey,
};
use rand::rngs::StdRng;
use rand::SeedableRng;
use schemars::JsonSchema;
use serde::ser::Serializer;
use serde::{Deserialize, Deserializer, Serialize};
use serde_with::{serde_as, Bytes};
use std::hash::Hash;
use strum_macros::EnumString;

pub type DefaultHash = Blake2b256;

///////////////////////////////////////////////
/// Account Keys
///
/// * The following section defines the keypairs that are used by
/// * accounts to interact with Rooch.
/// * Currently we support eddsa and ecdsa on Rooch.
///
#[derive(Debug, From, PartialEq, Eq)]
pub enum RoochKeyPair {
    Ed25519(Ed25519KeyPair),
}

impl RoochKeyPair {
    pub fn public(&self) -> PublicKey {
        match self {
            RoochKeyPair::Ed25519(kp) => PublicKey::Ed25519(kp.public().clone()),
        }
    }
}

impl EncodeDecodeBase64 for RoochKeyPair {
    /// Encode a RoochKeyPair as `flag || privkey` in Base64. Note that the pubkey is not encoded.
    fn encode_base64(&self) -> String {
        let mut bytes: Vec<u8> = Vec::new();
        match self {
            RoochKeyPair::Ed25519(kp) => {
                bytes.push(self.public().flag());
                bytes.extend_from_slice(kp.as_bytes());
            }
        }
        Base64::encode(&bytes[..])
    }

    /// Decode a RoochKeyPair from `flag || privkey` in Base64. The public key is computed directly from the private key bytes.
    fn decode_base64(value: &str) -> Result<Self, eyre::Report> {
        let bytes = Base64::decode(value).map_err(|e| eyre!("{}", e.to_string()))?;
        match SignatureScheme::from_flag_byte(bytes.first().ok_or_else(|| eyre!("Invalid length"))?)
        {
            Ok(x) => match x {
                SignatureScheme::ED25519 => Ok(RoochKeyPair::Ed25519(Ed25519KeyPair::from_bytes(
                    bytes.get(1..).ok_or_else(|| eyre!("Invalid length"))?,
                )?)),
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

#[derive(Clone, PartialEq, Eq, From, JsonSchema)]
pub enum PublicKey {
    #[schemars(with = "Base64")]
    Ed25519(Ed25519PublicKey),
}

impl PublicKey {
    pub fn flag(&self) -> u8 {
        RoochEd25519Signature::SCHEME.flag()
    }
    pub fn try_from_bytes(_: SignatureScheme, key_bytes: &[u8]) -> Result<PublicKey, eyre::Report> {
        Ok(PublicKey::Ed25519(Ed25519PublicKey::from_bytes(key_bytes)?))
    }

    pub fn scheme(&self) -> SignatureScheme {
        match self {
            PublicKey::Ed25519(_) => RoochEd25519Signature::SCHEME,
        }
    }
}

impl AsRef<[u8]> for PublicKey {
    fn as_ref(&self) -> &[u8] {
        match self {
            PublicKey::Ed25519(pk) => pk.as_ref(),
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

        let pk =
            Ed25519PublicKey::from_bytes(bytes.get(1..).ok_or_else(|| eyre!("Invalid length"))?)?;

        Ok(PublicKey::Ed25519(pk))
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

impl<T: RoochPublicKey> From<&T> for RoochAddress {
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

#[derive(Deserialize, Serialize, Debug, EnumString, strum_macros::Display)]
#[strum(serialize_all = "lowercase")]
pub enum SignatureScheme {
    ED25519,
}

impl SignatureScheme {
    pub fn flag(&self) -> u8 {
        match self {
            SignatureScheme::ED25519 => 0x00,
        }
    }

    pub fn from_flag(flag: &str) -> Result<SignatureScheme, RoochError> {
        let byte_int = flag
            .parse::<u8>()
            .map_err(|_| RoochError::KeyConversionError("Invalid key scheme".to_string()))?;
        Self::from_flag_byte(&byte_int)
    }

    pub fn from_flag_byte(byte_int: &u8) -> Result<SignatureScheme, RoochError> {
        match byte_int {
            0x00 => Ok(SignatureScheme::ED25519),
            _ => Err(RoochError::KeyConversionError(
                "Invalid key scheme".to_string(),
            )),
        }
    }
}

pub trait RoochSignatureInner: Sized + ToFromBytes + PartialEq + Eq + Hash {
    type Sig: Authenticator<PubKey = Self::PubKey>;
    type PubKey: VerifyingKey<Sig = Self::Sig> + RoochPublicKey;
    type KeyPair: KeypairTraits<PubKey = Self::PubKey, Sig = Self::Sig>;

    const LENGTH: usize = Self::Sig::LENGTH + Self::PubKey::LENGTH + 1;
    const SCHEME: SignatureScheme = Self::PubKey::SIGNATURE_SCHEME;

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

pub trait RoochPublicKey: VerifyingKey {
    const SIGNATURE_SCHEME: SignatureScheme;
}

impl RoochPublicKey for Ed25519PublicKey {
    const SIGNATURE_SCHEME: SignatureScheme = SignatureScheme::ED25519;
}

//
// Ed25519 Rooch Signature port
//

#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash, AsRef, AsMut)]
#[as_ref(forward)]
#[as_mut(forward)]
pub struct RoochEd25519Signature(
    #[schemars(with = "Base64")]
    #[serde_as(as = "Readable<Base64, Bytes>")]
    [u8; Ed25519PublicKey::LENGTH + Ed25519Signature::LENGTH + 1],
);

// Implementation useful for simplify testing when mock signature is needed
impl Default for RoochEd25519Signature {
    fn default() -> Self {
        Self([0; Ed25519PublicKey::LENGTH + Ed25519Signature::LENGTH + 1])
    }
}

impl ToFromBytes for RoochEd25519Signature {
    fn from_bytes(bytes: &[u8]) -> Result<Self, FastCryptoError> {
        if bytes.len() != Self::LENGTH {
            return Err(FastCryptoError::InputLengthWrong(Self::LENGTH));
        }
        let mut sig_bytes = [0; Self::LENGTH];
        sig_bytes.copy_from_slice(bytes);
        Ok(Self(sig_bytes))
    }
}

impl RoochSignatureInner for RoochEd25519Signature {
    type Sig = Ed25519Signature;
    type PubKey = Ed25519PublicKey;
    type KeyPair = Ed25519KeyPair;
    const LENGTH: usize = Ed25519PublicKey::LENGTH + Ed25519Signature::LENGTH + 1;
}

/// Generate a keypair from the specified RNG (useful for testing with seedable rngs).
pub fn get_key_pair_from_rng<KP: KeypairTraits, R>(csprng: &mut R) -> (RoochAddress, KP)
where
    R: rand::CryptoRng + rand::RngCore,
    <KP as KeypairTraits>::PubKey: RoochPublicKey,
{
    let kp = KP::generate(&mut StdRng::from_rng(csprng).unwrap());
    (
        RoochAddress(moveos_types::h256::sha3_256_of(&[
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ])),
        kp,
    )
    //    (kp.public().into(), kp)
}

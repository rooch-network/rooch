// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use crate::{
    address::{BitcoinAddress, RoochAddress},
    authentication_key::AuthenticationKey,
    error::{RoochError, RoochResult},
    rooch_key::ROOCH_SECRET_KEY_HRP,
};
use anyhow::{anyhow, bail};
use bech32::{encode, Bech32, EncodeError};
use bitcoin::secp256k1::SecretKey;
use derive_more::{AsRef, From};
pub use enum_dispatch::enum_dispatch;
use eyre::eyre;
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
    encoding::{Base64, Encoding},
};
use fastcrypto::{
    error::FastCryptoError,
    secp256k1::{Secp256k1KeyPair, Secp256k1PublicKeyAsBytes},
};
use fastcrypto::{
    hash::{Blake2b256, HashFunction},
    secp256k1::{Secp256k1PublicKey, Secp256k1Signature, Secp256k1SignatureAsBytes},
};
use multibase;
use schemars::JsonSchema;
use serde::ser::Serializer;
use serde::{Deserialize, Deserializer, Serialize};
use serde_with::serde_as;
use std::{hash::Hash, str::FromStr};

pub type DefaultHash = Blake2b256;

#[derive(Debug, PartialEq)]
pub enum SignatureScheme {
    Ed25519,
    Secp256k1,
}

impl SignatureScheme {
    pub fn flag(&self) -> u8 {
        match self {
            SignatureScheme::Ed25519 => 0,
            SignatureScheme::Secp256k1 => 1,
        }
    }

    pub fn from_flag_byte(byte_int: u8) -> Result<SignatureScheme, RoochError> {
        match byte_int {
            0 => Ok(SignatureScheme::Ed25519),
            1 => Ok(SignatureScheme::Secp256k1),
            _ => Err(RoochError::InvalidSignatureScheme),
        }
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, From, PartialEq, Eq)]
pub enum RoochKeyPair {
    ///For SessionKey
    Ed25519(Ed25519KeyPair),
    ///For Bitcoin
    Secp256k1(Secp256k1KeyPair),
}

impl RoochKeyPair {
    pub fn generate_ed25519() -> Self {
        let rng = &mut rand::thread_rng();
        let ed25519_keypair = Ed25519KeyPair::generate(rng);
        RoochKeyPair::Ed25519(ed25519_keypair)
    }

    pub fn generate_secp256k1() -> Self {
        let rng = &mut rand::thread_rng();
        let secp256k1_keypair = Secp256k1KeyPair::generate(rng);
        RoochKeyPair::Secp256k1(secp256k1_keypair)
    }

    pub fn from_ed25519_bytes(bytes: &[u8]) -> Result<Self, FastCryptoError> {
        Ok(RoochKeyPair::Ed25519(Ed25519KeyPair::from_bytes(bytes)?))
    }

    pub fn from_secp256k1_bytes(bytes: &[u8]) -> Result<Self, FastCryptoError> {
        Ok(RoochKeyPair::Secp256k1(Secp256k1KeyPair::from_bytes(
            bytes,
        )?))
    }

    pub fn sign(&self, msg: &[u8]) -> Signature {
        Signer::sign(self, msg)
    }

    pub fn sign_secure<T>(&self, value: &T) -> Signature
    where
        T: Serialize,
    {
        Signature::sign_secure(value, self)
    }

    pub fn public(&self) -> PublicKey {
        match self {
            RoochKeyPair::Ed25519(kp) => PublicKey::Ed25519(kp.public().into()),
            RoochKeyPair::Secp256k1(kp) => PublicKey::Secp256k1(kp.public().into()),
        }
    }

    pub fn bitcoin_public_key(&self) -> Result<bitcoin::PublicKey, anyhow::Error> {
        match self {
            RoochKeyPair::Secp256k1(kp) => {
                Ok(bitcoin::PublicKey::from_slice(kp.public().as_bytes())?)
            }
            _ => bail!("Only secp256k1 public key can be converted to bitcoin public key"),
        }
    }

    pub fn private(&self) -> &[u8] {
        match self {
            RoochKeyPair::Ed25519(kp) => kp.as_bytes(),
            RoochKeyPair::Secp256k1(kp) => kp.as_bytes(),
        }
    }

    /// Get the secp256k1 keypair
    pub fn secp256k1_keypair(&self) -> Option<bitcoin::key::Keypair> {
        match self.secp256k1_secret_key() {
            Some(sk) => {
                let keypair = bitcoin::key::Keypair::from_secret_key(
                    &bitcoin::secp256k1::Secp256k1::new(),
                    &sk,
                );
                Some(keypair)
            }
            None => None,
        }
    }

    /// Get the secp256k1 private key
    pub fn secp256k1_secret_key(&self) -> Option<SecretKey> {
        match self {
            RoochKeyPair::Secp256k1(kp) => {
                SecretKey::from_slice(kp.secret.as_bytes()).ok()
                //The bitcoin and fastcrypto dependent on different version secp256k1 library
                //So we cannot directly return the private key
                //Some(&kp.secret.privkey)
            }
            _ => None,
        }
    }

    /// Authentication key is the hash of the public key
    pub fn authentication_key(&self) -> AuthenticationKey {
        self.public().authentication_key()
    }

    pub fn copy(&self) -> Self {
        match self {
            RoochKeyPair::Ed25519(kp) => RoochKeyPair::Ed25519(kp.copy()),
            RoochKeyPair::Secp256k1(kp) => RoochKeyPair::Secp256k1(kp.copy()),
        }
    }

    // Export Private Key method exports a private key in bech32 format
    pub fn export_private_key(&self) -> Result<String, EncodeError> {
        let mut priv_key_bytes =
            Vec::with_capacity(self.public().flag() as usize + self.private().len());
        priv_key_bytes.push(self.public().flag());
        priv_key_bytes.extend_from_slice(self.private());
        // encode hrp and private key bytes using bech32 method
        let bech32_encoded = encode::<Bech32>(*ROOCH_SECRET_KEY_HRP, &priv_key_bytes)?;
        Ok(bech32_encoded)
    }

    /// Encode the public key to multibase format using base58btc encoding
    /// This is a convenience method that delegates to the public key's multibase encoding
    pub fn public_key_to_multibase(&self) -> String {
        self.public().to_multibase()
    }

    /// Encode only the raw public key bytes (without flag) to multibase
    /// This is useful for DID verification methods
    pub fn raw_public_key_to_multibase(&self) -> String {
        self.public().raw_to_multibase()
    }

    /// Get the signature scheme of this keypair
    pub fn scheme(&self) -> SignatureScheme {
        match self {
            RoochKeyPair::Ed25519(_) => SignatureScheme::Ed25519,
            RoochKeyPair::Secp256k1(_) => SignatureScheme::Secp256k1,
        }
    }

    /// Create a keypair from a multibase-encoded public key string and private key bytes
    /// This is primarily used for testing and key reconstruction
    pub fn from_private_key_and_multibase_public_key(
        private_key_bytes: &[u8],
        multibase_public_key: &str,
    ) -> Result<Self, anyhow::Error> {
        // First decode the public key to determine the scheme
        let public_key = PublicKey::from_multibase(multibase_public_key)?;

        match public_key.scheme() {
            SignatureScheme::Ed25519 => {
                let keypair = Ed25519KeyPair::from_bytes(private_key_bytes)
                    .map_err(|e| anyhow!("Failed to create Ed25519 keypair: {}", e))?;

                // Verify that the public key matches
                let derived_public = PublicKey::Ed25519(keypair.public().into());
                if derived_public != public_key {
                    return Err(anyhow!(
                        "Private key does not match the provided public key"
                    ));
                }

                Ok(RoochKeyPair::Ed25519(keypair))
            }
            SignatureScheme::Secp256k1 => {
                let keypair = Secp256k1KeyPair::from_bytes(private_key_bytes)
                    .map_err(|e| anyhow!("Failed to create Secp256k1 keypair: {}", e))?;

                // Verify that the public key matches
                let derived_public = PublicKey::Secp256k1(keypair.public().into());
                if derived_public != public_key {
                    return Err(anyhow!(
                        "Private key does not match the provided public key"
                    ));
                }

                Ok(RoochKeyPair::Secp256k1(keypair))
            }
        }
    }

    /// Generate a complete did:key string from the public key
    /// Format: "did:key:z6Mk..." or "did:key:zQ3s..."
    pub fn to_did_key_string(&self) -> String {
        self.public().to_did_key_string()
    }
}

impl Signer<Signature> for RoochKeyPair {
    fn sign(&self, msg: &[u8]) -> Signature {
        match self {
            RoochKeyPair::Ed25519(kp) => kp.sign(msg),
            RoochKeyPair::Secp256k1(kp) => kp.sign(msg),
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
            RoochKeyPair::Secp256k1(kp) => {
                bytes.extend_from_slice(kp.as_bytes());
            }
        }
        Base64::encode(&bytes[..])
    }

    /// Decode a RoochKeyPair from `flag || privkey` in Base64. The public key is computed directly from the private key bytes.
    fn decode_base64(value: &str) -> Result<Self, FastCryptoError> {
        let bytes = Base64::decode(value)?;
        match SignatureScheme::from_flag_byte(*bytes.first().ok_or(FastCryptoError::InvalidInput)?)
        {
            // Process Rooch key pair by default
            Ok(scheme) => match scheme {
                SignatureScheme::Ed25519 => Ok(RoochKeyPair::Ed25519(Ed25519KeyPair::from_bytes(
                    bytes.get(1..).ok_or(FastCryptoError::InvalidInput)?,
                )?)),
                SignatureScheme::Secp256k1 => {
                    Ok(RoochKeyPair::Secp256k1(Secp256k1KeyPair::from_bytes(
                        bytes.get(1..).ok_or(FastCryptoError::InvalidInput)?,
                    )?))
                }
            },
            _ => Err(FastCryptoError::InvalidInput),
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
pub struct MultibasePublicKey {
    pub verification_method_type: String,
    pub multibase_str: String,
}

#[derive(Debug, Clone, PartialEq, Eq, JsonSchema)]
pub enum PublicKey {
    Ed25519(Ed25519PublicKeyAsBytes),
    Secp256k1(Secp256k1PublicKeyAsBytes),
}

impl AsRef<[u8]> for PublicKey {
    fn as_ref(&self) -> &[u8] {
        match self {
            PublicKey::Ed25519(pk) => &pk.0,
            PublicKey::Secp256k1(pk) => &pk.0,
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

    fn decode_base64(value: &str) -> Result<Self, FastCryptoError> {
        let bytes = Base64::decode(value)?;
        match SignatureScheme::from_flag_byte(*bytes.first().ok_or(FastCryptoError::InvalidInput)?)
        {
            Ok(x) => match x {
                SignatureScheme::Ed25519 => {
                    let pk: Ed25519PublicKey = Ed25519PublicKey::from_bytes(
                        bytes.get(1..).ok_or(FastCryptoError::InvalidInput)?,
                    )?;
                    Ok(PublicKey::Ed25519((&pk).into()))
                }
                SignatureScheme::Secp256k1 => {
                    let pk: Secp256k1PublicKey = Secp256k1PublicKey::from_bytes(
                        bytes.get(1..).ok_or(FastCryptoError::InvalidInput)?,
                    )?;
                    Ok(PublicKey::Secp256k1((&pk).into()))
                }
            },
            Err(_) => Err(FastCryptoError::InvalidInput),
        }
    }
}

impl Serialize for PublicKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = self.to_string();
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
        Self::from_str(s.as_str()).map_err(|e| Error::custom(e.to_string()))
    }
}

impl PublicKey {
    pub fn flag(&self) -> u8 {
        self.scheme().flag()
    }

    pub fn scheme(&self) -> SignatureScheme {
        match self {
            PublicKey::Ed25519(_) => Ed25519RoochSignature::SCHEME,
            PublicKey::Secp256k1(_) => Secp256k1RoochSignature::SCHEME,
        }
    }

    pub fn did_verification_method_type(&self) -> String {
        match self {
            PublicKey::Ed25519(_) => "Ed25519VerificationKey2020".to_string(),
            PublicKey::Secp256k1(_) => "EcdsaSecp256k1VerificationKey2019".to_string(),
        }
    }

    pub fn authentication_key(&self) -> AuthenticationKey {
        self.into()
    }

    pub fn rooch_address(&self) -> Result<RoochAddress, anyhow::Error> {
        let bitcoin_address = self.bitcoin_address()?;
        Ok(bitcoin_address.to_rooch_address())
    }

    pub fn bitcoin_address(&self) -> Result<BitcoinAddress, anyhow::Error> {
        match self {
            PublicKey::Secp256k1(pk) => {
                let xonly_pubkey =
                    bitcoin::XOnlyPublicKey::from(bitcoin::PublicKey::from_slice(&pk.0)?);
                let secp = bitcoin::secp256k1::Secp256k1::verification_only();
                // Rooch BitcoinAddress do not distinguish between network
                Ok(BitcoinAddress::from(bitcoin::Address::p2tr(
                    &secp,
                    xonly_pubkey,
                    None,
                    bitcoin::Network::Bitcoin,
                )))
            }
            _ => bail!("Only secp256k1 public key can be converted to bitcoin address"),
        }
    }

    pub fn xonly_public_key(&self) -> Result<bitcoin::XOnlyPublicKey, anyhow::Error> {
        match self {
            PublicKey::Secp256k1(pk) => {
                let xonly_pubkey =
                    bitcoin::XOnlyPublicKey::from(bitcoin::PublicKey::from_slice(&pk.0)?);
                Ok(xonly_pubkey)
            }
            _ => bail!("Only secp256k1 public key can be converted to xonly public key"),
        }
    }

    pub fn to_hex(&self) -> String {
        hex::encode(self.as_ref())
    }

    pub fn to_hex_literal(&self) -> String {
        format!("0x{}", self.to_hex())
    }

    pub fn from_hex(hex: &str) -> Result<Self, anyhow::Error> {
        let bytes = hex::decode(hex.strip_prefix("0x").unwrap_or(hex))?;
        Self::from_bytes(&bytes)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, anyhow::Error> {
        match SignatureScheme::from_flag_byte(
            *bytes
                .first()
                .ok_or_else(|| anyhow!("Invalid public key length"))?,
        ) {
            Ok(x) => match x {
                SignatureScheme::Ed25519 => {
                    let pk: Ed25519PublicKey = Ed25519PublicKey::from_bytes(
                        bytes
                            .get(1..)
                            .ok_or_else(|| anyhow!("Invalid public key length"))?,
                    )?;
                    Ok(PublicKey::Ed25519((&pk).into()))
                }
                SignatureScheme::Secp256k1 => {
                    let pk: Secp256k1PublicKey = Secp256k1PublicKey::from_bytes(
                        bytes
                            .get(1..)
                            .ok_or_else(|| anyhow!("Invalid public key length"))?,
                    )?;
                    Ok(PublicKey::Secp256k1((&pk).into()))
                }
            },
            Err(e) => Err(anyhow!("Invalid bytes :{}", e)),
        }
    }

    pub fn from_bitcoin_pubkey(pk: &bitcoin::PublicKey) -> Result<Self, anyhow::Error> {
        let bytes = pk.to_bytes();
        let pk = Secp256k1PublicKey::from_bytes(&bytes)?;
        Ok(PublicKey::Secp256k1((&pk).into()))
    }

    /// Encode the public key to multibase format using DID standard multicodec prefixes
    /// Ed25519: 0xed01 prefix -> z6Mk... format
    /// Secp256k1: 0xe701 prefix -> zQ3s... format
    pub fn to_multibase(&self) -> String {
        let mut prefixed_key = match self {
            PublicKey::Ed25519(_) => vec![0xed, 0x01], // Ed25519 multicodec prefix
            PublicKey::Secp256k1(_) => vec![0xe7, 0x01], // Secp256k1 multicodec prefix
        };
        prefixed_key.extend_from_slice(self.raw_public_key_bytes());
        multibase::encode(multibase::Base::Base58Btc, &prefixed_key)
    }

    /// Decode a multibase-encoded public key string with DID standard multicodec prefixes
    /// Supports Ed25519 (z6Mk...) and Secp256k1 (zQ3s...) formats
    pub fn from_multibase(multibase_str: &str) -> Result<Self, anyhow::Error> {
        let (base, decoded_bytes) = multibase::decode(multibase_str)
            .map_err(|e| anyhow!("Failed to decode multibase string: {}", e))?;

        // Verify base58btc encoding (DID standard)
        if base != multibase::Base::Base58Btc {
            return Err(anyhow!("Multibase public key must use base58btc encoding"));
        }

        // Check minimum length (at least 2 bytes for multicodec prefix)
        if decoded_bytes.len() < 2 {
            return Err(anyhow!("Invalid multibase public key: too short"));
        }

        // Extract multicodec prefix
        let first_byte = decoded_bytes[0];
        let second_byte = decoded_bytes[1];

        match (first_byte, second_byte) {
            (0xed, 0x01) => {
                // Ed25519 multicodec
                if decoded_bytes.len() != 34 {
                    // 2 bytes prefix + 32 bytes key
                    return Err(anyhow!("Invalid Ed25519 multibase public key length"));
                }
                let raw_key = &decoded_bytes[2..];
                let pk = Ed25519PublicKey::from_bytes(raw_key)?;
                Ok(PublicKey::Ed25519((&pk).into()))
            }
            (0xe7, 0x01) => {
                // Secp256k1 multicodec
                if decoded_bytes.len() != 35 {
                    // 2 bytes prefix + 33 bytes key
                    return Err(anyhow!("Invalid Secp256k1 multibase public key length"));
                }
                let raw_key = &decoded_bytes[2..];
                let pk = Secp256k1PublicKey::from_bytes(raw_key)?;
                Ok(PublicKey::Secp256k1((&pk).into()))
            }
            _ => Err(anyhow!(
                "Unsupported multicodec prefix: 0x{:02x}{:02x}",
                first_byte,
                second_byte
            )),
        }
    }

    /// Get the raw public key bytes without the flag
    /// For Ed25519: 32 bytes
    /// For Secp256k1: 33 bytes (compressed)
    pub fn raw_public_key_bytes(&self) -> &[u8] {
        self.as_ref()
    }

    /// Encode only the raw public key bytes (without flag) to multibase
    /// This is useful for DID verification methods
    pub fn raw_to_multibase(&self) -> String {
        multibase::encode(multibase::Base::Base58Btc, self.raw_public_key_bytes())
    }

    /// Decode raw public key bytes from multibase and create PublicKey with specified scheme
    pub fn from_raw_multibase(
        multibase_str: &str,
        scheme: SignatureScheme,
    ) -> Result<Self, anyhow::Error> {
        let (base, decoded_bytes) = multibase::decode(multibase_str)
            .map_err(|e| anyhow!("Failed to decode multibase string: {}", e))?;

        // Verify the encoding is supported
        match base {
            multibase::Base::Base58Btc
            | multibase::Base::Base64Pad
            | multibase::Base::Base16Lower => {
                // These are supported encodings
            }
            _ => {
                return Err(anyhow!("Unsupported multibase encoding: {:?}", base));
            }
        }

        // Validate key length based on scheme
        match scheme {
            SignatureScheme::Ed25519 => {
                if decoded_bytes.len() != 32 {
                    return Err(anyhow!(
                        "Invalid Ed25519 public key length: expected 32 bytes, got {}",
                        decoded_bytes.len()
                    ));
                }
                let pk = Ed25519PublicKey::from_bytes(&decoded_bytes)?;
                Ok(PublicKey::Ed25519((&pk).into()))
            }
            SignatureScheme::Secp256k1 => {
                if decoded_bytes.len() != 33 {
                    return Err(anyhow!(
                        "Invalid Secp256k1 public key length: expected 33 bytes, got {}",
                        decoded_bytes.len()
                    ));
                }
                let pk = Secp256k1PublicKey::from_bytes(&decoded_bytes)?;
                Ok(PublicKey::Secp256k1((&pk).into()))
            }
        }
    }

    pub fn to_multibase_public_key(&self) -> MultibasePublicKey {
        MultibasePublicKey {
            verification_method_type: self.did_verification_method_type(),
            multibase_str: self.raw_to_multibase(),
        }
    }

    /// Generate a complete did:key string from the public key
    /// Format: "did:key:" + multibase_identifier
    pub fn to_did_key_string(&self) -> String {
        format!("did:key:{}", self.to_multibase())
    }

    /// Parse a complete did:key string and extract the public key
    /// Format: "did:key:z6Mk..." or "did:key:zQ3s..."
    pub fn from_did_key_string(did_key_string: &str) -> Result<Self, anyhow::Error> {
        if !did_key_string.starts_with("did:key:") {
            return Err(anyhow!("Invalid did:key string format"));
        }

        let identifier = &did_key_string[8..]; // Skip "did:key:"
        Self::from_multibase(identifier)
    }

    /// Get the multicodec prefix for this public key type
    /// Returns the 2-byte multicodec prefix used in did:key identifiers
    pub fn multicodec_prefix(&self) -> Vec<u8> {
        match self {
            PublicKey::Ed25519(_) => vec![0xed, 0x01],
            PublicKey::Secp256k1(_) => vec![0xe7, 0x01],
        }
    }
}

impl std::fmt::Display for PublicKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_hex_literal())
    }
}

impl FromStr for PublicKey {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let pk = Self::from_hex(s).map_err(|e| anyhow!("{}", e.to_string()))?;
        Ok(pk)
    }
}

pub trait RoochPublicKey: VerifyingKey {
    const SIGNATURE_SCHEME: SignatureScheme;
}

impl RoochPublicKey for Ed25519PublicKey {
    const SIGNATURE_SCHEME: SignatureScheme = SignatureScheme::Ed25519;
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
    const SCHEME: SignatureScheme = Self::PubKey::SIGNATURE_SCHEME;

    fn get_verification_inputs(&self) -> RoochResult<(Self::Sig, Self::PubKey)> {
        // Is this signature emitted by the expected author?
        let bytes = self.public_key_bytes();
        let pk = Self::PubKey::from_bytes(bytes)
            .map_err(|_| RoochError::KeyConversionError("Invalid public key".to_owned()))?;

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

// Enums for signature auth validator signatures
#[enum_dispatch]
#[derive(Clone, JsonSchema, Debug, PartialEq, Eq, Hash)]
pub enum Signature {
    Ed25519RoochSignature,
    Secp256k1RoochSignature,
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
    /// Sign the message with the secret key.
    /// Different SignatureScheme will have different hash function.
    /// The Secp256k1 will use sha256 to hash the message.
    pub fn sign(msg: &[u8], secret: &dyn Signer<Signature>) -> Self {
        Signer::sign(secret, msg)
    }

    /// Sign the message with bcs serialization and use Blake2b256 to hash the message.
    pub fn sign_secure<T>(value: &T, secret: &dyn Signer<Signature>) -> Self
    where
        T: Serialize,
    {
        let mut hasher = DefaultHash::default();
        hasher.update(bcs::to_bytes(&value).expect("Message serialization should not fail"));
        Signer::sign(secret, &hasher.finalize().digest)
    }

    /// Parse [enum CompressedSignature] from trait Signature `flag || sig || pk`.
    /// This is useful for the MultiSig to combine partial signature into a MultiSig public key.
    pub fn to_compressed(&self) -> Result<CompressedSignature, RoochError> {
        match self {
            Signature::Ed25519RoochSignature(sig) => Ok(CompressedSignature::Ed25519(
                (&Ed25519Signature::from_bytes(sig.signature_bytes()).map_err(|_| {
                    RoochError::InvalidSignature {
                        error: "Cannot parse sig".to_owned(),
                    }
                })?)
                    .into(),
            )),
            Signature::Secp256k1RoochSignature(sig) => Ok(CompressedSignature::Secp256k1(
                (&Secp256k1Signature::from_bytes(sig.signature_bytes()).map_err(|_| {
                    RoochError::InvalidSignature {
                        error: "Cannot parse sig".to_owned(),
                    }
                })?)
                    .into(),
            )),
        }
    }

    /// Parse [struct PublicKey] from trait RoochSignature `flag || sig || pk`.
    /// This is useful for the MultiSig to construct the bitmap in [struct MultiPublicKey].
    pub fn to_public_key(&self) -> Result<PublicKey, RoochError> {
        let bytes = self.public_key_bytes();
        // Process Rooch signature by default
        Ok(PublicKey::Ed25519(
            (&Ed25519PublicKey::from_bytes(bytes)
                .map_err(|_| RoochError::KeyConversionError("Cannot parse pk".to_owned()))?)
                .into(),
        ))
    }
}

impl AsRef<[u8]> for Signature {
    fn as_ref(&self) -> &[u8] {
        match self {
            Signature::Ed25519RoochSignature(sig) => sig.as_ref(),
            Signature::Secp256k1RoochSignature(sig) => sig.as_ref(),
        }
    }
}

impl ToFromBytes for Signature {
    fn from_bytes(bytes: &[u8]) -> Result<Self, FastCryptoError> {
        match bytes.first() {
            Some(x) => {
                if x == &Ed25519RoochSignature::SCHEME.flag() {
                    Ok(<Ed25519RoochSignature as ToFromBytes>::from_bytes(bytes)?.into())
                } else if x == &Secp256k1RoochSignature::SCHEME.flag() {
                    Ok(<Secp256k1RoochSignature as ToFromBytes>::from_bytes(bytes)?.into())
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
    Secp256k1(Secp256k1SignatureAsBytes),
}

impl AsRef<[u8]> for CompressedSignature {
    fn as_ref(&self) -> &[u8] {
        match self {
            CompressedSignature::Ed25519(sig) => &sig.0,
            CompressedSignature::Secp256k1(sig) => &sig.0,
        }
    }
}

#[enum_dispatch(Signature)]
pub trait RoochSignature: Sized + ToFromBytes {
    fn signature_bytes(&self) -> &[u8];
    fn public_key_bytes(&self) -> &[u8];
    fn scheme(&self) -> SignatureScheme;

    fn verify(&self, value: &[u8]) -> RoochResult<()>;
    fn verify_secure<T>(&self, value: &T) -> RoochResult<()>
    where
        T: Serialize,
    {
        let mut hasher = DefaultHash::default();
        hasher.update(bcs::to_bytes(&value).expect("Message serialization should not fail"));
        let digest = hasher.finalize().digest;
        self.verify(digest.as_ref())
    }
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

    fn scheme(&self) -> SignatureScheme {
        S::PubKey::SIGNATURE_SCHEME
    }

    fn verify(&self, value: &[u8]) -> RoochResult<()> {
        let (sig, pk) = &self.get_verification_inputs()?;
        pk.verify(value, sig)
            .map_err(|e| RoochError::InvalidSignature {
                error: format!("Fail to verify user sig {}", e),
            })
    }
}

//
// Ed25519 Rooch Signature port
//
#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash, AsRef)]
#[as_ref(forward)]
pub struct Ed25519RoochSignature(
    #[schemars(with = "Base64")]
    // Replace the problematic serde_as attribute with a simpler one
    #[serde_as(as = "serde_with::base64::Base64")]
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
// Secp256k1 Signature port
//
#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash, AsRef)]
#[as_ref(forward)]
pub struct Secp256k1RoochSignature(
    #[schemars(with = "Base64")]
    // Replace the problematic serde_as attribute with a simpler one
    #[serde_as(as = "serde_with::base64::Base64")]
    [u8; Secp256k1PublicKey::LENGTH + Secp256k1Signature::LENGTH + 1],
);

impl RoochSignatureInner for Secp256k1RoochSignature {
    type Sig = Secp256k1Signature;
    type PubKey = Secp256k1PublicKey;
    type KeyPair = Secp256k1KeyPair;
    const LENGTH: usize = Secp256k1PublicKey::LENGTH + Secp256k1Signature::LENGTH + 1;
}

impl RoochPublicKey for Secp256k1PublicKey {
    const SIGNATURE_SCHEME: SignatureScheme = SignatureScheme::Secp256k1;
}

impl ToFromBytes for Secp256k1RoochSignature {
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
        Secp256k1RoochSignature::new(self, msg).into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bitcoin::network;
    use ethers::utils::keccak256;
    use fastcrypto::{
        secp256k1::{Secp256k1KeyPair, Secp256k1PrivateKey},
        traits::{KeyPair, ToFromBytes},
    };

    // this test ensure the Rooch public key to address keep the same as the old version
    // we should also keep the Rooch public key to address algorithm the same as the move version
    #[test]
    fn test_rooch_public_key_to_address() {
        let private_key = Secp256k1PrivateKey::from_bytes(&[0xcd; 32]).unwrap();
        let rooch_keypair = RoochKeyPair::Secp256k1(Secp256k1KeyPair::from(private_key));
        let btc_address = rooch_keypair.public().bitcoin_address().unwrap();
        let btc_address_str = btc_address
            .format(network::Network::Bitcoin.to_num())
            .unwrap();
        //println!("{}", btc_address_str);
        assert_eq!(
            btc_address_str,
            "bc1pesylj5fdhxktcnl34t9r9ple0anatqwhanhtsh33szpehqhgtagqcj0rk5"
        );
        let rooch_address = btc_address.to_rooch_address();
        //println!("hex:{}, bech32:{}", rooch_address.to_hex(), rooch_address.to_bech32());
        assert_eq!(
            rooch_address.to_hex_literal(),
            "0x0c9fae081aec16249e3c9c94e09170eb7222767e0b2db04e9c7144d6e5a4e804"
        );
        assert_eq!(
            rooch_address.to_bech32(),
            "rooch1pj06uzq6astzf83unj2wpytsadezyan7pvkmqn5uw9zddedyaqzq4090g0"
        );
    }

    // this test is to ensure that the ECDSA recoverable algorithm works for Ethereum public key to address
    #[test]
    fn test_ethereum_public_key_to_address() {
        let private_key = Secp256k1PrivateKey::from_bytes(&[1u8; 32]).unwrap(); // use 1u8.
        let keypair: Secp256k1KeyPair = private_key.into();
        let public_key = keypair.public();
        let uncompressed = public_key.pubkey.serialize_uncompressed();
        let uncompressed_64 = uncompressed[1..65].to_vec();
        let hashed = keccak256(uncompressed_64);
        let address_bytes = hashed[12..32].to_vec();
        let address_str = format!("0x{}", hex::encode(address_bytes)); // Include "0x" prefix
        let expected_address = "0x1a642f0e3c3af545e7acbd38b07251b3990914f1";
        assert_eq!(address_str, expected_address);
    }

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct SignData {
        value: Vec<u8>,
    }

    #[test]
    fn test_secp256k1_signature() {
        let kp = RoochKeyPair::generate_secp256k1();
        let message = b"hello world";
        let signature = kp.sign(message);
        assert!(signature.verify(message).is_ok());

        let value = SignData {
            value: message.to_vec(),
        };
        let signature = kp.sign_secure(&value);
        assert!(signature.verify_secure(&value).is_ok());
    }

    #[test]
    fn test_ed25519_signature() {
        let kp = RoochKeyPair::generate_ed25519();
        let message = b"hello world";
        let signature = kp.sign(message);
        assert!(signature.verify(message).is_ok());

        let value = SignData {
            value: message.to_vec(),
        };
        let signature = kp.sign_secure(&value);
        assert!(signature.verify_secure(&value).is_ok());
    }

    #[test]
    fn test_multibase_public_key_encoding_decoding() {
        // Test Ed25519 public key
        let ed25519_kp = RoochKeyPair::generate_ed25519();
        let ed25519_public = ed25519_kp.public();

        // Test full public key encoding (with flag)
        let ed25519_multibase = ed25519_public.to_multibase();
        assert!(ed25519_multibase.starts_with('z')); // base58btc prefix

        let decoded_ed25519 = PublicKey::from_multibase(&ed25519_multibase).unwrap();
        assert_eq!(ed25519_public, decoded_ed25519);

        // Test raw public key encoding (without flag)
        let ed25519_raw_multibase = ed25519_public.raw_to_multibase();
        assert!(ed25519_raw_multibase.starts_with('z')); // base58btc prefix

        let decoded_ed25519_raw =
            PublicKey::from_raw_multibase(&ed25519_raw_multibase, SignatureScheme::Ed25519)
                .unwrap();
        assert_eq!(ed25519_public, decoded_ed25519_raw);

        // Test Secp256k1 public key
        let secp256k1_kp = RoochKeyPair::generate_secp256k1();
        let secp256k1_public = secp256k1_kp.public();

        // Test full public key encoding (with flag)
        let secp256k1_multibase = secp256k1_public.to_multibase();
        assert!(secp256k1_multibase.starts_with('z')); // base58btc prefix

        let decoded_secp256k1 = PublicKey::from_multibase(&secp256k1_multibase).unwrap();
        assert_eq!(secp256k1_public, decoded_secp256k1);

        // Test raw public key encoding (without flag)
        let secp256k1_raw_multibase = secp256k1_public.raw_to_multibase();
        assert!(secp256k1_raw_multibase.starts_with('z')); // base58btc prefix

        let decoded_secp256k1_raw =
            PublicKey::from_raw_multibase(&secp256k1_raw_multibase, SignatureScheme::Secp256k1)
                .unwrap();
        assert_eq!(secp256k1_public, decoded_secp256k1_raw);
    }

    #[test]
    fn test_keypair_multibase_methods() {
        // Test Ed25519 keypair
        let ed25519_kp = RoochKeyPair::generate_ed25519();

        let public_multibase = ed25519_kp.public_key_to_multibase();
        let raw_public_multibase = ed25519_kp.raw_public_key_to_multibase();

        assert!(public_multibase.starts_with('z'));
        assert!(raw_public_multibase.starts_with('z'));
        assert_ne!(public_multibase, raw_public_multibase); // Should be different due to flag

        // Test Secp256k1 keypair
        let secp256k1_kp = RoochKeyPair::generate_secp256k1();

        let public_multibase = secp256k1_kp.public_key_to_multibase();
        let raw_public_multibase = secp256k1_kp.raw_public_key_to_multibase();

        assert!(public_multibase.starts_with('z'));
        assert!(raw_public_multibase.starts_with('z'));
        assert_ne!(public_multibase, raw_public_multibase); // Should be different due to flag
    }

    #[test]
    fn test_multibase_error_handling() {
        // Test invalid multibase string
        let result = PublicKey::from_multibase("invalid_multibase");
        assert!(result.is_err());

        // Test empty multibase string
        let result = PublicKey::from_multibase("");
        assert!(result.is_err());

        // Test wrong key length for Ed25519
        let result = PublicKey::from_raw_multibase("z1234", SignatureScheme::Ed25519);
        assert!(result.is_err());

        // Test wrong key length for Secp256k1
        let result = PublicKey::from_raw_multibase("z1234", SignatureScheme::Secp256k1);
        assert!(result.is_err());
    }

    #[test]
    fn test_multibase_compatibility_with_move() {
        // This test ensures our multibase encoding is compatible with the Move implementation
        // Generate a test key
        let kp = RoochKeyPair::generate_ed25519();
        let public_key = kp.public();

        // Get raw bytes (32 bytes for Ed25519)
        let raw_bytes = public_key.raw_public_key_bytes();
        assert_eq!(raw_bytes.len(), 32);

        // Encode to multibase
        let multibase_str = public_key.raw_to_multibase();

        // Should start with 'z' (base58btc prefix)
        assert!(multibase_str.starts_with('z'));

        // Decode back and verify
        let decoded =
            PublicKey::from_raw_multibase(&multibase_str, SignatureScheme::Ed25519).unwrap();
        assert_eq!(public_key, decoded);

        // Test with Secp256k1
        let secp_kp = RoochKeyPair::generate_secp256k1();
        let secp_public = secp_kp.public();

        // Get raw bytes (33 bytes for Secp256k1 compressed)
        let secp_raw_bytes = secp_public.raw_public_key_bytes();
        assert_eq!(secp_raw_bytes.len(), 33);

        // Encode to multibase
        let secp_multibase_str = secp_public.raw_to_multibase();

        // Should start with 'z' (base58btc prefix)
        assert!(secp_multibase_str.starts_with('z'));

        // Decode back and verify
        let secp_decoded =
            PublicKey::from_raw_multibase(&secp_multibase_str, SignatureScheme::Secp256k1).unwrap();
        assert_eq!(secp_public, secp_decoded);
    }

    #[test]
    fn test_did_key_multicodec_encoding() {
        // Test Ed25519 with DID standard multicodec
        let ed25519_kp = RoochKeyPair::generate_ed25519();
        let ed25519_public = ed25519_kp.public();

        // Test multibase encoding with multicodec prefix
        let ed25519_multibase = ed25519_public.to_multibase();
        assert!(ed25519_multibase.starts_with("z6Mk")); // Ed25519 multicodec results in z6Mk prefix

        // Test round-trip
        let decoded_ed25519 = PublicKey::from_multibase(&ed25519_multibase).unwrap();
        assert_eq!(ed25519_public, decoded_ed25519);

        // Test did:key string generation
        let ed25519_did_key = ed25519_public.to_did_key_string();
        assert!(ed25519_did_key.starts_with("did:key:z6Mk"));

        // Test did:key string parsing
        let parsed_ed25519 = PublicKey::from_did_key_string(&ed25519_did_key).unwrap();
        assert_eq!(ed25519_public, parsed_ed25519);

        // Test Secp256k1 with DID standard multicodec
        let secp256k1_kp = RoochKeyPair::generate_secp256k1();
        let secp256k1_public = secp256k1_kp.public();

        // Test multibase encoding with multicodec prefix
        let secp256k1_multibase = secp256k1_public.to_multibase();
        assert!(secp256k1_multibase.starts_with("zQ3s")); // Secp256k1 multicodec results in zQ3s prefix

        // Test round-trip
        let decoded_secp256k1 = PublicKey::from_multibase(&secp256k1_multibase).unwrap();
        assert_eq!(secp256k1_public, decoded_secp256k1);

        // Test did:key string generation
        let secp256k1_did_key = secp256k1_public.to_did_key_string();
        assert!(secp256k1_did_key.starts_with("did:key:zQ3s"));

        // Test did:key string parsing
        let parsed_secp256k1 = PublicKey::from_did_key_string(&secp256k1_did_key).unwrap();
        assert_eq!(secp256k1_public, parsed_secp256k1);
    }

    #[test]
    fn test_multicodec_prefix() {
        let ed25519_kp = RoochKeyPair::generate_ed25519();
        let ed25519_public = ed25519_kp.public();
        assert_eq!(ed25519_public.multicodec_prefix(), vec![0xed, 0x01]);

        let secp256k1_kp = RoochKeyPair::generate_secp256k1();
        let secp256k1_public = secp256k1_kp.public();
        assert_eq!(secp256k1_public.multicodec_prefix(), vec![0xe7, 0x01]);
    }

    #[test]
    fn test_keypair_did_key_methods() {
        let ed25519_kp = RoochKeyPair::generate_ed25519();
        let ed25519_did_key = ed25519_kp.to_did_key_string();
        assert!(ed25519_did_key.starts_with("did:key:z6Mk"));

        let secp256k1_kp = RoochKeyPair::generate_secp256k1();
        let secp256k1_did_key = secp256k1_kp.to_did_key_string();
        assert!(secp256k1_did_key.starts_with("did:key:zQ3s"));
    }
}

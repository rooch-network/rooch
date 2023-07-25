// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::crypto::{BuiltinScheme, Signature};
use anyhow::Result;

#[cfg(any(test, feature = "fuzzing"))]
use super::ethereum::EthereumTransaction;
#[cfg(any(test, feature = "fuzzing"))]
use ethers::types::U256;
#[cfg(any(test, feature = "fuzzing"))]
use fastcrypto::ed25519::Ed25519KeyPair;
#[cfg(any(test, feature = "fuzzing"))]
use fastcrypto::secp256k1::schnorr::SchnorrKeyPair;
#[cfg(any(test, feature = "fuzzing"))]
use fastcrypto::traits::KeyPair;
#[cfg(any(test, feature = "fuzzing"))]
use proptest::{collection::vec, prelude::*};
use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};

/// A `Authenticator` is an an abstraction of a account authenticator.
/// It is a part of `AccountAbstraction`

pub trait BuiltinAuthenticator {
    fn scheme(&self) -> BuiltinScheme;
    fn payload(&self) -> Vec<u8>;
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Ed25519Authenticator {
    pub signature: Signature,
}

impl BuiltinAuthenticator for Ed25519Authenticator {
    fn scheme(&self) -> BuiltinScheme {
        BuiltinScheme::Ed25519
    }
    fn payload(&self) -> Vec<u8> {
        self.signature.as_ref().to_vec()
    }
}
#[cfg(any(test, feature = "fuzzing"))]
impl Arbitrary for Ed25519Authenticator {
    type Parameters = ();
    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        arb_ed25519_authenticator().boxed()
    }
    type Strategy = BoxedStrategy<Self>;
}

#[cfg(any(test, feature = "fuzzing"))]
prop_compose! {
    fn arb_ed25519_authenticator()(
        seed in any::<u64>(),
        message in vec(any::<u8>(), 1..1000)
    ) -> Ed25519Authenticator {
        let mut rng = StdRng::seed_from_u64(seed);
        let ed25519_keypair: Ed25519KeyPair = Ed25519KeyPair::generate(&mut rng);
        Ed25519Authenticator {
            signature: Signature::new_hashed(&message, &ed25519_keypair)
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SchnorrAuthenticator {
    pub signature: Signature,
}

impl BuiltinAuthenticator for SchnorrAuthenticator {
    fn scheme(&self) -> BuiltinScheme {
        BuiltinScheme::Schnorr
    }
    fn payload(&self) -> Vec<u8> {
        self.signature.as_ref().to_vec()
    }
}
#[cfg(any(test, feature = "fuzzing"))]
impl Arbitrary for SchnorrAuthenticator {
    type Parameters = ();
    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        arb_schnorr_authenticator().boxed()
    }
    type Strategy = BoxedStrategy<Self>;
}

#[cfg(any(test, feature = "fuzzing"))]
prop_compose! {
    fn arb_schnorr_authenticator()(
        seed in any::<u64>(),
        message in vec(any::<u8>(), 32)
    ) -> SchnorrAuthenticator {
        let mut rng = StdRng::seed_from_u64(seed);
        let kp = SchnorrKeyPair::generate(&mut rng);
        SchnorrAuthenticator {
            signature: Signature::new_hashed(&message, &kp)
        }
    }
}

// TODO: MultiEd25519

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EcdsaAuthenticator {
    pub signature: Signature,
}

impl BuiltinAuthenticator for EcdsaAuthenticator {
    fn scheme(&self) -> BuiltinScheme {
        BuiltinScheme::Ecdsa
    }
    fn payload(&self) -> Vec<u8> {
        self.signature.as_ref().to_vec()
    }
}

#[cfg(any(test, feature = "fuzzing"))]
impl Arbitrary for EcdsaAuthenticator {
    type Parameters = ();
    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        arb_ecdsa_authenticator().boxed()
    }
    type Strategy = BoxedStrategy<Self>;
}

#[cfg(any(test, feature = "fuzzing"))]
prop_compose! {
    fn arb_ecdsa_authenticator()(
     r in vec(any::<u64>(), 4..=4).prop_map(|v| U256(v.try_into().unwrap())),
     s in vec(any::<u64>(), 4..=4).prop_map(|v| U256(v.try_into().unwrap())),
     // Although v is an u64 type, it is actually an u8 value.
     v in any::<u8>().prop_filter("Valid v value", |&v| v == 27 || v == 28),
    ) -> EcdsaAuthenticator {
        let dummy_tx = ethers::core::types::Transaction {
            r: r,
            s: s,
            v: v.into(),
            hash: H256::zero(),
            nonce: U256::zero(),
            block_hash: None,
            block_number: None,
            transaction_index: None,
            from: Address::zero(),
            to: None,
            value: U256::zero(),
            gas_price: None,
            gas: U256::zero(),
            input: Bytes::new().into(),
            transaction_type: None, // For EIP-2718
            access_list: None, // For EIP-2930
            max_priority_fee_per_gas: None, // For EIP-1559
            max_fee_per_gas: None, // For EIP-1559
            chain_id: None, // For EIP-1559
            other: Default::default(), // Captures unknown fields (if any)
        };
        println!("dummy_tx.input: {}", dummy_tx.input);
        let message = dummy_tx.input.clone();
        let eth_tx = EthereumTransaction(dummy_tx);
        let sig = EthereumTransaction::convert_eth_signature_to_recoverable_secp256k1_signature(&eth_tx, message).unwrap();
        EcdsaAuthenticator {
            signature: sig
        }
    }
}

impl<T> From<T> for Authenticator
where
    T: BuiltinAuthenticator,
{
    fn from(value: T) -> Self {
        let scheme = value.scheme() as u64;
        let payload = value.payload();
        Authenticator { scheme, payload }
    }
}

impl From<Signature> for Authenticator {
    fn from(sign: Signature) -> Self {
        match sign.to_public_key().unwrap().scheme() {
            BuiltinScheme::Ed25519 => Authenticator::ed25519(sign),
            BuiltinScheme::Ecdsa => Authenticator::ecdsa(sign),
            BuiltinScheme::MultiEd25519 => todo!(),
            BuiltinScheme::Schnorr => Authenticator::schnorr(sign),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Authenticator {
    pub scheme: u64,
    pub payload: Vec<u8>,
}

impl Authenticator {
    /// Unique identifier for the signature scheme
    pub fn scheme(&self) -> u64 {
        self.scheme
    }

    /// Create a single-signature ed25519 authenticator
    pub fn ed25519(signature: Signature) -> Self {
        Ed25519Authenticator { signature }.into()
    }

    /// Create a single-signature ecdsa authenticator
    pub fn ecdsa(signature: Signature) -> Self {
        EcdsaAuthenticator { signature }.into()
    }

    /// Create a single-signature schnorr authenticator
    pub fn schnorr(signature: Signature) -> Self {
        SchnorrAuthenticator { signature }.into()
    }

    /// Create a custom authenticator
    pub fn new(scheme: u64, payload: Vec<u8>) -> Self {
        Self { scheme, payload }
    }
}

impl FromStr for Authenticator {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.strip_prefix("0x").unwrap_or(s);
        let bytes = hex::decode(s)?;
        bcs::from_bytes(bytes.as_slice()).map_err(Into::into)
    }
}

impl fmt::Display for Authenticator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Authenticator[scheme id: {:?}, payload: {}]",
            self.scheme(),
            hex::encode(&self.payload),
        )
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_ecdsa_authenticator_serialize_deserialize(authenticator in any::<super::EcdsaAuthenticator>()) {
            let serialized = serde_json::to_string(&authenticator).unwrap();
            let deserialized: super:: EcdsaAuthenticator = serde_json::from_str(&serialized).unwrap();
            assert_eq!(authenticator.signature, deserialized.signature);
        }

        #[test]
        fn test_ed25519_authenticator_serialize_deserialize(authenticator in any::<super::Ed25519Authenticator>()) {
            let serialized = serde_json::to_string(&authenticator).unwrap();
            let deserialized: super::Ed25519Authenticator = serde_json::from_str(&serialized).unwrap();
            assert_eq!(authenticator.signature, deserialized.signature);
        }

        #[test]
        fn test_schnorr_authenticator_serialize_deserialize(authenticator in any::<super::SchnorrAuthenticator>()) {
            let serialized = serde_json::to_string(&authenticator).unwrap();
            let deserialized: super::SchnorrAuthenticator = serde_json::from_str(&serialized).unwrap();
            assert_eq!(authenticator.signature, deserialized.signature);
        }
    }
}

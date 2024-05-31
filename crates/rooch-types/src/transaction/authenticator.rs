// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
#[cfg(any(test, feature = "fuzzing"))]
use fastcrypto::ed25519::Ed25519KeyPair;
#[cfg(any(test, feature = "fuzzing"))]
use fastcrypto::traits::KeyPair;
#[cfg(any(test, feature = "fuzzing"))]
use proptest::{collection::vec, prelude::*};
#[cfg(any(test, feature = "fuzzing"))]
use rand::{rngs::StdRng, SeedableRng};
use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};

use crate::{
    crypto::{RoochKeyPair, Signature, SignatureScheme},
    framework::{
        auth_payload::{AuthPayload, SignData},
        auth_validator::BuiltinAuthValidator,
    },
    rooch_network::{BuiltinChainID, RoochNetwork},
};

use super::RoochTransactionData;

/// A `Authenticator` is an abstraction of a account authenticator.
/// It is a part of `AccountAbstraction`

pub trait BuiltinAuthenticator {
    fn auth_validator_id(&self) -> u64;
    fn payload(&self) -> Vec<u8>;
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RoochAuthenticator {
    pub signature: Signature,
}

impl BuiltinAuthenticator for RoochAuthenticator {
    fn auth_validator_id(&self) -> u64 {
        BuiltinAuthValidator::Rooch.flag().into()
    }
    fn payload(&self) -> Vec<u8> {
        self.signature.as_ref().to_vec()
    }
}
#[cfg(any(test, feature = "fuzzing"))]
impl Arbitrary for RoochAuthenticator {
    type Parameters = ();
    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        arb_rooch_authenticator().boxed()
    }
    type Strategy = BoxedStrategy<Self>;
}

#[cfg(any(test, feature = "fuzzing"))]
prop_compose! {
    fn arb_rooch_authenticator()(
        seed in any::<u64>(),
        message in vec(any::<u8>(), 1..1000)
    ) -> RoochAuthenticator {
        let mut rng = StdRng::seed_from_u64(seed);
        let ed25519_keypair: Ed25519KeyPair = Ed25519KeyPair::generate(&mut rng);
        RoochAuthenticator {
            signature: Signature::sign(&message, &ed25519_keypair)
        }
    }
}

impl<T> From<T> for Authenticator
where
    T: BuiltinAuthenticator,
{
    fn from(value: T) -> Self {
        let auth_validator_id = value.auth_validator_id();
        let payload = value.payload();
        Authenticator {
            auth_validator_id,
            payload,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BitcoinAuthenticator {
    pub payload: AuthPayload,
}

impl BuiltinAuthenticator for BitcoinAuthenticator {
    fn auth_validator_id(&self) -> u64 {
        BuiltinAuthValidator::Bitcoin.flag().into()
    }
    fn payload(&self) -> Vec<u8> {
        bcs::to_bytes(&self.payload).expect("Serialize BitcoinAuthenticator should success")
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Authenticator {
    pub auth_validator_id: u64,
    pub payload: Vec<u8>,
}

impl Authenticator {
    /// Unique identifier for the signature of auth validator id
    pub fn auth_validator_id(&self) -> u64 {
        self.auth_validator_id
    }

    pub fn genesis() -> Self {
        Self {
            auth_validator_id: BuiltinAuthValidator::Rooch.flag().into(),
            payload: vec![],
        }
    }

    /// Create a rooch authenticator for session key
    pub fn rooch(kp: &RoochKeyPair, tx_data: &RoochTransactionData) -> Self {
        debug_assert_eq!(kp.public().scheme(), SignatureScheme::Ed25519);
        let data_hash = tx_data.tx_hash();
        let signature = kp.sign(data_hash.as_bytes());
        RoochAuthenticator { signature }.into()
    }

    /// Create a bitcoin authenticator for RoochTransaction
    /// We simulate the Bitcoin Wallet message signature
    pub fn bitcoin(kp: &RoochKeyPair, tx_data: &RoochTransactionData) -> Self {
        debug_assert_eq!(kp.public().scheme(), SignatureScheme::Secp256k1);
        let sign_data = SignData::new(tx_data);
        //The Bitcoin wallet uses sha2_256 twice, the Secp256k1 sign method will hash the data again
        let data_hash = sign_data.data_hash();
        let signature = kp.sign(data_hash.as_bytes());
        let bitcoin_address = kp
            .public()
            .bitcoin_address()
            .expect("Generate bitcoin address should success");
        //TODO handle custom network
        let rooch_network = RoochNetwork::from(
            BuiltinChainID::try_from(tx_data.chain_id).unwrap_or(BuiltinChainID::default()),
        );
        let bitcoin_address_str = bitcoin_address
            .format(rooch_network.genesis_config.bitcoin_network)
            .expect("format bitcoin address should success");
        let auth_payload = AuthPayload::new(sign_data, signature, bitcoin_address_str);
        BitcoinAuthenticator {
            payload: auth_payload,
        }
        .into()
    }

    /// Create a custom authenticator
    pub fn new(auth_validator_id: u64, payload: Vec<u8>) -> Self {
        Self {
            auth_validator_id,
            payload,
        }
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
            "Authenticator[auth validator id: {:?}, payload: {}]",
            self.auth_validator_id(),
            hex::encode(&self.payload),
        )
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_rooch_authenticator_serialize_deserialize(authenticator in any::<super::RoochAuthenticator>()) {
            let serialized = serde_json::to_string(&authenticator).unwrap();
            let deserialized: super::RoochAuthenticator = serde_json::from_str(&serialized).unwrap();
            assert_eq!(authenticator.signature, deserialized.signature);
        }
    }
}

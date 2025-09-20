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
        auth_payload::{AuthPayload, MultisignAuthPayload, SignData},
        auth_validator::BuiltinAuthValidator,
    },
    rooch_network::{BuiltinChainID, RoochNetwork},
};
use moveos_types::h256::sha2_256_of;

use super::RoochTransactionData;

// The size of the signature data.
pub const AUTH_PAYLOAD_SIZE: u64 = 219;

/// A `Authenticator` is an abstraction of a account authenticator.
/// It is a part of `AccountAbstraction`

pub trait BuiltinAuthenticator {
    fn auth_validator_id(&self) -> u64;
    fn payload(&self) -> Vec<u8>;
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SessionAuthenticator {
    pub signature: Signature,
}

impl SessionAuthenticator {
    pub fn new(signature: Signature) -> Self {
        Self { signature }
    }

    pub fn sign(kp: &RoochKeyPair, tx_data: &RoochTransactionData) -> Self {
        let data_hash = tx_data.tx_hash();
        let signature = kp.sign(data_hash.as_bytes());
        Self { signature }
    }
}

impl BuiltinAuthenticator for SessionAuthenticator {
    fn auth_validator_id(&self) -> u64 {
        BuiltinAuthValidator::Session.flag().into()
    }
    fn payload(&self) -> Vec<u8> {
        self.signature.as_ref().to_vec()
    }
}
#[cfg(any(test, feature = "fuzzing"))]
impl Arbitrary for SessionAuthenticator {
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
    ) -> SessionAuthenticator {
        let mut rng = StdRng::seed_from_u64(seed);
        let ed25519_keypair: Ed25519KeyPair = Ed25519KeyPair::generate(&mut rng);
        SessionAuthenticator {
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

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct BitcoinAuthenticator {
    pub payload: AuthPayload,
}

impl BitcoinAuthenticator {
    pub fn new(payload: AuthPayload) -> Self {
        Self { payload }
    }

    pub fn sign(kp: &RoochKeyPair, tx_data: &RoochTransactionData) -> Self {
        assert_eq!(kp.public().scheme(), SignatureScheme::Secp256k1);
        let sign_data = SignData::new_with_default(tx_data);
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
        BitcoinAuthenticator {
            payload: AuthPayload::new(sign_data, signature, bitcoin_address_str),
        }
    }
}

impl BuiltinAuthenticator for BitcoinAuthenticator {
    fn auth_validator_id(&self) -> u64 {
        BuiltinAuthValidator::Bitcoin.flag().into()
    }
    fn payload(&self) -> Vec<u8> {
        bcs::to_bytes(&self.payload).expect("Serialize BitcoinAuthenticator should success")
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct BitcoinMultisignAuthenticator {
    pub payload: MultisignAuthPayload,
}

impl BuiltinAuthenticator for BitcoinMultisignAuthenticator {
    fn auth_validator_id(&self) -> u64 {
        BuiltinAuthValidator::BitcoinMultisign.flag().into()
    }
    fn payload(&self) -> Vec<u8> {
        bcs::to_bytes(&self.payload)
            .expect("Serialize BitcoinMultisignAuthenticator should success")
    }
}

impl BitcoinMultisignAuthenticator {
    pub fn new(payload: MultisignAuthPayload) -> Self {
        Self { payload }
    }

    pub fn build_multisig_authenticator(authenticators: Vec<BitcoinAuthenticator>) -> Result<Self> {
        if authenticators.is_empty() {
            return Err(anyhow::anyhow!("At least one authenticator is required"));
        }
        let payload = MultisignAuthPayload::build_multisig_payload(
            authenticators.into_iter().map(|a| a.payload).collect(),
        )?;
        Ok(Self { payload })
    }
}

/// Signing envelope types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SigningEnvelope {
    RawTxHash = 0x00,
    BitcoinMessageV0 = 0x01,
    WebAuthnV0 = 0x02,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct DIDAuthPayload {
    pub envelope: u8,
    pub vm_fragment: String,
    pub signature: Vec<u8>,
    pub message: Option<Vec<u8>>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DIDAuthenticator {
    pub payload: DIDAuthPayload,
}

impl DIDAuthenticator {
    pub fn new(payload: DIDAuthPayload) -> Self {
        Self { payload }
    }

    pub fn sign(
        kp: &RoochKeyPair,
        tx_data: &RoochTransactionData,
        vm_fragment: &str,
        envelope: SigningEnvelope,
    ) -> Result<Self> {
        let tx_hash = tx_data.tx_hash();

        // Compute digest based on envelope type
        let (digest, message) = match envelope {
            SigningEnvelope::RawTxHash => (tx_hash.as_bytes().to_vec(), None),
            SigningEnvelope::BitcoinMessageV0 => {
                let message = format!("Rooch Transaction:\n{}", hex::encode(tx_hash));
                let message_bytes = message.as_bytes();
                let digest = bitcoin_message_digest(message_bytes);
                (digest, Some(message_bytes.to_vec()))
            }
            SigningEnvelope::WebAuthnV0 => {
                // WebAuthn implementation would go here
                return Err(anyhow::anyhow!("WebAuthn not yet implemented"));
            }
        };

        let signature = kp.sign(&digest);

        let payload = DIDAuthPayload {
            envelope: envelope as u8,
            vm_fragment: vm_fragment.to_string(),
            signature: signature.to_compressed()?.as_ref().to_vec(), // Use 64-byte compressed signature
            message,
        };

        Ok(Self { payload })
    }
}

impl BuiltinAuthenticator for DIDAuthenticator {
    fn auth_validator_id(&self) -> u64 {
        BuiltinAuthValidator::DID.flag().into()
    }

    fn payload(&self) -> Vec<u8> {
        bcs::to_bytes(&self.payload).expect("Serialize DIDAuthenticator should success")
    }
}

/// Helper function to compute Bitcoin message digest
fn bitcoin_message_digest(message: &[u8]) -> Vec<u8> {
    let prefix = b"Bitcoin Signed Message:\n";
    let varint = varint_encode(message.len());

    let mut full_message = Vec::new();
    full_message.extend_from_slice(prefix);
    full_message.extend_from_slice(&varint);
    full_message.extend_from_slice(message);

    let first_hash = sha2_256_of(&full_message);
    sha2_256_of(first_hash.as_bytes()).0.to_vec()
}

/// Simple varint encoding for message length
fn varint_encode(len: usize) -> Vec<u8> {
    if len < 0xfd {
        vec![len as u8]
    } else if len <= 0xffff {
        let mut bytes = vec![0xfd];
        bytes.extend_from_slice(&(len as u16).to_le_bytes());
        bytes
    } else if len <= 0xffffffff {
        let mut bytes = vec![0xfe];
        bytes.extend_from_slice(&(len as u32).to_le_bytes());
        bytes
    } else {
        let mut bytes = vec![0xff];
        bytes.extend_from_slice(&(len as u64).to_le_bytes());
        bytes
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
            auth_validator_id: BuiltinAuthValidator::Session.flag().into(),
            payload: vec![],
        }
    }

    pub fn sign(kp: &RoochKeyPair, tx_data: &RoochTransactionData) -> Self {
        match kp.public().scheme() {
            SignatureScheme::Ed25519 => Self::session(kp, tx_data),
            SignatureScheme::Secp256k1 => Self::bitcoin(kp, tx_data),
            SignatureScheme::EcdsaR1 => Self::session(kp, tx_data),
        }
    }

    /// Create a rooch authenticator for session key
    pub fn session(kp: &RoochKeyPair, tx_data: &RoochTransactionData) -> Self {
        SessionAuthenticator::sign(kp, tx_data).into()
    }

    /// Create a bitcoin authenticator for RoochTransaction
    /// We simulate the Bitcoin Wallet message signature
    pub fn bitcoin(kp: &RoochKeyPair, tx_data: &RoochTransactionData) -> Self {
        BitcoinAuthenticator::sign(kp, tx_data).into()
    }

    /// Create a bitcoin multisign authenticator for RoochTransaction
    pub fn bitcoin_multisign(authenticators: Vec<BitcoinAuthenticator>) -> Result<Self> {
        BitcoinMultisignAuthenticator::build_multisig_authenticator(authenticators).map(Into::into)
    }

    /// Create a DID authenticator
    pub fn did(
        kp: &RoochKeyPair,
        tx_data: &RoochTransactionData,
        vm_fragment: &str,
    ) -> Result<Self> {
        let did_auth =
            DIDAuthenticator::sign(kp, tx_data, vm_fragment, SigningEnvelope::RawTxHash)?;
        Ok(did_auth.into())
    }

    /// Create a DID authenticator with Bitcoin message envelope
    pub fn did_bitcoin_message(
        kp: &RoochKeyPair,
        tx_data: &RoochTransactionData,
        vm_fragment: &str,
    ) -> Result<Self> {
        let did_auth =
            DIDAuthenticator::sign(kp, tx_data, vm_fragment, SigningEnvelope::BitcoinMessageV0)?;
        Ok(did_auth.into())
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
    use super::*;
    use crate::crypto::RoochKeyPair;
    #[cfg(any(test, feature = "fuzzing"))]
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_rooch_authenticator_serialize_deserialize(authenticator in any::<super::SessionAuthenticator>()) {
            let serialized = serde_json::to_string(&authenticator).unwrap();
            let deserialized: super::SessionAuthenticator = serde_json::from_str(&serialized).unwrap();
            assert_eq!(authenticator.signature, deserialized.signature);
        }
    }

    #[test]
    fn test_session_authenticator_signature_format() {
        // Test with different signature schemes
        let test_cases = vec![
            ("Ed25519", RoochKeyPair::generate_ed25519()),
            ("Secp256k1", RoochKeyPair::generate_secp256k1()),
            ("EcdsaR1", RoochKeyPair::generate_ecdsa_r1()),
        ];

        for (scheme_name, kp) in test_cases {
            println!("Testing Session Authenticator with {}", scheme_name);

            let tx_data = create_test_tx_data();
            let session_auth = SessionAuthenticator::sign(&kp, &tx_data);
            let payload = session_auth.payload();

            // Verify payload format: [scheme(1)] + [signature(N)] + [public_key(M)]
            assert!(
                !payload.is_empty(),
                "Payload should not be empty for {}",
                scheme_name
            );

            // Verify scheme flag matches keypair
            let expected_scheme = kp.public().scheme().flag();
            assert_eq!(
                payload[0], expected_scheme,
                "Scheme flag mismatch for {}",
                scheme_name
            );

            // Verify total length matches expected format
            let expected_length = match kp.public().scheme() {
                SignatureScheme::Ed25519 => 1 + 64 + 32, // scheme + signature + pubkey
                SignatureScheme::Secp256k1 => 1 + 64 + 33, // scheme + signature + compressed pubkey
                SignatureScheme::EcdsaR1 => 1 + 64 + 33, // scheme + signature + pubkey
            };
            assert_eq!(
                payload.len(),
                expected_length,
                "Payload length mismatch for {}",
                scheme_name
            );

            // Verify serialization/deserialization
            let serialized = serde_json::to_string(&session_auth).unwrap();
            let deserialized: SessionAuthenticator = serde_json::from_str(&serialized).unwrap();
            assert_eq!(
                session_auth, deserialized,
                "Serialization roundtrip failed for {}",
                scheme_name
            );

            println!("✅ {} Session Authenticator format verified", scheme_name);
        }
    }

    #[test]
    fn test_did_authenticator_signature_format() {
        // Test with different signature schemes
        let test_cases = vec![
            ("Ed25519", RoochKeyPair::generate_ed25519()),
            ("Secp256k1", RoochKeyPair::generate_secp256k1()),
            ("EcdsaR1", RoochKeyPair::generate_ecdsa_r1()),
        ];

        for (scheme_name, kp) in test_cases {
            println!("Testing DID Authenticator with {}", scheme_name);

            let tx_data = create_test_tx_data();
            let did_auth =
                DIDAuthenticator::sign(&kp, &tx_data, "test-vm", SigningEnvelope::RawTxHash)
                    .unwrap();

            // Verify DID payload structure
            assert_eq!(
                did_auth.payload.envelope, 0,
                "RawTxHash envelope should be 0"
            );
            assert_eq!(
                did_auth.payload.vm_fragment, "test-vm",
                "VM fragment mismatch"
            );
            assert!(
                did_auth.payload.message.is_none(),
                "RawTxHash should have no message"
            );

            // Critical: DID signature must be 64 bytes (compressed signature only)
            assert_eq!(
                did_auth.payload.signature.len(),
                64,
                "DID signature must be 64 bytes for {} (got {})",
                scheme_name,
                did_auth.payload.signature.len()
            );

            // Verify signature doesn't start with scheme flag
            if !did_auth.payload.signature.is_empty() {
                let scheme_flag = kp.public().scheme().flag();
                assert_ne!(
                    did_auth.payload.signature[0], scheme_flag,
                    "DID signature should not start with scheme flag for {}",
                    scheme_name
                );
            }

            // Verify BCS serialization/deserialization
            let serialized = bcs::to_bytes(&did_auth.payload).unwrap();
            let deserialized: DIDAuthPayload = bcs::from_bytes(&serialized).unwrap();
            assert_eq!(
                did_auth.payload, deserialized,
                "BCS serialization roundtrip failed for {}",
                scheme_name
            );

            // Verify authenticator serialization
            let auth_serialized = bcs::to_bytes(&did_auth).unwrap();
            let auth_deserialized: DIDAuthenticator = bcs::from_bytes(&auth_serialized).unwrap();
            assert_eq!(
                did_auth, auth_deserialized,
                "Authenticator serialization roundtrip failed for {}",
                scheme_name
            );

            println!("✅ {} DID Authenticator format verified", scheme_name);
        }
    }

    #[test]
    fn test_did_authenticator_bitcoin_message_envelope() {
        let kp = RoochKeyPair::generate_secp256k1();
        let tx_data = create_test_tx_data();

        let did_auth =
            DIDAuthenticator::sign(&kp, &tx_data, "btc-vm", SigningEnvelope::BitcoinMessageV0)
                .unwrap();

        // Verify Bitcoin message envelope
        assert_eq!(
            did_auth.payload.envelope, 1,
            "BitcoinMessageV0 envelope should be 1"
        );
        assert_eq!(did_auth.payload.vm_fragment, "btc-vm");
        assert!(
            did_auth.payload.message.is_some(),
            "BitcoinMessageV0 should have message"
        );
        assert_eq!(
            did_auth.payload.signature.len(),
            64,
            "Signature should be 64 bytes"
        );

        // Verify message format
        let message = did_auth.payload.message.as_ref().unwrap();
        let message_str = String::from_utf8(message.clone()).unwrap();
        assert!(
            message_str.starts_with("Rooch Transaction:\n"),
            "Message should start with canonical prefix"
        );
        assert!(
            message_str.contains(&hex::encode(tx_data.tx_hash())),
            "Message should contain tx hash"
        );

        // Verify serialization
        let serialized = bcs::to_bytes(&did_auth.payload).unwrap();
        let deserialized: DIDAuthPayload = bcs::from_bytes(&serialized).unwrap();
        assert_eq!(did_auth.payload, deserialized);

        println!("✅ Bitcoin message envelope format verified");
    }

    #[test]
    fn test_signature_format_consistency() {
        let kp = RoochKeyPair::generate_secp256k1();
        let tx_data = create_test_tx_data();

        // Create both authenticators
        let session_auth = SessionAuthenticator::sign(&kp, &tx_data);
        let did_auth =
            DIDAuthenticator::sign(&kp, &tx_data, "test-vm", SigningEnvelope::RawTxHash).unwrap();

        let session_payload = session_auth.payload();

        // Extract components from session payload
        let session_scheme = session_payload[0];
        let session_signature = &session_payload[1..65]; // 64 bytes
        let session_pubkey = &session_payload[65..]; // 33 bytes for secp256k1

        // Verify DID uses compressed signature (same as session signature part)
        assert_eq!(did_auth.payload.signature.len(), 64);
        assert_eq!(did_auth.payload.signature, session_signature);

        // Verify scheme consistency (though DID doesn't store it)
        assert_eq!(session_scheme, kp.public().scheme().flag());

        // Verify public key consistency
        assert_eq!(session_pubkey, kp.public().as_ref());

        println!("✅ Signature format consistency verified between Session and DID authenticators");
    }

    #[test]
    fn test_authenticator_serialization_stability() {
        // Test that serialization format is stable across different runs
        let kp = RoochKeyPair::generate_secp256k1();
        let tx_data = create_test_tx_data();

        // Create authenticators multiple times
        let auth1 =
            DIDAuthenticator::sign(&kp, &tx_data, "stable-vm", SigningEnvelope::RawTxHash).unwrap();
        let auth2 =
            DIDAuthenticator::sign(&kp, &tx_data, "stable-vm", SigningEnvelope::RawTxHash).unwrap();

        // Signatures should be identical for same input
        assert_eq!(auth1.payload.signature, auth2.payload.signature);
        assert_eq!(auth1.payload.envelope, auth2.payload.envelope);
        assert_eq!(auth1.payload.vm_fragment, auth2.payload.vm_fragment);
        assert_eq!(auth1.payload.message, auth2.payload.message);

        // Serialization should be identical
        let serialized1 = bcs::to_bytes(&auth1).unwrap();
        let serialized2 = bcs::to_bytes(&auth2).unwrap();
        assert_eq!(serialized1, serialized2);

        println!("✅ Authenticator serialization stability verified");
    }

    #[test]
    fn test_varint_encoding() {
        // Test varint encoding for different message lengths
        let test_cases = vec![
            (0, vec![0]),
            (252, vec![252]),
            (253, vec![0xfd, 253, 0]),
            (65535, vec![0xfd, 255, 255]),
            (65536, vec![0xfe, 0, 0, 1, 0]),
        ];

        for (input, expected) in test_cases {
            let result = varint_encode(input);
            assert_eq!(
                result, expected,
                "Varint encoding failed for input {}",
                input
            );
        }

        println!("✅ Varint encoding verified");
    }

    #[test]
    fn test_bitcoin_message_digest() {
        let message = b"Hello, Bitcoin!";
        let digest = bitcoin_message_digest(message);

        // Verify digest is 32 bytes (SHA256 output)
        assert_eq!(digest.len(), 32);

        // Verify digest is deterministic
        let digest2 = bitcoin_message_digest(message);
        assert_eq!(digest, digest2);

        // Verify different messages produce different digests
        let different_message = b"Hello, World!";
        let different_digest = bitcoin_message_digest(different_message);
        assert_ne!(digest, different_digest);

        println!("✅ Bitcoin message digest verified");
    }

    // Helper function to create test transaction data
    fn create_test_tx_data() -> crate::transaction::RoochTransactionData {
        use crate::address::RoochAddress;
        use move_core_types::account_address::AccountAddress;
        use move_core_types::identifier::Identifier;
        use move_core_types::language_storage::ModuleId;
        use moveos_types::move_types::FunctionId;
        use moveos_types::transaction::MoveAction;

        let module_id = ModuleId::new(
            AccountAddress::from_hex_literal("0x1").unwrap(),
            Identifier::new("test").unwrap(),
        );
        let function_id = FunctionId::new(module_id, Identifier::new("test_function").unwrap());
        let action = MoveAction::new_function_call(function_id, vec![], vec![]);

        crate::transaction::RoochTransactionData::new_for_test(
            RoochAddress::from(AccountAddress::from_hex_literal("0x42").unwrap()),
            0,
            action,
        )
    }
}

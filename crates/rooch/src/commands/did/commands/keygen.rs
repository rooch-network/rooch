// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::{CommandAction, WalletContextOptions};
use async_trait::async_trait;
use clap::Parser;
use rooch_types::crypto::{self, EncodeDecodeBase64, RoochKeyPair, SignatureScheme};
use rooch_types::error::RoochResult;
use serde::{Deserialize, Serialize};

/// Generate cryptographic keys for DID operations
#[derive(Debug, Parser)]
pub struct KeygenCommand {
    #[clap(subcommand)]
    pub keygen_type: KeygenType,
}

#[derive(Debug, Parser)]
pub enum KeygenType {
    /// Generate an Ed25519 key pair
    #[clap(name = "ed25519")]
    Ed25519(Ed25519KeygenCommand),

    /// Generate a Secp256k1 key pair
    #[clap(name = "secp256k1")]
    Secp256k1(Secp256k1KeygenCommand),

    /// Generate an ECDSA R1 (P-256) key pair
    #[clap(name = "ecdsa-r1")]
    EcdsaR1(EcdsaR1KeygenCommand),

    /// Generate an RSASSA-PKCS1-v1_5 (RS256) key pair
    #[clap(name = "rsassa-pkcs1-v1_5")]
    Rsa(RsaKeygenCommand),

    /// Generate a did:key identifier from a multibase public key
    #[clap(name = "did-key")]
    DidKey(DidKeyCommand),
}

#[derive(Debug, Parser)]
pub struct Ed25519KeygenCommand {
    /// Output format
    #[clap(
        long,
        default_value = "multibase",
        help = "Output format: multibase, hex, base64"
    )]
    pub format: String,

    /// Include private key in output
    #[clap(
        long,
        help = "Include private key in output (use with caution)",
        default_value = "true"
    )]
    pub include_private: bool,

    /// Generate raw public key without scheme flag
    #[clap(
        long,
        help = "Generate raw public key bytes without scheme flag",
        default_value = "true"
    )]
    pub raw: bool,

    #[clap(flatten)]
    pub context_options: WalletContextOptions,
}

#[derive(Debug, Parser)]
pub struct Secp256k1KeygenCommand {
    /// Output format
    #[clap(
        long,
        default_value = "multibase",
        help = "Output format: multibase, hex, base64"
    )]
    pub format: String,

    /// Include private key in output
    #[clap(
        long,
        help = "Include private key in output (use with caution)",
        default_value = "true"
    )]
    pub include_private: bool,

    /// Generate raw public key without scheme flag
    #[clap(
        long,
        help = "Generate raw public key bytes without scheme flag",
        default_value = "true"
    )]
    pub raw: bool,

    #[clap(flatten)]
    pub context_options: WalletContextOptions,
}

#[derive(Debug, Parser)]
pub struct EcdsaR1KeygenCommand {
    /// Output format
    #[clap(
        long,
        default_value = "multibase",
        help = "Output format: multibase, hex, base64"
    )]
    pub format: String,

    /// Include private key in output
    #[clap(
        long,
        help = "Include private key in output (use with caution)",
        default_value = "true"
    )]
    pub include_private: bool,

    /// Generate raw public key without scheme flag
    #[clap(
        long,
        help = "Generate raw public key bytes without scheme flag",
        default_value = "true"
    )]
    pub raw: bool,

    #[clap(flatten)]
    pub context_options: WalletContextOptions,
}

#[derive(Debug, Parser)]
pub struct Rs256KeygenCommand {
    /// Output format
    #[clap(
        long,
        default_value = "multibase",
        help = "Output format: multibase, hex, base64"
    )]
    pub format: String,

    /// Include private key in output
    #[clap(
        long,
        help = "Include private key in output (use with caution)",
        default_value = "true"
    )]
    pub include_private: bool,

    /// Generate raw public key without scheme flag
    #[clap(
        long,
        help = "Generate raw public key bytes without scheme flag",
        default_value = "true"
    )]
    pub raw: bool,

    #[clap(flatten)]
    pub context_options: WalletContextOptions,
}

#[derive(Debug, Parser)]
pub struct DidKeyCommand {
    /// Multibase-encoded public key or did:key identifier
    #[clap(
        help = "Multibase-encoded public key (e.g., z8y1uAid...) or did:key identifier (e.g., z6MkpTHR8VNs...)"
    )]
    pub multibase_public_key: String,

    /// Key type (ed25519, secp256k1, ecdsa-r1, or rsassa-pkcs1-v1_5)
    #[clap(
        long,
        default_value = "ed25519",
        help = "Key type: ed25519, secp256k1, ecdsa-r1, rsassa-pkcs1-v1_5"
    )]
    pub key_type: String,

    #[clap(flatten)]
    pub context_options: WalletContextOptions,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KeygenOutput {
    pub key_type: String,
    pub public_key: PublicKeyOutput,
    pub private_key: Option<PrivateKeyOutput>,
    pub did_key: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PublicKeyOutput {
    pub multibase: String,
    pub hex: String,
    pub base64: String,
    pub raw_multibase: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PrivateKeyOutput {
    pub hex: String,
    pub base64: String,
    pub bech32: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DidKeyOutput {
    pub did_key: String,
    pub public_key_multibase: String,
    pub key_type: String,
}

#[async_trait]
impl CommandAction<KeygenOutput> for KeygenCommand {
    async fn execute(self) -> RoochResult<KeygenOutput> {
        match self.keygen_type {
            KeygenType::Ed25519(cmd) => cmd.execute().await,
            KeygenType::Secp256k1(cmd) => cmd.execute().await,
            KeygenType::EcdsaR1(cmd) => cmd.execute().await,
            KeygenType::Rsa(cmd) => cmd.execute().await,
            KeygenType::DidKey(cmd) => {
                let result = cmd.execute().await?;
                Ok(KeygenOutput {
                    key_type: result.key_type,
                    public_key: PublicKeyOutput {
                        multibase: result.public_key_multibase.clone(),
                        hex: "".to_string(),
                        base64: "".to_string(),
                        raw_multibase: Some(result.public_key_multibase),
                    },
                    private_key: None,
                    did_key: Some(result.did_key),
                })
            }
        }
    }
}

#[async_trait]
impl CommandAction<KeygenOutput> for Ed25519KeygenCommand {
    async fn execute(self) -> RoochResult<KeygenOutput> {
        let keypair = RoochKeyPair::generate_ed25519();
        let public_key = keypair.public();

        let public_key_output = PublicKeyOutput {
            multibase: public_key.to_multibase(),
            hex: public_key.to_hex_literal(),
            base64: public_key.encode_base64(),
            raw_multibase: if self.raw {
                Some(public_key.raw_to_multibase())
            } else {
                None
            },
        };

        let private_key_output = if self.include_private {
            Some(PrivateKeyOutput {
                hex: format!("0x{}", hex::encode(keypair.private())),
                base64: keypair.encode_base64(),
                bech32: keypair.export_private_key().map_err(|e| {
                    rooch_types::error::RoochError::CommandArgumentError(format!(
                        "Failed to export private key: {}",
                        e
                    ))
                })?,
            })
        } else {
            None
        };

        // Generate did:key identifier
        let did_key = generate_did_key(&public_key.raw_to_multibase(), "ed25519")?;

        Ok(KeygenOutput {
            key_type: "Ed25519".to_string(),
            public_key: public_key_output,
            private_key: private_key_output,
            did_key: Some(did_key),
        })
    }
}

#[async_trait]
impl CommandAction<KeygenOutput> for Secp256k1KeygenCommand {
    async fn execute(self) -> RoochResult<KeygenOutput> {
        let keypair = RoochKeyPair::generate_secp256k1();
        let public_key = keypair.public();

        let public_key_output = PublicKeyOutput {
            multibase: public_key.to_multibase(),
            hex: public_key.to_hex_literal(),
            base64: public_key.encode_base64(),
            raw_multibase: if self.raw {
                Some(public_key.raw_to_multibase())
            } else {
                None
            },
        };

        let private_key_output = if self.include_private {
            Some(PrivateKeyOutput {
                hex: format!("0x{}", hex::encode(keypair.private())),
                base64: keypair.encode_base64(),
                bech32: keypair.export_private_key().map_err(|e| {
                    rooch_types::error::RoochError::CommandArgumentError(format!(
                        "Failed to export private key: {}",
                        e
                    ))
                })?,
            })
        } else {
            None
        };

        // Generate did:key identifier
        let did_key = generate_did_key(&public_key.raw_to_multibase(), "secp256k1")?;

        Ok(KeygenOutput {
            key_type: "Secp256k1".to_string(),
            public_key: public_key_output,
            private_key: private_key_output,
            did_key: Some(did_key),
        })
    }
}

#[async_trait]
impl CommandAction<KeygenOutput> for EcdsaR1KeygenCommand {
    async fn execute(self) -> RoochResult<KeygenOutput> {
        let keypair = RoochKeyPair::generate_ecdsa_r1();
        let public_key = keypair.public();

        let public_key_output = PublicKeyOutput {
            multibase: public_key.to_multibase(),
            hex: public_key.to_hex_literal(),
            base64: public_key.encode_base64(),
            raw_multibase: if self.raw {
                Some(public_key.raw_to_multibase())
            } else {
                None
            },
        };

        let private_key_output = if self.include_private {
            Some(PrivateKeyOutput {
                hex: format!("0x{}", hex::encode(keypair.private())),
                base64: keypair.encode_base64(),
                bech32: keypair.export_private_key().map_err(|e| {
                    rooch_types::error::RoochError::CommandArgumentError(format!(
                        "Failed to export private key: {}",
                        e
                    ))
                })?,
            })
        } else {
            None
        };

        // Generate did:key identifier
        let did_key = generate_did_key(&public_key.raw_to_multibase(), "ecdsa-r1")?;

        Ok(KeygenOutput {
            key_type: "EcdsaR1".to_string(),
            public_key: public_key_output,
            private_key: private_key_output,
            did_key: Some(did_key),
        })
    }
}

#[async_trait]
impl CommandAction<KeygenOutput> for Rs256KeygenCommand {
    async fn execute(self) -> RoochResult<KeygenOutput> {
        let keypair = RoochKeyPair::generate_rs256();
        let public_key = keypair.public();

        let public_key_output = PublicKeyOutput {
            multibase: public_key.to_multibase(),
            hex: public_key.to_hex_literal(),
            base64: public_key.encode_base64(),
            raw_multibase: if self.raw {
                Some(public_key.raw_to_multibase())
            } else {
                None
            },
        };

        let private_key_output = if self.include_private {
            Some(PrivateKeyOutput {
                hex: format!("0x{}", hex::encode(keypair.private())),
                base64: keypair.encode_base64(),
                bech32: keypair.export_private_key().map_err(|e| {
                    rooch_types::error::RoochError::CommandArgumentError(format!(
                        "Failed to export private key: {}",
                        e
                    ))
                })?,
            })
        } else {
            None
        };

        // Generate did:key identifier
        let did_key = generate_did_key(&public_key.raw_to_multibase(), "rs256")?;

        Ok(KeygenOutput {
            key_type: "Rs256".to_string(),
            public_key: public_key_output,
            private_key: private_key_output,
            did_key: Some(did_key),
        })
    }
}

#[async_trait]
impl CommandAction<DidKeyOutput> for DidKeyCommand {
    async fn execute(self) -> RoochResult<DidKeyOutput> {
        // Validate key type
        let scheme = match self.key_type.to_lowercase().as_str() {
            "ed25519" => SignatureScheme::Ed25519,
            "secp256k1" => SignatureScheme::Secp256k1,
            "ecdsa-r1" => SignatureScheme::EcdsaR1,
            _ => {
                return Err(rooch_types::error::RoochError::CommandArgumentError(
                    format!("Unsupported key type: {}", self.key_type),
                ));
            }
        };

        // Check if input is already a did:key identifier or a raw multibase key
        let (raw_multibase_key, did_key) = if self.multibase_public_key.starts_with("z6Mk")
            || self.multibase_public_key.starts_with("zQ3s")
        {
            // Input appears to be a did:key identifier, extract raw key and validate
            let did_key_full = format!("did:key:{}", self.multibase_public_key);

            // Extract raw public key from did:key format
            let raw_key = extract_raw_key_from_did_key(&self.multibase_public_key, &self.key_type)?;

            (raw_key, did_key_full)
        } else {
            // Input is a raw multibase key, validate and generate did:key
            let _public_key = rooch_types::crypto::PublicKey::from_raw_multibase(
                &self.multibase_public_key,
                scheme,
            )
            .map_err(|e| {
                rooch_types::error::RoochError::CommandArgumentError(format!(
                    "Invalid multibase public key: {}",
                    e
                ))
            })?;

            let did_key = generate_did_key(&self.multibase_public_key, &self.key_type)?;
            (self.multibase_public_key.clone(), did_key)
        };

        Ok(DidKeyOutput {
            did_key,
            public_key_multibase: raw_multibase_key,
            key_type: self.key_type,
        })
    }
}

/// Extract raw multibase public key from a did:key identifier
fn extract_raw_key_from_did_key(did_key_identifier: &str, key_type: &str) -> RoochResult<String> {
    // Decode the multibase string
    let (_, decoded_bytes) = multibase::decode(did_key_identifier).map_err(|e| {
        rooch_types::error::RoochError::CommandArgumentError(format!(
            "Failed to decode did:key identifier: {}",
            e
        ))
    })?;

    // Check multicodec prefix and extract raw key
    if decoded_bytes.len() < 2 {
        return Err(rooch_types::error::RoochError::CommandArgumentError(
            "Invalid did:key format: too short".to_string(),
        ));
    }

    let (expected_prefix, expected_key_len) = match key_type.to_lowercase().as_str() {
        "ed25519" => ([0xed, 0x01], 32),
        "secp256k1" => ([0xe7, 0x01], 33),
        _ => {
            return Err(rooch_types::error::RoochError::CommandArgumentError(
                format!("Unsupported key type: {}", key_type),
            ));
        }
    };

    // Verify multicodec prefix
    if decoded_bytes.len() < 2
        || decoded_bytes[0] != expected_prefix[0]
        || decoded_bytes[1] != expected_prefix[1]
    {
        return Err(rooch_types::error::RoochError::CommandArgumentError(
            format!("Invalid multicodec prefix for {} key", key_type),
        ));
    }

    // Extract raw key bytes
    if decoded_bytes.len() != expected_key_len + 2 {
        return Err(rooch_types::error::RoochError::CommandArgumentError(
            format!(
                "Invalid key length for {}: expected {} bytes, got {}",
                key_type,
                expected_key_len,
                decoded_bytes.len() - 2
            ),
        ));
    }

    let raw_key_bytes = &decoded_bytes[2..];

    // Encode raw key as multibase
    let raw_multibase = multibase::encode(multibase::Base::Base58Btc, raw_key_bytes);

    Ok(raw_multibase)
}

/// Generate a did:key identifier from a multibase public key according to W3C DID Key spec
///
/// The W3C DID Key specification defines the format as:
/// did:key:MULTIBASE(base58-btc, MULTICODEC(public-key-type, raw-public-key-bytes))
///
/// Multicodec prefixes:
/// - Ed25519: 0xed01 -> results in z6Mk... format
/// - Secp256k1: 0xe701 -> results in zQ3s... format
fn generate_did_key(multibase_public_key: &str, key_type: &str) -> RoochResult<String> {
    // Validate the multibase format
    if !multibase_public_key.starts_with('z') {
        return Err(rooch_types::error::RoochError::CommandArgumentError(
            "Multibase public key must start with 'z' (base58btc encoding)".to_string(),
        ));
    }

    // Decode the raw public key bytes from multibase
    let (_, raw_key_bytes) = multibase::decode(multibase_public_key).map_err(|e| {
        rooch_types::error::RoochError::CommandArgumentError(format!(
            "Failed to decode multibase key: {}",
            e
        ))
    })?;

    // Generate did:key according to W3C spec with proper multicodec prefixes
    let did_key = match key_type.to_lowercase().as_str() {
        "ed25519" => {
            // Validate Ed25519 key length (32 bytes)
            if raw_key_bytes.len() != crypto::ED25519_PUBLIC_KEY_LENGTH {
                return Err(rooch_types::error::RoochError::CommandArgumentError(
                    format!(
                        "Invalid Ed25519 key length: expected {} bytes, got {}",
                        crypto::ED25519_PUBLIC_KEY_LENGTH,
                        raw_key_bytes.len()
                    ),
                ));
            }

            // For Ed25519, prepend the multicodec identifier 0xed01
            let mut full_key_bytes = vec![0xed, 0x01];
            full_key_bytes.extend_from_slice(&raw_key_bytes);

            let did_key_multibase = multibase::encode(multibase::Base::Base58Btc, &full_key_bytes);
            format!("did:key:{}", did_key_multibase)
        }
        "secp256k1" => {
            // Validate Secp256k1 compressed key length (33 bytes)
            if raw_key_bytes.len() != crypto::SECP256K1_PUBLIC_KEY_LENGTH {
                return Err(rooch_types::error::RoochError::CommandArgumentError(
                    format!(
                        "Invalid Secp256k1 key length: expected {} bytes, got {}",
                        crypto::SECP256K1_PUBLIC_KEY_LENGTH,
                        raw_key_bytes.len()
                    ),
                ));
            }

            // For Secp256k1, prepend the multicodec identifier 0xe701
            let mut full_key_bytes = vec![0xe7, 0x01];
            full_key_bytes.extend_from_slice(&raw_key_bytes);

            let did_key_multibase = multibase::encode(multibase::Base::Base58Btc, &full_key_bytes);
            format!("did:key:{}", did_key_multibase)
        }
        "ecdsa-r1" => {
            // Validate ECDSA R1 key length (33 bytes)
            if raw_key_bytes.len() != crypto::SECP256R1_PUBLIC_KEY_LENGTH {
                return Err(rooch_types::error::RoochError::CommandArgumentError(
                    format!(
                        "Invalid ECDSA R1 key length: expected {} bytes, got {}",
                        crypto::SECP256R1_PUBLIC_KEY_LENGTH,
                        raw_key_bytes.len()
                    ),
                ));
            }

            // For ECDSA R1, prepend the multicodec identifier 0x1201
            let mut full_key_bytes = vec![0x12, 0x01];
            full_key_bytes.extend_from_slice(&raw_key_bytes);

            let did_key_multibase = multibase::encode(multibase::Base::Base58Btc, &full_key_bytes);
            format!("did:key:{}", did_key_multibase)
        }
        _ => {
            return Err(rooch_types::error::RoochError::CommandArgumentError(
                format!("Unsupported key type for did:key: {}", key_type),
            ));
        }
    };

    Ok(did_key)
}

impl KeygenCommand {
    pub async fn execute_serialized(self) -> RoochResult<String> {
        let result = self.execute().await?;
        Ok(serde_json::to_string_pretty(&result)?)
    }
}

# DID Auth Validator Design

## Overview

This document describes the DID auth validator implementation for Rooch, which enables direct authentication using DID Document verification methods without requiring intermediate session key creation. The implementation includes a comprehensive compatibility system that allows seamless coexistence with existing session key authentication.

## Motivation

The current authentication flow for DID-based accounts requires:
1. Creating a session key from a DID verification method
2. Using the session key to sign transactions

This approach has several limitations:
- **Indirect authentication**: Users cannot directly use their DID authentication methods
- **Additional complexity**: Requires session key management
- **Limited flexibility**: Temporary or one-time operations still need session creation

The proposed DID auth validator addresses these issues by:
- Enabling direct authentication via DID Document verification methods
- Supporting multiple signature formats through the envelope extension mechanism
- Providing a cleaner separation of concerns between DID and session-based authentication

## Architecture

### Component Overview

```
┌─────────────────────┐     ┌─────────────────────┐
│ Transaction         │     │ DID Auth Validator  │
│ Validator           │────▶│                     │
└─────────────────────┘     └──────────┬──────────┘
                                       │
                            ┌──────────▼──────────┐
                            │ DID Module          │
                            │ - DID Document      │
                            │ - Verification      │
                            │   Methods           │
                            └─────────────────────┘
```

### Validator Registration

The DID auth validator will be registered as a built-in validator with ID 4:

```move
const DID_VALIDATOR_ID: u64 = 4;
```

## Implementation Details

### 1. Authenticator Payload Format

The DID authenticator payload uses **BCS (Binary Canonical Serialization)** for consistent serialization across all platforms:

```move
struct DIDAuthPayload has copy, store, drop {
    scheme: u8,
    envelope: u8,
    vm_fragment: String,
    signature: vector<u8>,
    message: Option<vector<u8>>,
}
```

Fields:
- `scheme`: Authentication scheme (Ed25519, Secp256k1, Secp256r1)
- `envelope`: Signing envelope type (RawTxHash, BitcoinMessageV0, WebAuthnV0) - **always required**
- `vm_fragment`: Verification method fragment (e.g., "key-1")
- `signature`: The signature bytes
- `message`: Optional message for certain envelope types (BitcoinMessageV0, WebAuthnV0)

**Design Decisions**:
1. **BCS Serialization**: Uses standard BCS format for consistency between Move, Rust, and TypeScript implementations
2. **Explicit Envelope**: Always requires an envelope byte (no legacy compatibility needed)
3. **Type Safety**: Leverages Move's type system for better validation
4. **Specific Error Codes**: Uses detailed error codes for better debugging (see Error Handling section)
5. **Compatibility Strategy**: Uses session_key field encoding to maintain backward compatibility without structural changes

Note: The DID identifier is derived from the transaction sender address, eliminating redundancy in the payload. This optimization:
- Reduces payload size and gas costs
- Prevents potential mismatch between claimed DID and actual sender
- Simplifies client-side implementation

### 2. Envelope Types

The DID validator supports the same envelope types as session validator:

- **0x00 RawTxHash**: Direct signature over transaction hash
- **0x01 BitcoinMessageV0**: Bitcoin-style message signing
- **0x02 WebAuthnV0**: WebAuthn authentication

### 3. Validation Flow

```move
public fun validate(authenticator_payload: vector<u8>): DID {
    // 1. Parse authenticator payload
    let auth_payload = parse_did_auth_payload(&authenticator_payload);
    
    // 2. Derive DID from sender address
    let sender = tx_context::sender();
    let sender_did = did::new_rooch_did_by_address(sender);
    let did_identifier = did::get_did_identifier_string(&sender_did);
    
    // 3. Resolve DID Document
    let did_object_id = did::resolve_did_object_id(&did_identifier);
    assert!(
        object::exists_object_with_type<DIDDocument>(did_object_id),
        auth_validator::error_validate_invalid_authenticator()
    );
    let did_doc = did::borrow_did_document(did_object_id);
    
    // 4. Verify the verification method is authorized for authentication
    assert!(
        vector::contains(
            did::doc_authentication_methods(did_doc), 
            &auth_payload.verification_method_fragment
        ),
        auth_validator::error_validate_invalid_authenticator()
    );
    
    // 5. Get verification method details
    let vm_opt = did::doc_verification_method(
        did_doc, 
        &auth_payload.verification_method_fragment
    );
    assert!(
        option::is_some(&vm_opt),
        auth_validator::error_validate_invalid_authenticator()
    );
    let vm = option::extract(&mut vm_opt);
    
    // 6. Compute message digest based on envelope type
    let tx_hash = tx_context::tx_hash();
    let digest = compute_digest(
        tx_hash, 
        auth_payload.envelope_type, 
        auth_payload.message
    );
    
    // 7. Verify signature
    let valid = did::verify_signature_by_type(
        digest,
        auth_payload.signature,
        did::verification_method_public_key_multibase(&vm),
        did::verification_method_type(&vm)
    );
    
    assert!(valid, auth_validator::error_validate_invalid_authenticator());
    
    // Return the DID for transaction context
    sender_did
}
```

### 4. Integration with Transaction Validator

Add DID validator handling in `transaction_validator::validate`:

```move
else if (auth_validator_id == did_validator::auth_validator_id()) {
    let (_did, vm_fragment) = did_validator::validate(authenticator_payload);
    
    // Encode vm_fragment in session_key field for backward compatibility
    let encoded_vm_info = encode_did_vm_info(vm_fragment);
    
    // DID accounts may not have associated Bitcoin addresses
    let bitcoin_address = address_mapping::resolve_bitcoin(sender);
    (bitcoin_address, option::some(encoded_vm_info), option::none())
}

/// Encode DID VM fragment for storage in session_key field
/// Format: "DID_VM:" + vm_fragment
fun encode_did_vm_info(vm_fragment: String): vector<u8> {
    let prefix = b"DID_VM:";
    let result = vector::empty<u8>();
    vector::append(&mut result, prefix);
    vector::append(&mut result, *string::bytes(&vm_fragment));
    result
}
```

### 5. Compatibility with Existing DID Module

To maintain compatibility with existing `did.move` logic that depends on session keys, we encode the DID verification method fragment in the `session_key` field of `TxValidateResult`. This approach avoids modifying the `TxValidateResult` structure while providing full backward compatibility.

The `did.move` module is updated to detect and handle both authentication methods:

```move
/// Get verification method fragment from transaction context
/// Supports both DID validator and session key authentication
fun get_vm_fragment_from_context(did_document_data: &DIDDocument): Option<String> {
    // Get session key from context (might be encoded DID info or actual session key)
    let session_key_opt = auth_validator::get_session_key_from_ctx_option();
    if (option::is_some(&session_key_opt)) {
        let session_key = option::extract(&mut session_key_opt);
        
        // Check if this is encoded DID validator info
        if (is_did_validator_data(&session_key)) {
            let vm_fragment = extract_vm_fragment_from_did_data(&session_key);
            return option::some(vm_fragment)
        } else {
            // Fall back to session key logic for backward compatibility
            find_verification_method_by_session_key(did_document_data, &session_key)
        }
    } else {
        option::none()
    }
}

/// Check if session key data is actually encoded DID validator info
fun is_did_validator_data(session_key: &vector<u8>): bool {
    let prefix = b"DID_VM:";
    if (vector::length(session_key) < vector::length(&prefix)) {
        return false
    };
    
    let i = 0;
    while (i < vector::length(&prefix)) {
        if (*vector::borrow(session_key, i) != *vector::borrow(&prefix, i)) {
            return false
        };
        i = i + 1;
    };
    true
}

/// Extract VM fragment from encoded DID validator data
fun extract_vm_fragment_from_did_data(session_key: &vector<u8>): String {
    let prefix = b"DID_VM:";
    let prefix_len = vector::length(&prefix);
    let vm_fragment_bytes = vector::empty<u8>();
    
    let i = prefix_len;
    while (i < vector::length(session_key)) {
        vector::push_back(&mut vm_fragment_bytes, *vector::borrow(session_key, i));
        i = i + 1;
    };
    
    string::utf8(vm_fragment_bytes)
}
```

**Encoding Strategy**:
- **DID Validator**: Stores `"DID_VM:" + vm_fragment` in the `session_key` field
- **Session Key**: Stores actual authentication key bytes in the `session_key` field
- **Detection**: Uses `"DID_VM:"` prefix to distinguish between the two formats

This approach provides:
- **Full Backward Compatibility**: No changes to `TxValidateResult` structure
- **Clear Identification**: Prefix-based detection prevents confusion
- **Seamless Migration**: Applications can gradually adopt DID validator
- **Easy Cleanup**: Future removal of session key dependency is straightforward

### 6. Authentication Method Comparison

| Authentication Method | session_key Field Content | Processing Logic |
|----------------------|---------------------------|------------------|
| **Session Key** | Raw authentication key bytes | `find_verification_method_by_session_key()` |
| **DID Validator** | `"DID_VM:" + vm_fragment` | Direct extraction of vm_fragment |

## Compatibility Solution Summary

### Problem Statement

The original DID system relied on session keys stored in `TxValidateResult.session_key` for authorization checks in `did.move`. When introducing the DID validator that bypasses session key creation, existing business logic would break because:

1. `did.move` functions expect `session_key` to be present in transaction context
2. DID validator doesn't create session keys, only validates DID verification methods directly
3. Modifying `TxValidateResult` structure would break compatibility with existing contracts

### Solution: Session Key Field Encoding

**Approach**: Encode DID verification method information in the existing `session_key` field using a distinguishable format.

**Implementation**:
1. **Encoding**: DID validator stores `"DID_VM:" + vm_fragment` in `session_key` field
2. **Detection**: `did.move` checks for `"DID_VM:"` prefix to identify DID validator transactions
3. **Routing**: Automatically routes to appropriate logic based on detected format

**Benefits**:
- ✅ **Zero Breaking Changes**: No modifications to existing structures or interfaces
- ✅ **Full Backward Compatibility**: Session key authentication continues unchanged
- ✅ **Transparent Migration**: Applications can adopt DID validator gradually
- ✅ **Clear Separation**: Prefix-based detection prevents confusion
- ✅ **Future-Proof**: Easy to clean up when migration is complete

**Trade-offs**:
- Slight complexity in `did.move` detection logic
- Temporary encoding overhead (removed after full migration)

This solution successfully bridges the gap between old session key authentication and new DID validator authentication, enabling a smooth transition without breaking existing functionality.

## Error Handling

The DID validator uses specific error codes for better debugging and troubleshooting:

| Error Code | Constant | Description |
|------------|----------|-------------|
| 101001 | `ErrorInvalidDIDAuthPayload` | BCS deserialization failed - invalid payload format |
| 101002 | `ErrorInvalidEnvelopeType` | Invalid or unsupported envelope type |
| 101003 | `ErrorDIDDocumentNotFound` | DID document not found for sender address |
| 101004 | `ErrorVerificationMethodNotAuthorized` | Verification method not in authentication relationship |
| 101005 | `ErrorVerificationMethodNotFound` | Verification method not found in DID document |
| 101006 | `ErrorInvalidEnvelopeMessage` | Invalid message for envelope type (e.g., wrong Bitcoin message format) |
| 101007 | `ErrorSignatureVerificationFailed` | Cryptographic signature verification failed |

**Error Code Strategy**:
- DID validator uses 101xxx range (101001-101999) to avoid conflicts with other validators
- Auth validator uses 1xxx range (1001-1013)
- This clear separation makes debugging much easier and prevents error code conflicts

## Security Considerations

### 1. Verification Method Authorization

- Only verification methods listed in the `authentication` relationship can be used
- The validator always checks the current state of the DID Document
- Revoked or removed verification methods are immediately invalid

### 2. Sender-DID Binding

- DID identifier is always derived from the transaction sender address
- This ensures the DID being authenticated matches the account sending the transaction
- Prevents impersonation attacks where a user might try to authenticate with another account's DID

### 3. Replay Protection

- Transaction hash binding prevents replay attacks
- Envelope-specific protections (e.g., WebAuthn challenge verification)
- Chain ID validation in transaction validator

### 4. DID Document Integrity

- DID Documents are stored as immutable objects with controlled update mechanisms
- Controller verification ensures only authorized parties can modify documents
- Timestamp tracking enables audit trails

## SDK Support

### TypeScript SDK Example

```typescript
class DIDAuthenticator implements IAuthenticator {
    constructor(
        private verificationMethodFragment: string,
        private signer: Signer,
        private envelope: SigningEnvelope = SigningEnvelope.RawTxHash
    ) {}
    
    async sign(message: Uint8Array): Promise<Signature> {
        // Sign based on envelope type
        const signData = this.prepareSignData(message);
        return await this.signer.sign(signData);
    }
    
    build(): Authenticator {
        // Build DID authenticator payload
        // Note: DID identifier is derived from sender address on-chain
        const payload = encodeDIDAuthPayload({
            verificationMethodFragment: this.verificationMethodFragment,
            envelope: this.envelope,
            signature: this.signature,
            message: this.envelopeMessage
        });
        
        return new Authenticator(DID_VALIDATOR_ID, payload);
    }
}
```

### Rust SDK Example

```rust
use rooch_types::{
    crypto::{RoochKeyPair, Signature, SignatureScheme},
    framework::auth_validator::BuiltinAuthValidator,
    transaction::{Authenticator, RoochTransactionData},
};
use serde::{Deserialize, Serialize};
use anyhow::Result;

const DID_VALIDATOR_ID: u64 = 4;

/// Signing envelope types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SigningEnvelope {
    RawTxHash = 0x00,
    BitcoinMessageV0 = 0x01, 
    WebAuthnV0 = 0x02,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DIDAuthPayload {
    pub scheme: u8,
    pub envelope: u8,
    pub vm_fragment: String,
    pub signature: Vec<u8>,
    pub message: Option<Vec<u8>>,
}

pub struct DIDAuthenticator {
    vm_fragment: String,
    envelope: SigningEnvelope,
}

impl DIDAuthenticator {
    pub fn new(vm_fragment: String) -> Self {
        Self {
            vm_fragment,
            envelope: SigningEnvelope::RawTxHash,
        }
    }

    pub fn with_envelope(mut self, envelope: SigningEnvelope) -> Self {
        self.envelope = envelope;
        self
    }

    pub fn sign(
        &self,
        kp: &RoochKeyPair,
        tx_data: &RoochTransactionData,
    ) -> Result<Authenticator> {
        let tx_hash = tx_data.tx_hash();
        
        // Compute digest based on envelope type
        let (digest, message) = match self.envelope {
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
            scheme: signature.scheme().flag(),
            envelope: self.envelope as u8,
            vm_fragment: self.vm_fragment.clone(),
            signature: signature.as_ref().to_vec(),
            message,
        };

        let payload_bytes = bcs::to_bytes(&payload)?;
        Ok(Authenticator::new(DID_VALIDATOR_ID, payload_bytes))
    }
}

/// Helper function to compute Bitcoin message digest
fn bitcoin_message_digest(message: &[u8]) -> Vec<u8> {
    use sha2::{Sha256, Digest};
    
    let mut hasher = Sha256::new();
    hasher.update(b"Bitcoin Signed Message:\n");
    hasher.update(&varint_encode(message.len()));
    hasher.update(message);
    let first_hash = hasher.finalize();
    
    let mut second_hasher = Sha256::new();
    second_hasher.update(&first_hash);
    second_hasher.finalize().to_vec()
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

// Usage example
impl Authenticator {
    /// Create a DID authenticator
    pub fn did(
        kp: &RoochKeyPair,
        tx_data: &RoochTransactionData,
        vm_fragment: &str,
    ) -> Result<Self> {
        let did_auth = DIDAuthenticator::new(vm_fragment.to_string());
        did_auth.sign(kp, tx_data)
    }
    
    /// Create a DID authenticator with Bitcoin message envelope
    pub fn did_bitcoin_message(
        kp: &RoochKeyPair,
        tx_data: &RoochTransactionData,
        vm_fragment: &str,
    ) -> Result<Self> {
        let did_auth = DIDAuthenticator::new(vm_fragment.to_string())
            .with_envelope(SigningEnvelope::BitcoinMessageV0);
        did_auth.sign(kp, tx_data)
    }
}
```

### SDK Implementation Notes

#### TypeScript SDK
- The `DIDAuthenticator` class follows the existing `IAuthenticator` interface pattern
- Envelope types should be defined as an enum matching the Move constants
- The payload encoding should use BCS serialization for consistency

#### Rust SDK
- The implementation extends the existing `Authenticator` struct with DID-specific methods
- Envelope-specific digest computation is handled within the SDK to simplify usage
- The `DIDAuthPayload` struct must match the expected format on-chain
- Signature scheme is automatically determined from the key pair
- Bitcoin message digest computation follows the standard format with proper varint encoding

Both SDKs should:
- Validate the verification method fragment format before signing
- Handle errors gracefully with meaningful error messages
- Support all three envelope types (RawTxHash, BitcoinMessageV0, WebAuthnV0)
- Use the same payload format to ensure cross-SDK compatibility

## Testing Strategy

### 1. Unit Tests

- Payload parsing with various formats
- Signature verification for each scheme
- Envelope type handling
- Error cases (invalid DID, unauthorized method, bad signature)

### 2. Integration Tests

- Full transaction flow with DID authentication
- Interaction with DID module operations
- Cross-validator compatibility (e.g., session keys created from DID)

### 3. Test Cases

```move
#[test]
fun test_did_auth_raw_tx_hash() {
    // Test direct transaction hash signing
}

#[test]
fun test_did_auth_bitcoin_message() {
    // Test Bitcoin message envelope
}

#[test]
fun test_did_auth_webauthn() {
    // Test WebAuthn envelope
}

#[test]
#[expected_failure(abort_code = auth_validator::error_validate_invalid_authenticator())]
fun test_did_auth_unauthorized_method() {
    // Test using non-authentication verification method
}
```

## Migration and Compatibility

### Backward Compatibility

- **Full Compatibility**: Existing session-based authentication continues to work unchanged
- **No Breaking Changes**: No modifications to `TxValidateResult` or other core structures
- **Transparent Detection**: System automatically detects authentication method type
- **Existing DID Documents**: No changes required for existing DID Documents

### Migration Path

1. **Phase 1 - Deployment**
   - Deploy DID validator module
   - Register validator in genesis or via upgrade
   - Update SDKs to support DID authentication

2. **Phase 2 - Coexistence**
   - Both session key and DID validator authentication work simultaneously
   - Applications can choose which method to use per transaction
   - Gradual testing and adoption by applications

3. **Phase 3 - Migration**
   - Applications gradually migrate from session keys to DID validator
   - Monitor usage patterns and performance
   - Provide migration tools and documentation

4. **Phase 4 - Cleanup (Future)**
   - When session key usage drops to acceptable levels
   - Remove session key compatibility code from `did.move`
   - Simplify authentication logic

### Compatibility Implementation Details

The compatibility system works by:

1. **Encoding Detection**: DID validator data is prefixed with `"DID_VM:"` in the `session_key` field
2. **Automatic Routing**: `did.move` automatically detects the format and routes to appropriate logic
3. **Zero Overhead**: No performance impact on existing session key authentication
4. **Clean Separation**: Clear distinction between old and new authentication methods

## Future Extensions

### 1. Capability-Based Authorization

```move
struct DIDAuthPayloadV2 {
    // ... existing fields ...
    requested_capability: Option<String>, // e.g., "transfer", "update"
    capability_params: Option<vector<u8>>, // Capability-specific parameters
}
```

### 2. Multi-Signature Support

- Support for threshold signatures using multiple verification methods
- Aggregate signature schemes

### 3. Delegation Chains

- Support for transitive delegations
- Verification of delegation paths

### 4. Performance Optimizations

- DID Document caching with TTL
- Batch verification for multiple transactions
- Bloom filters for quick authorization checks

## References

- [W3C DID Core Specification](https://www.w3.org/TR/did-core/)
- [Rooch DID Module Documentation](./rooch_move_guide.md#42-did-system-and-authentication)
- [Session Signing Envelope Design](./session_signing_envelope.md)

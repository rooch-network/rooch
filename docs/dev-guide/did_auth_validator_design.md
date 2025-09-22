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

The DID authenticator payload uses BCS (Binary Canonical Serialization) and contains:

- envelope (u8): Signing envelope type (RawTxHash, BitcoinMessageV0, WebAuthnV0). Always required
- vm_fragment (String): DID verification method fragment (e.g., "key-1")
- signature (vector<u8>): Signature bytes
- message (Option<vector<u8>>): Optional, required by some envelopes (e.g., BitcoinMessageV0, WebAuthnV0)

**Design Decisions**:
1. **BCS Serialization**: Consistent across Move, Rust, and TypeScript
2. **Explicit Envelope**: Always requires an envelope byte
3. **No Explicit Scheme**: The verification algorithm is chosen based on the DID Verification Method type, not an explicit `scheme` field
4. **Specific Error Codes**: Uses detailed error codes (see Error Handling)
5. **Compatibility Strategy**: Uses `session_key` field encoding to maintain backward compatibility without structural changes

Note: The DID identifier is derived from the transaction sender address, eliminating redundancy in the payload. This optimization:
- Reduces payload size and gas costs
- Prevents potential mismatch between claimed DID and actual sender
- Simplifies client-side implementation

### 2. Envelope Types

The DID validator supports the same envelope types as session validator and defines their verification digests explicitly:

- **0x00 RawTxHash**: Direct signature over transaction hash
  - Digest: `tx_hash`
- **0x01 BitcoinMessageV0**: Bitcoin-style message signing
  - Digest: `SHA256(SHA256("Bitcoin Signed Message:\n" + VarInt(len(message)) + message))`
  - Requires an explicit `message` in the payload; see Template Binding below
- **0x02 WebAuthnV0**: WebAuthn authentication
  - Digest: `authenticator_data || SHA256(client_data_json)`
  - `message` contains a BCS-encoded `WebauthnAuthPayload` structure; the `challenge` in `client_data_json` MUST equal the base64-encoded `tx_hash`

Unknown envelope values MUST be rejected. Reserved for future extensions: `0x03 Bip322Simple`, `0x10..` vendor-specific envelopes.

#### Template Binding (BitcoinMessageV0)

To prevent replay and phishing, the `message` for `BitcoinMessageV0` MUST equal a canonical template derived from the transaction under verification:

```
Rooch Transaction:\n<hex_lowercase(tx_hash)>
```

- `hex_lowercase(tx_hash)` is the lowercase hex encoding of the 32-byte Rooch transaction hash
- The template MUST be exactly reproduced on-chain for equality comparison; messages deviating from the template MUST be rejected

#### Implementation Notes (Move)

- Hex encoding: implement a simple nibble-to-ASCII function to produce `hex_lowercase(tx_hash)` deterministically
- VarInt: initially support the single-byte branch (`len(message) < 253`) since the canonical template length is small; extend to full VarInt if/when needed
- Keep parsing loops and strict length checks to avoid out-of-bounds reads

### 3. Validation Flow (Abstract)

At a high level, the validator:

1. Parses the BCS-encoded authenticator payload
2. Derives the sender's DID from the transaction sender address
3. Loads the current DID Document for the sender
4. Checks that `vm_fragment` is authorized under the DID's `authentication` relationship
5. Computes the message digest according to the `envelope`
6. Verifies the signature using the DID Verification Method's type and public key
7. Returns the sender DID and `vm_fragment` to the transaction validator

Envelope notes (conceptual):
- RawTxHash: digest is the Rooch `tx_hash`
- BitcoinMessageV0: digest follows Bitcoin message signing (double SHA-256 over prefix + length + message). The implementation ensures the `message` equals the canonical template derived from `tx_hash`
- WebAuthnV0: digest is `authenticator_data || SHA256(client_data_json)` and the `challenge` must equal base64-encoded `tx_hash`

### 4. Integration with Transaction Validator (Abstract)

The transaction validator invokes the DID validator and, on success, stores the result in the transaction context:

- The returned `vm_fragment` is encoded into the existing `session_key` field using a distinct prefix (e.g., `"DID_VM:"`) to preserve backward compatibility
- Helper APIs in `rooch_framework::auth_validator` handle encoding/decoding and accessors for the transaction context

### 5. Compatibility with Existing DID Module (Abstract)

To remain compatible with contracts that expect a session key in the transaction context:

- The DID validator encodes the `vm_fragment` into the `session_key` field with a distinct prefix (e.g., `"DID_VM:"`)
- Consumers can detect the prefix to distinguish DID validation from a real session key
- Convenience functions in `rooch_framework::auth_validator` provide accessors to obtain either the actual session key or the DID VM fragment

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

### SDK Notes (Abstract)

- Encode DID authenticator payload with BCS; include `envelope`, `vm_fragment`, `signature`, and optional `message`
- For Bitcoin-only wallets, build the canonical template from `tx_hash` and use the Bitcoin message envelope
- Do not include an explicit scheme in the DID payload; rely on the DID Verification Method type and signature bytes
- Reference implementations:
  - TypeScript: `sdk/typescript/rooch-sdk/src/crypto/authenticator.ts`
  - Rust: `crates/rooch-types/src/transaction/authenticator.rs`

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
- BitcoinMessageV0 success with correct template and signature
- Failure when `message` mismatches the canonical template (BitcoinMessageV0)
- Failure for unknown/unsupported `envelope` values
- Failure for message length inconsistencies (e.g., VarInt mismatch)
- Error cases (invalid DID, unauthorized method, bad signature)

### 2. Integration Tests

- Full transaction flow with DID authentication
- Interaction with DID module operations
- Cross-validator compatibility (e.g., session keys created from DID)

### 3. Test Cases (Examples)

- Direct `tx_hash` signing (RawTxHash)
- Bitcoin message envelope with canonical template (BitcoinMessageV0)
- WebAuthn envelope with challenge binding
- Unauthorized verification method should fail

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
 - Source: `frameworks/rooch-framework/sources/auth_validator/did_validator.move`
 - Source: `frameworks/rooch-framework/sources/auth_validator/auth_validator.move`

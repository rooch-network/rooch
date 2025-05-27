# Rooch DID Implementation Guide

This document provides developers and AI with a comprehensive overview of the Rooch DID system, including architectural design, technical details, usage methods, and development considerations.

## Table of Contents

1. [System Architecture](#system-architecture)
2. [Core Components](#core-components)
3. [DID Standard Compliance](#did-standard-compliance)
4. [Key Management](#key-management)
5. [Permission Model](#permission-model)
6. [Command-Line Interface (CLI)](#command-line-interface-cli)
7. [Development Guide](#development-guide)
8. [Testing and Validation](#testing-and-validation)

## System Architecture

The Rooch DID system employs a layered architecture, comprising the following main components:

### Core Layer (Move Contracts)
- **DID Module** (`frameworks/rooch-framework/sources/did.move`)
  - Creation, management, and querying of DID documents
  - Management of verification methods and services
  - Permission validation and access control
  - Integration with the Rooch session key system

- **Multibase Module** (`frameworks/moveos-stdlib/sources/multibase.move`)
  - Support for multiple encoding formats (base58btc, base64, base16)
  - Handling of multicodec prefixes for the DID:Key standard
  - Key encoding/decoding utility functions

### Application Layer (Rust CLI)
- **DID Command Module** (`crates/rooch/src/commands/did/`)
  - DID creation (self-creation, CADOP-assisted creation)
  - DID management (verification methods, services, relationships)
  - Querying DID information
  - Key generation utility

- **Type Definitions** (`crates/rooch-types/src/framework/did.rs`)
  - Bindings between Rust and Move types
  - Serialization/deserialization support
  - Client API interfaces

## Core Components

### DID Document Structure

```move
struct DIDDocument has key {
    id: DID,                                    // DID identifier
    controller: vector<DID>,                    // List of controllers
    verification_methods: SimpleMap<String, VerificationMethod>, // Verification methods
    authentication: vector<String>,             // Authentication relationship
    assertion_method: vector<String>,           // Assertion method relationship
    capability_invocation: vector<String>,      // Capability invocation relationship
    capability_delegation: vector<String>,      // Capability delegation relationship
    key_agreement: vector<String>,              // Key agreement relationship
    services: SimpleMap<String, Service>,       // Service endpoints
    also_known_as: vector<String>,             // Aliases
    account_cap: AccountCap,                   // Associated account capability
}
```

### Verification Method Types

Supported verification method types:
- **Ed25519VerificationKey2020**: Ed25519 signature algorithm
- **EcdsaSecp256k1VerificationKey2019**: Secp256k1 signature algorithm

### Verification Relationship Types

- **Authentication (0)**: Identity authentication
- **AssertionMethod (1)**: Assertion method
- **CapabilityInvocation (2)**: Capability invocation
- **CapabilityDelegation (3)**: Capability delegation
- **KeyAgreement (4)**: Key agreement

## DID Standard Compliance

### W3C DID Specification Support

The Rooch DID system fully complies with the following standards:
- [W3C DID Core Specification](https://www.w3.org/TR/did-core/)
- [W3C DID Key Method Specification](https://w3c-ccg.github.io/did-method-key/)
- [Multibase Specification](https://github.com/multiformats/multibase)
- [Multicodec Specification](https://github.com/multiformats/multicodec)

### DID:Key Format

According to the W3C DID Key specification, the DID:Key format is:
```
did:key:MULTIBASE(base58-btc, MULTICODEC(public-key-type, raw-public-key-bytes))
```

#### Multicodec Prefixes
- **Ed25519**: `0xed01` → Encoded as `z6Mk...`
- **Secp256k1**: `0xe701` → Encoded as `zQ3s...`

#### Examples
```
# Ed25519
did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH

# Secp256k1  
did:key:zQ3shokFTS3brHcDQrn82RUDfCZESWL1ZdCEJwekUDPQiYBme
```

## Key Management

### Key Generation

Supports multiple key generation methods:

```bash
# Generate Ed25519 key pair
rooch did keygen ed25519 [--raw] [--include-private]

# Generate Secp256k1 key pair
rooch did keygen secp256k1 [--raw] [--include-private]

# Generate did:key from multibase public key
rooch did keygen did-key <MULTIBASE_KEY> --key-type <TYPE>
```

### Key Formats

Supports multiple key encoding formats:
- **multibase**: Full public key including scheme flag (base58btc encoded)
- **hex**: Hexadecimal format (0x prefix)
- **base64**: Base64 encoded format
- **raw_multibase**: Raw public key bytes (base58btc encoded, used for DID verification methods)

### Session Key Integration

The DID system is deeply integrated with the Rooch session key system:
- Ed25519/Secp256k1 verification methods added to the `authentication` relationship are automatically registered as session keys.
- Supports permission validation based on session keys.
- Automatically manages the lifecycle of session keys.

## Permission Model

### Permission Validation Mechanism

Permission validation for DID operations follows these principles:

1. **Transaction Sender**: DID address (address of the DID Document)
2. **Transaction Signer**: DID controller address (private key stored in the keystore)
3. **Permission Validation**: Move contract validates signer permissions on-chain

### Permission Levels

- **CapabilityDelegation**: Key management and verification relationship modification
- **CapabilityInvocation**: Service management
- **Authentication**: Identity authentication and session key usage

### Permission Validation Flow

```rust
// 1. Verify the signer is the DID's associated account
assert!(sender == did_account_address, ErrorSignerNotDIDAccount);

// 2. Get the session key from the current transaction context
let session_key = auth_validator::get_session_key_from_ctx_option();

// 3. Find the corresponding verification method
let vm_fragment = find_verification_method_by_session_key(did_document, &session_key);

// 4. Check the permission relationship of the verification method
assert!(has_required_relationship(vm_fragment, required_permission));
```

## Command-Line Interface (CLI)

### DID Creation

#### Self-Creation DID
```bash
rooch did create self
```
- Creates a DID using the user's account key.
- User has full control over the DID.
- Supports Secp256k1 account keys.

#### CADOP-Assisted Creation
```bash
rooch did create cadop \
  --user-did-key <USER_DID_KEY> \
  --custodian-service-key <CUSTODIAN_KEY> \
  --custodian-key-type <KEY_TYPE> \
  --sender <CUSTODIAN_ADDRESS>
```
- Custodian assists in DID creation.
- User retains control (did:key controller).
- Custodian provides a service key.

### DID Management

#### Verification Method Management
```bash
# Add verification method
rooch did manage add-vm \
  --did-address <DID_ADDRESS> \
  --fragment <FRAGMENT> \
  --method-type <TYPE> \
  --public-key <MULTIBASE_KEY> \
  --relationships <RELATIONSHIPS>

# Remove verification method
rooch did manage remove-vm \
  --did-address <DID_ADDRESS> \
  --fragment <FRAGMENT>

# Manage verification relationship
rooch did manage add-relationship \
  --did-address <DID_ADDRESS> \
  --fragment <FRAGMENT> \
  --relationship <RELATIONSHIP>
```

#### Service Management
```bash
# Add service
rooch did manage add-service \
  --did-address <DID_ADDRESS> \
  --fragment <FRAGMENT> \
  --service-type <TYPE> \
  --endpoint <URL> \
  --properties <KEY=VALUE>

# Update service
rooch did manage update-service \
  --did-address <DID_ADDRESS> \
  --fragment <FRAGMENT> \
  --service-type <NEW_TYPE> \
  --endpoint <NEW_URL> \
  --properties <NEW_PROPERTIES>
```

### DID Query

```bash
# Query by DID string
rooch did query did <DID_STRING>

# Query by address
rooch did query address <ADDRESS>

# Query by ObjectID
rooch did query object-id <OBJECT_ID>

# Query by controller
rooch did query controller <CONTROLLER_DID>

# Check existence
rooch did query exists <IDENTIFIER>
```

## Development Guide

### Adding a New Verification Method Type

1. **Update Constant Definitions**:
```move
const VERIFICATION_METHOD_TYPE_NEW: vector<u8> = b"NewVerificationKey2024";
```

2. **Extend Multibase Module**:
```move
public fun decode_new_key(multibase_str: &String): Option<vector<u8>> {
    // Implement decoding logic for the new key type
}
```

3. **Update Permission Validation**:
```move
// Add support for the new type in find_verification_method_by_session_key
```

4. **Add Rust Bindings**:
```rust
pub enum VerificationMethodType {
    NewVerificationKey2024,
    // ...
}
```

### Adding a New Service Type

1. **Define Service Type Constant**:
```move
const SERVICE_TYPE_NEW: vector<u8> = b"NewServiceType";
```

2. **Implement Service Validation Logic**:
```move
fun validate_new_service_properties(properties: &SimpleMap<String, String>): bool {
    // Validate service properties
}
```

3. **Update CLI Commands**:
```rust
// Add support for the new type in service management commands
```

### Extending the Permission Model

1. **Add New Verification Relationship**:
```move
const VERIFICATION_RELATIONSHIP_NEW: u8 = 5;
```

2. **Update Permission Check Functions**:
```move
fun assert_authorized_for_new_operation(did_document: &DIDDocument, signer: &signer) {
    // Implement permission validation for the new operation
}
```

3. **Update Rust Type Definitions**:
```rust
pub enum VerificationRelationship {
    NewRelationship = 5,
    // ...
}
```

## Testing and Validation

### Unit Tests

- **Move Contract Tests**: Located in `#[test]` functions within each module.
- **Rust Unit Tests**: Located in the `#[cfg(test)]` module of `crates/rooch-types/src/framework/did.rs`.

### Integration Tests

- **Functional Tests**: `crates/testsuite/features/did.feature`
- **End-to-End Tests**: Covering the complete flow of DID creation, management, and querying.

### Test Coverage

- ✅ DID creation (self-creation, CADOP)
- ✅ Verification method management (add, remove, relationship management)
- ✅ Service management (add, update, remove)
- ✅ Permission validation (various permission levels)
- ✅ Key generation and format conversion
- ✅ DID standard compliance
- ✅ Error handling and edge cases

### Running Tests

```bash
# Move contract tests
rooch move test -p frameworks/rooch-framework

# Rust unit tests
cargo test -p rooch-types did

# Integration tests
rooch test features/did.feature
```

## Security Considerations

### Key Security

1. **Private Key Protection**: Private keys are generated only when necessary; secure storage is recommended.
2. **Session Keys**: Use the session key mechanism to reduce exposure of the main key.
3. **Permission Minimization**: Assign only necessary permission relationships to verification methods.

### Access Control

1. **Multi-Layer Validation**: Dual permission validation on client-side and on-chain.
2. **Time Limits**: Session keys have an expiration mechanism.
3. **Audit Logs**: All DID operations are recorded as events.

### Standard Compliance

1. **W3C Compatibility**: Strict adherence to W3C DID specifications.
2. **Interoperability**: Compatibility with other DID systems.
3. **Forward Compatibility**: Design considers future extensibility.

## Troubleshooting

### Common Errors

1. **ErrorSignerNotDIDAccount**: Signer is not the DID's associated account.
   - Solution: Ensure the correct controller account is used for signing.

2. **ErrorNoSessionKeyInContext**: No session key in the transaction context.
   - Solution: Ensure a valid session key is used for the transaction.

3. **ErrorInsufficientPermission**: Insufficient permission.
   - Solution: Check the permission relationship configuration of the verification method.

4. **ErrorDIDKeyControllerPublicKeyMismatch**: DID:Key controller public key mismatch.
   - Solution: Verify that the did:key identifier matches the provided public key.

### Debugging Tips

1. **View DID Document**: Use the `rooch did query` command to check DID status.
2. **Verify Permissions**: Check the permission relationship configuration of verification methods.
3. **Test Keys**: Use `rooch did keygen` to validate key formats.
4. **View Events**: Check transaction events to understand operation results.

## References

- [W3C DID Core Specification](https://www.w3.org/TR/did-core/)
- [W3C DID Key Method Specification](https://w3c-ccg.github.io/did-method-key/)
- [Multibase Specification](https://github.com/multiformats/multibase)
- [Multicodec Table](https://github.com/multiformats/multicodec/blob/master/table.csv)
- [Rooch Framework Documentation](https://rooch.network/docs)

---

*This document records the complete implementation of the Rooch DID system, providing a technical reference for future developers and AI. If updates are required, please keep this document synchronized with the code implementation.* 
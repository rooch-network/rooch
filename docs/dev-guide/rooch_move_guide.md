# Rooch Move Development Guide

## 1. Overview

This document provides a comprehensive guide for developing Move smart contracts on the Rooch blockchain. It covers special syntax, best practices, core Rooch concepts, and solutions to common problems, tailored for both new developers and AI assistants. Rooch is a Bitcoin-focused Layer 2 solution with unique DID (Decentralized Identity) capabilities.

## 2. Development Environment Setup

### Prerequisites
- Rooch CLI: Ensure the Rooch command-line tool is installed and configured.
- Code Editor: Cursor or VS Code is recommended, with appropriate Move language extensions.

### Compilation Commands
```bash
# Compile the core framework
rooch move build -p frameworks/rooch-framework

# Compile an example project (e.g., basic_object)
rooch move build -p examples/basic_object

# View detailed compilation information
rooch move build -p frameworks/rooch-framework --verbose
```

### Project Structure
```
rooch/
├── .cursorrules                    # Cursor AI project rules
├── .vscode/                        # VS Code configuration
│   ├── settings.json              # Editor settings
│   └── tasks.json                 # Build tasks
├── docs/                          # Project documentation
│   ├── rooch_move_guide.md         # This guide
│   └── ...                        # Other relevant documents
├── frameworks/
│   ├── rooch-framework/sources/   # Core framework modules
│   └── moveos-stdlib/sources/     # MoveOS standard library extensions
├── examples/                      # Example projects
└── Makefile                       # Shortcut commands
```

### Useful Make Commands

The project includes a comprehensive `Makefile` for common development tasks. Below are some of the most useful commands. For a full list, run `make help`.

```bash
# Show all available Makefile targets
make help

# Build all Rust (release profile) and Move components
make build

# Run all Rust and Move tests
make test

# Run all linters (Rust clippy, Rust machete, Move non-ASCII check)
make lint

# Clean all Rust and Move build artifacts
make clean-all

# Full development cycle: clean, build (Rust release, Move), test (Rust, Move)
make dev

# Run all checks typically performed in CI (lint, build, test)
make ci-checks

# Quick compilation check (Rust debug, rooch-framework)
make quick-check

# Build only Move components (core frameworks)
make build-move

# Build only the rooch-framework (Move)
make move-framework

# Run only Move tests (all frameworks and examples)
make test-move

# Run only Move DID module tests (within rooch-framework)
make test-move-did

# Build and run Move example tests
make test-move-examples
```

## 3. Move Language Constraints and Patterns

### 3.1. ASCII Comments Only
All comments in Move source files must be in English using ASCII characters. Non-ASCII characters will cause compilation errors.

```move
// ✅ Correct: English comment
/// Creates a new DID document with the specified parameters.

// ❌ Incorrect: Non-ASCII comments will lead to compilation failure.
// 创建新的DID文档 (This will cause an error)
```
**Reason**: The Move compiler requires all source code files to be ASCII encoded.

### 3.2. Entry Function Parameter Restrictions
Entry functions in Move do not support `Option<T>` type parameters directly.

```move
// ❌ Incorrect: Entry functions do not support Option<T> parameters.
public entry fun create_did(
    signer: &signer,
    optional_param: Option<String>  // Compilation error
) { /* ... */ }

// ✅ Correct: Use separate entry functions for optional parameters.
public entry fun create_did_simple(signer: &signer) { /* ... */ }
public entry fun create_did_with_param(signer: &signer, param: String) { /* ... */ }
```

### 3.3. Friend Function Pattern
To allow controlled access between modules, Move uses `public(friend)`. The friendship must be declared in the module header.

```move
// Declare friend relationship at the module level
module my_module {
    friend rooch_framework::another_module;

    public(friend) fun internal_function_for_friend() { /* ... */ }
}
```

## 4. Core Rooch Concepts

### 4.1. Rooch Address System

Rooch utilizes an account system based on Bitcoin addresses, a core feature of its Layer 2 architecture for Bitcoin.

**Mapping Relationship:**
```
Bitcoin Ecosystem             Rooch Ecosystem
    ↓                          ↓
Bitcoin Private Key ──┐              Rooch Account
Bitcoin Public Key  ──┼── Signature  ──→ Rooch Transaction
Bitcoin Address     ──┘   Validation     Rooch Address
```

**Key Components:**
-   `bitcoin_validator.move`: Validates Bitcoin wallet signatures, confirms public key-Bitcoin address correspondence, and maps Bitcoin addresses to Rooch addresses.
-   `auth_validator.move`: Manages transaction validation results, stores validated Bitcoin addresses in the transaction context, and provides address query interfaces.
-   `bitcoin_address.move`: Handles various Bitcoin address formats, provides address verification and conversion functions, and implements the Bitcoin address to Rooch address mapping.

#### 4.1.1. Address Derivation Mechanism

**1. Bitcoin Address Generation:**
Different Bitcoin address types have distinct generation methods:
-   **P2PKH (Legacy)**: `hash160(public_key)` + network prefix. Example: `1...`
-   **P2WPKH (SegWit v0)**: `hash160(public_key)` + SegWit version. Example: `bc1q...`
-   **P2TR (Taproot)**: `taproot_tweak(x_only_public_key, merkle_root)` + Taproot version. Example: `bc1p...`
-   **P2SH-P2WPKH (Nested SegWit)**: `hash160(witness_script)` + script prefix. Example: `3...`

**2. Rooch Address Mapping:**
The Rooch address is derived from a Bitcoin address using a hash function.
```move
// In bitcoin_address.move (conceptual)
public fun to_rooch_address(addr: &BitcoinAddress): address {
    // The actual implementation uses specific hashing defined in MoveOS
    let hash = moveos_std::hash::blake2b256(&addr.bytes); // addr.bytes represents raw bytes of the Bitcoin address
    moveos_std::bcs::to_address(hash)
}
```
-   Uses Blake2b-256 hash algorithm.
-   Input is the raw byte representation of the Bitcoin address.
-   Output is a 32-byte Rooch address.
-   The mapping is deterministic.

#### 4.1.2. Transaction Verification Flow

1.  **Transaction Submission**: User's Bitcoin wallet signs a transaction and submits it to the Rooch network.
2.  **Validator Processing**: `bitcoin_validator.move` performs:
    *   Signature validation.
    *   Verification of Bitcoin address against the public key.
    *   Verification that the derived Rooch address matches the sender.
3.  **Context Storage**: The validated Bitcoin address is stored in the transaction context by `auth_validator.move`.

```move
// Conceptual flow in bitcoin_validator.move
public(friend) fun validate_transaction_authenticator(
    transaction_authenticator_payload: vector<u8>,
    tx_hash: vector<u8>, // Transaction hash
    sender_rooch_address: address // Expected sender's Rooch address
) : BitcoinAddress {
    // 1. Deserialize payload
    let auth_payload = auth_payload::from_bytes(transaction_authenticator_payload);
    
    // 2. Validate signature (pseudo-code)
    // validate_signature(&auth_payload, &tx_hash);
    
    // 3. Extract Bitcoin address and public key from payload
    let bitcoin_address_str = auth_payload::get_bitcoin_address_str(&auth_payload);
    let public_key_bytes = auth_payload::get_public_key_bytes(&auth_payload);
    let bitcoin_addr_from_payload = bitcoin_address::from_string(&bitcoin_address_str); // Assuming conversion

    // 4. Verify public key against Bitcoin address
    assert!(
        bitcoin_address::verify_bitcoin_address_with_public_key(&bitcoin_addr_from_payload, &public_key_bytes),
        // Use appropriate error from error.move or auth_validator
        auth_validator::error_validate_invalid_authenticator() 
    );
    
    // 5. Verify derived Rooch address matches sender
    let derived_rooch_address = bitcoin_address::to_rooch_address(&bitcoin_addr_from_payload);
    assert!(derived_rooch_address == sender_rooch_address, auth_validator::error_validate_invalid_authenticator());
    
    bitcoin_addr_from_payload // Return the validated BitcoinAddress object
}

// In auth_validator.move
// struct TxValidateResult has copy, store, drop {
//     // ... other fields
//     bitcoin_address: Option<BitcoinAddress>, // Stores the validated Bitcoin address
// }
// public fun get_bitcoin_address_from_ctx(): BitcoinAddress { /* ... */ }
```

#### 4.1.3. Application Layer Integration (e.g., DID System)

Applications retrieve the validated Bitcoin address from the transaction context.

```move
// Example in a DID module
fun verify_pk_for_did_account(
    did_account_address: address, // The Rooch address of the DID account
    user_provided_public_key_multibase: &String
) {
    // 1. Decode user-provided public key
    let pk_bytes_opt = multibase::decode_secp256k1_key(user_provided_public_key_multibase); // Or Ed25519
    assert!(option::is_some(&pk_bytes_opt), ErrorInvalidPublicKeyMultibaseFormat);
    let pk_bytes = option::destroy_some(pk_bytes_opt);

    // 2. Get validated Bitcoin address from transaction context
    let bitcoin_address_from_ctx = auth_validator::get_bitcoin_address_from_ctx();

    // 3. Verify that the user-provided public key corresponds to the Bitcoin address from context
    assert!(
        bitcoin_address::verify_bitcoin_address_with_public_key(&bitcoin_address_from_ctx, &pk_bytes),
        ErrorPublicKeyMismatchWithContextAddress // Example error
    );

    // 4. Verify that the Bitcoin address from context maps to the target DID's Rooch account address
    let derived_rooch_address = bitcoin_address::to_rooch_address(&bitcoin_address_from_ctx);
    assert!(derived_rooch_address == did_account_address, ErrorAddressMismatchWithDIDAccount);
}

#### 4.1.4. Supported Bitcoin Address Types

| Type        | Prefix (Mainnet) | Encoding | Key Features                          | Use Case        |
|-------------|------------------|----------|---------------------------------------|-----------------|
| P2PKH       | `1...`           | Base58   | Legacy, widely compatible             | Traditional     |
| P2SH-P2WPKH | `3...`           | Base58   | SegWit compatibility (nested)         | Transition      |
| P2WPKH      | `bc1q...`        | Bech32   | Native SegWit v0, lower fees          | Modern SegWit   |
| P2TR        | `bc1p...`        | Bech32m  | Taproot (SegWit v1), privacy, scripts | Advanced        |

Rooch's `bitcoin_address::verify_bitcoin_address_with_public_key` function is designed to handle these common formats.

#### 4.1.5. Address System Best Practices & Security

-   **Always use context-derived addresses**: For any operation requiring the sender's Bitcoin address, retrieve it via `auth_validator::get_bitcoin_address_from_ctx()`. Do not trust unvalidated addresses passed as arguments.
-   **Verify public keys**: When a public key is provided (e.g., for a DID), verify it against the context-derived Bitcoin address using `bitcoin_address::verify_bitcoin_address_with_public_key`.
-   **Avoid direct derivation from AuthKey**: Do NOT derive a Rooch address directly from an `AuthenticationKey` (e.g., from `session_key::*_public_key_to_authentication_key`) and assume it's the user's primary Rooch account address. The primary Rooch account address is always derived from a Bitcoin address.
    ```move
    // ❌ Incorrect: Deriving address from authentication key for account identification
    // let auth_key = session_key::secp256k1_public_key_to_authentication_key(&pk_bytes);
    // let wrong_rooch_addr = address::from_bytes(auth_key); // This is NOT the user's main Rooch address

    // ✅ Correct: Using context from auth_validator
    let bitcoin_address = auth_validator::get_bitcoin_address_from_ctx();
    let correct_rooch_addr = bitcoin_address::to_rooch_address(&bitcoin_address);
    ```
-   **Replay Protection**: Rooch employs standard mechanisms like nonces, timestamps, and chain ID validation to prevent replay attacks.

### 4.2. DID (Decentralized Identity) System

Rooch includes a DID system built on Move.

#### 4.2.1. Permission Model
-   `capabilityDelegation`: Permission to manage keys (add/remove verification methods).
-   `capabilityInvocation`: Permission to use services or invoke actions associated with the DID.
-   `authentication`: Permission to authenticate as the DID.

#### 4.2.2. Session Key Support
Session keys provide temporary, scoped access. Rooch supports:
-   **Ed25519**: `session_key::ed25519_public_key_to_authentication_key(&pk_bytes)`
-   **Secp256k1**: `session_key::secp256k1_public_key_to_authentication_key(&pk_bytes)`

Authentication keys are derived using a blake2b256 hash with a scheme prefix.

#### 4.2.3. Common DID Patterns

**Permission Verification (Conceptual):**
```move
fun assert_authorized_for_capability_delegation(
    did_document_data: &DIDDocument, // Assuming a DIDDocument struct
    did_signer: &signer // Could also be based on session key from context
) {
    let sender_address = signer::address_of(did_signer);
    let did_account_address = account::account_address_from_capability(&did_document_data.account_cap); // Example
    assert!(sender_address == did_account_address, ErrorSignerNotDIDAccount);

    // If using session keys:
    let session_key_auth_key_opt = auth_validator::get_session_key_from_ctx_option();
    assert!(option::is_some(&session_key_auth_key_opt), ErrorNoSessionKeyInContext);
    let session_key_auth_key = option::destroy_some(session_key_auth_key_opt);
    
    // Find the verification method associated with the session key
    let vm_fragment_opt = find_verification_method_by_auth_key(did_document_data, &session_key_auth_key);
    assert!(option::is_some(&vm_fragment_opt), ErrorSessionKeyNotFoundInDID);
    let vm_fragment = option::destroy_some(vm_fragment_opt); // This is an identifier/fragment like "#key-1"

    // Check if this verification method has capabilityDelegation permission
    assert!(
        vector::contains(&did_document_data.capability_delegation, &vm_fragment),
        ErrorInsufficientPermissionForDelegation
    );
}
```

**Object Operations:**
```move
// Borrow a mutable reference to an object
let obj_ref_mut = object::borrow_mut_object_extend<MyObjectType>(object_id);
let data_mut = object::borrow_mut(obj_ref_mut);
// ... modify data_mut ...

// Borrow an immutable reference to an object
let obj_ref = object::borrow_object<MyObjectType>(object_id);
let data_immut = object::borrow(obj_ref);
// ... read data_immut ...
```

## 5. Cryptography and Encoding

### 5.1. Multibase Support
Rooch Move utilizes `multibase` for encoding public keys.
-   **Encoding**:
    ```move
    let encoded_ed_key = multibase::encode_ed25519_key(&public_key_bytes);     // z... (base58btc)
    let encoded_secp_key = multibase::encode_secp256k1_key(&public_key_bytes); // z... (base58btc)
    ```
-   **Decoding**:
    ```move
    let decoded_ed_key_opt = multibase::decode_ed25519_key(&multibase_string);
    let decoded_secp_key_opt = multibase::decode_secp256k1_key(&multibase_string);
    ```
-   **Supported Formats**:
    -   `base58btc` (prefix 'z'): Default for keys.
    -   `base64pad` (prefix 'M'): RFC4648 with padding.
    -   `base16` (prefix 'f'): Hexadecimal.

### 5.2. Key Types and Lengths
-   Ed25519 public keys: 32 bytes.
-   Secp256k1 compressed public keys: 33 bytes.
-   Always validate key lengths before processing.

## 6. Error Handling

### 6.1. Error Constant Pattern
Define error codes as constants within your modules.
```move
// In your_module.move
const ErrorDocumentNotFound: u64 = 1;
const ErrorAlreadyExists: u64 = 2;
const ErrorUnauthorizedAccess: u64 = 3;
// ... and so on, incrementing sequentially.

// Document errors clearly
/// The requested DID document does not exist.
const ErrorDIDDocumentNotExist: u64 = 1;
/// The DID document already exists and cannot be re-created.
const ErrorDIDAlreadyExists: u64 = 2;
```

### 6.2. Assertion Pattern
Use `assert!` with the defined error code directly. The `error` module is not used.
```move
assert!(condition, ErrorCode); // e.g., ErrorDocumentNotFound

// Example from did.move
assert!(
    option::is_some(&pk_bytes_opt), 
    ErrorInvalidPublicKeyMultibaseFormat
);

// For generic aborts, use the abort keyword directly
abort ErrorCode; // e.g., abort ErrorUnauthorizedAccess
```

## 7. Testing

Robust testing is crucial for smart contract development.

### 7.1. Compilation and Verification
Always compile your code after changes to catch syntax and type errors early. You can use the `Makefile` for convenience.
```bash
# Build a specific Move framework (e.g., rooch-framework)
make move-framework

# Build all core Move frameworks
make build-move

# For individual package compilation using Rooch CLI (typically after Rust build for the CLI):
# cargo run --profile optci --bin rooch -- move build -p frameworks/rooch-framework
# cargo run --profile optci --bin rooch -- move build -p examples/my_example
```
Note: Move compilation and testing via `make` targets will use an optimized Rooch CLI built with the `optci` profile.

### 7.2. Unit Tests
-   Use `#[test]` attribute for test functions.
-   Use `#[expected_failure(abort_code = ...)]` for testing error conditions.
-   Use `#[test_only]` for functions and `use` statements exclusively for tests.

```move
#[test_only]
use std::signer; // For test-only signer usage

#[test_only]
fun setup_test_scenario(): signer {
    genesis::init_for_test(); // Initialize genesis state for tests
    // ... other setup ...
    signer::dummy() // Example
}

#[test]
fun test_my_function_success() {
    let signer = setup_test_scenario();
    // ... test logic ...
    my_module::my_function(&signer, /* args */);
    // ... assertions ...
}

#[test]
#[expected_failure(abort_code = my_module::ErrorMySpecificError)]
fun test_my_function_failure_specific_error() {
    let signer = setup_test_scenario();
    // ... test logic that should fail ...
    my_module::my_function_that_fails(&signer, /* args */);
}
```

### 7.3. Test Commands
Use the `Makefile` for running tests. These commands utilize an optimized Rooch CLI.

```bash
# Run all Move tests (frameworks and examples)
make test-move

# Run tests for all core Move frameworks
make test-move-frameworks

# Run tests in a specific Move module or matching a name (using rooch cli directly with -f filter)
# Example: Test functions containing "did_test" in rooch-framework
# cargo run --profile optci --bin rooch -- move test -p frameworks/rooch-framework -f did_test

# Run a specific Move test function
# cargo run --profile optci --bin rooch -- move test -p frameworks/rooch-framework module_name::test_function_name

# Run Move DID module tests (shortcut in Makefile)
make test-move-did

# Build and run all Move example tests
make test-move-examples

# For more specific Rooch CLI test options:
# rooch move test -p frameworks/rooch-framework --list
# rooch move test -p frameworks/rooch-framework --state_on_error
```

### 7.4. Mocking for Tests

**Session Key Mocking (DID Example):**
```move
#[test_only]
fun mock_ed25519_session_key_for_test(): vector<u8> {
    // This is a simplified example; actual key generation might be more complex
    // or use fixed test vectors.
    let pk_bytes = vector[0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31];
    let auth_key = session_key::ed25519_public_key_to_authentication_key(&pk_bytes);
    auth_validator::set_simple_tx_validate_result_for_testing(option::some(auth_key));
    auth_key
}

#[test_only]
fun mock_bitcoin_address_for_test() {
    // For tests not requiring real crypto, you might set a random or fixed Bitcoin address
    // This depends on how auth_validator mocking is structured.
    // auth_validator::set_random_tx_validate_result_for_testing(option::none()); // Clears session key, sets random BTC address
    // let bitcoin_address = auth_validator::get_bitcoin_address_from_ctx();
    // This part needs alignment with actual auth_validator test utilities.
}
```
*Note: Actual mocking utilities in `auth_validator` should be used as available.*

## 8. Development Best Practices & Conventions

### 8.1. Code Quality Standards
-   **Clarity**: Write clear, descriptive names for functions, variables, and types.
-   **Modularity**: Design small, focused modules and functions.
-   **Comments**: Document complex logic, public APIs, and non-obvious behaviors in English.
-   **Error Handling**: Implement comprehensive error handling for all edge cases.
-   **Ownership**: Follow Move's ownership and borrowing rules strictly.
-   **Prefer Composition**: Favor composition over inheritance-like patterns where possible.

### 8.2. Code Review Checklist
-   [ ] All comments are in English and ASCII.
-   [ ] Entry function parameters do not use `Option<T>`.
-   [ ] `friend` relationships are correctly declared and used.
-   [ ] Error handling is comprehensive and uses defined error codes.
-   [ ] All code compiles successfully (`rooch move build`).
-   [ ] Tests cover success and failure cases, and pass (`rooch move test`).
-   [ ] Permission checks are correctly implemented (e.g., for DID operations).
-   [ ] Object lifecycle and borrowing rules are respected.
-   [ ] Public key and address verifications are sound.
-   [ ] No direct Rooch address derivation from `AuthenticationKey` for account identification.
-   [ ] Sensitive operations use data from `auth_validator` context where appropriate.
-   [ ] Cryptographic key lengths are validated.

## 9. Performance Optimization

-   **Avoid Large Vector Operations**: Prefer `Table` for frequent lookups or when dealing with large collections if appropriate.
-   **Minimize Object Borrow Time**: Release object references as soon as they are no longer needed.
-   **Batch Operations**: Where feasible, design functions to handle batch operations to reduce transaction overhead for multiple similar actions.
-   **Lazy Initialization**: Only create complex data structures when they are actually needed.

## 10. Tooling and Editor Integration

### 10.1. VS Code Integration
The project is configured for VS Code with:
-   Tasks for common operations (Build, Test, Clean). Access via `Cmd+Shift+P` -> "Tasks: Run Task".
    -   `Rooch: Build Framework` (Consider aligning with `make move-framework` or `make quick-check`)
    -   `Rooch: Test Framework` (Consider aligning with `make test-move-frameworks` or a specific framework test)
-   Shortcut `Cmd+Shift+B` for the default build task (ensure it aligns with a suitable `make` target, e.g., `make quick-check` or `make build`).
-   Settings for Move file syntax highlighting, ASCII enforcement, auto-formatting.

### 10.2. Cursor AI Enhancement
This project includes a `.cursorrules` file. This helps Cursor AI to:
-   Understand Rooch Move specific syntax and patterns.
-   Adhere to project coding conventions.
-   Provide context-aware code suggestions.
-   Recall common compilation commands and development patterns.

## 11. FAQ / Common Issues & Solutions

1.  **Problem: Compilation error due to non-ASCII (e.g., Chinese) comments.**
    *   **Solution**: Change all comments to English using only ASCII characters. Use `make lint` to help identify such issues.

2.  **Problem: `Option<T>` type used as a parameter in an `public entry fun`.**
    *   **Solution**: Refactor into multiple entry functions, one for each variant (e.g., one with the parameter, one without), or accept all necessary fields and let callers pass default/empty values for optional parts if applicable.

3.  **Problem: Cannot call a `public(friend)` function from another module.**
    *   **Solution**: Ensure the calling module is declared as a `friend` in the header of the module containing the `public(friend)` function.

4.  **Problem: Object borrowing conflicts (e.g., "cannot borrow `_` as mutable because it is already borrowed as immutable").**
    *   **Solution**: Carefully review code logic to ensure references are not held longer than necessary. Ensure mutable and immutable borrows of the same data do not overlap in scope. Release borrows explicitly if needed or restructure code flow.

5.  **Problem: Address mismatch errors.**
    *   **Solution**:
        *   Ensure you are using `auth_validator::get_bitcoin_address_from_ctx()` to get the validated Bitcoin address.
        *   Verify that `bitcoin_address::to_rooch_address()` is used for mapping to Rooch address.
        *   Do not confuse `AuthenticationKey`-derived addresses with Bitcoin-derived Rooch account addresses.

## 12. Bitcoin Integration Specifics

Rooch's deep integration with Bitcoin influences several aspects of Move development.

### 12.1. Secp256k1 as Default
-   Rooch natively supports Secp256k1, aligning with Bitcoin's cryptography. This is crucial for seamless Bitcoin wallet integration.
-   Session keys can be derived from Secp256k1 public keys:
    ```move
    // Example: Secp256k1 session key registration
    // let auth_key = session_key::secp256k1_public_key_to_authentication_key(&pk_bytes);
    // session_key::create_session_key_internal(signer, app_name, app_url, auth_key, scopes, interval_seconds);
    ```

### 12.2. Address Formats
-   Rooch addresses are Bech32 encoded.
-   The system is compatible with Bitcoin ecosystem address formats (P2PKH, P2WPKH, P2TR, P2SH-P2WPKH) for deriving the Rooch address. The underlying Bitcoin address type is abstracted away after initial validation and mapping.

---
This guide will be updated continuously as the Rooch project evolves. Developers are encouraged to consult this document before contributing code.

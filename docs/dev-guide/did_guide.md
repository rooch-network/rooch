# Comprehensive Rooch DID System Documentation

## 1. Overview

The Rooch DID (Decentralized Identifier) implementation is based on the [NIP-1](https://raw.githubusercontent.com/nuwa-protocol/NIPs/refs/heads/main/nips/nip-1.md) specification, aiming to provide a robust and flexible decentralized identity solution. In this design, each DID is not just an identifier but an independent **on-chain entity**, typically represented as a **Move Object (`Object<DIDDocument>`)**. This allows DIDs to have state, associated assets, and control operations through their internally defined verification methods, including managing a Rooch account specifically created for them.

This model follows the core principle of "a single primary identity + multiple operational sub-keys" while enhancing the capability of DIDs as a foundation for autonomous agent identities.

## 1.A System Architecture

The Rooch DID system employs a layered architecture, primarily comprising:

### Core Layer (Move Contracts)
- **DID Module** (`frameworks/rooch-framework/sources/did.move`):
  - Creation, management, and querying of DID documents.
  - Management of verification methods and services.
  - Permission validation and access control.
  - Integration with the Rooch session key system.
- **Multibase Key Module** (`frameworks/moveos-stdlib/sources/multibase_key.move`):
  - **Handles `did:key` identifiers only.** It provides key-type aware decoding for cryptographic keys (Ed25519, Secp256k1, Secp256r1) by parsing their respective multicodec prefixes from a `did:key` string. The primary function is `decode_with_type`.
- **Multibase Codec Module** (`frameworks/moveos-stdlib/sources/multibase_codec.move`):
  - **Handles standard multibase strings.** It implements the basic multibase encoding/decoding logic (e.g., base58btc, base64pad) for raw byte vectors, such as those found in the `publicKeyMultibase` field of a verification method. It has no knowledge of key types or multicodec prefixes. Key functions are `encode_base58btc` and `decode`.
- **DID Key Module** (`frameworks/moveos-stdlib/sources/did_key.move`):
  - A utility module built on top of `multibase_key` that provides helper functions to generate `did:key` method identifiers from raw public keys.

### Application Layer (Rust CLI)
- **DID Command Module** (`crates/rooch/src/commands/did/`):
  - DID creation (self-creation, CADOP-assisted creation).
  - DID management (verification methods, services, relationships).
  - DID information querying.
  - Key generation utility.
- **Type Definitions** (`crates/rooch-types/src/framework/did.rs`):
  - Rust and Move type bindings.
  - Serialization/deserialization support.

## 1.B DID Standard Compliance

The Rooch DID system adheres to the following core standards:
- [W3C DID Core Specification v1.0](https://www.w3.org/TR/did-core/)
- [W3C DID Key Method v1.0](https://w3c-ccg.github.io/did-method-key/)
- [Multibase Data Format Specification](https://datatracker.ietf.org/doc/html/draft-multiformats-multibase-03) (referenced by W3C DID Spec)
- [Multicodec Table](https://github.com/multiformats/multicodec/blob/master/table.csv) (used for defining key types)

### DID:Key Format
According to the W3C DID Key specification, the general format for the `did:key` method is:
`did:key:<multibase-prefix><multicodec-value-public-key-type><raw-public-key-bytes>`

Rooch primarily focuses on the `did:key` representation for the following public key types:

#### Multicodec Prefixes and Encoding
- **Ed25519**:
  - Multicodec name: `ed25519-pub`
  - Multicodec value (varint): `0xed01`
  - Typically encoded using Base58BTC (multibase prefix `z`), resulting in identifiers like `z6Mk...`.
- **Secp256k1**:
  - Multicodec name: `secp256k1-pub`
  - Multicodec value (varint): `0xe701`
  - Typically encoded using Base58BTC (multibase prefix `z`), resulting in identifiers like `zQ3s...`.
- **Secp256r1 (P-256)**:
  - Multicodec name: `p256-pub`
  - Multicodec value (varint): `0x1200`
  - Typically encoded using Base58BTC (multibase prefix `z`), resulting in identifiers like `z2...`.

#### Examples
```
# Ed25519 (32-byte public key)
did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH

# Secp256k1 (compressed format, 33-byte public key)
did:key:zQ3shokFTS3brHcDQrn82RUDfCZESWL1ZdCEJwekUDPQiYBme

# Secp256r1 / P-256 (compressed format, 33-byte public key)
did:key:z28d1ZBinrPjY86bK9tA1wH3n5GfT8GDU1s2gX3H4WA2v3S6B
```
The `did.move` module uses `multibase_key::decode_with_type` to parse these `did:key` identifiers.

## 2. Rooch DID Contract (`did.move`) Design

### 2.1 Core Design Principles

- **DID as Object**: The core data of each DID (DID document) is stored and managed as a Move Object.
- **Single Primary Identity**: Each agent has one primary DID (represented by the ID of `Object<DIDDocument>`), signifying its unique identity.
- **Independent Account Association**: Each `Object<DIDDocument>` is associated with a newly generated Rooch smart contract account upon creation, enabling it to initiate transactions and hold assets. Control of this account belongs to this DID.
- **Key-Controlled Account**: Verification methods in the `authentication` relationship of the DID document are automatically registered as `session_key`s for the associated Rooch account, allowing DID keys to authorize and initiate transactions on behalf of that DID.
- **Controller Authorization**: Modifications to `Object<DIDDocument>` (e.g., key management, service updates) are authorized by DIDs specified in its `controller` field, using keys with `capabilityDelegation` permission.
- **Multiple Operational Keys & Fine-Grained Permissions**: Supports various verification methods and relationships for flexible permission control.
- **Passkey Bootstrap**: Compatible with NIP-1 Passkey bootstrap principles.

### 2.2 System Architecture

#### `DIDDocument` as the Core Object

- `DIDDocument` is not a `Resource` stored under a user account but an independent `Object` on the chain, possessing a globally unique `ObjectID`. This `ObjectID` can serve as the primary on-chain reference for the DID within the Rooch ecosystem.
- **Creation**: Created via `create_did_object_internal(...)` (internal core function). This function will:
    1. Create a new Rooch account and obtain its `AccountCap`.
    2. Construct `DIDDocument` data, storing the new account address and securely associating the `AccountCap` with this DID (stored within `DIDDocument`).
    3. Set the initial `controller`(s) (specified by the creator) and initial verification methods (usually the controller's key) into the `DIDDocument`.
    4. Encapsulate the `DIDDocument` data into a new `Object` and return its `ObjectID`. This `ObjectID` is deterministically generated using `object::custom_object_id` with the DID's unique identifier string (e.g., the Bech32 address of the associated account) as a seed.
- **Control and Permissions**: Any modification to `Object<DIDDocument>` requires the signer to be a DID listed in `DIDDocument.controller`, and the key used must have the corresponding `capabilityDelegation` permission.

#### Deep Integration of DID and Rooch Accounts

- **Independent Account**: Each `Object<DIDDocument>` is strongly associated with a newly created Rooch account.
- **`AccountCap` Management**: The new account's `AccountCap` is stored directly as a field (`account_cap: AccountCap`) in the `DIDDocument` object. This means `Object<DIDDocument>` holds the full capability of its associated account.
    - **Permission-Controlled Usage**: Direct use of `AccountCap` is strictly permission-controlled. It requires invoking restricted functions using a verification method with special "account management" permission, authorized by the `DIDDocument`'s `controller`, to grant such operations.
- **`authentication` Key as `session_key`**: When a `verificationMethod` is added to the `authentication` verification relationship of `DIDDocument`, its public key is automatically registered as a `session_key` for the associated Rooch account.
    *   This allows any entity possessing the corresponding `authentication` private key to act on behalf of the DID (through its associated account) to initiate transactions.
    *   The `session_key` registration operation itself also requires invoking restricted functions, using the `AccountCap` to generate the necessary `signer`.

#### Global DID Registry (`DIDRegistry`)

- `DIDRegistry` is a global **named object**.
- **Main Function**: Used to discover and manage metadata of `Object<DIDDocument>`, particularly the mapping from controllers to the list of DIDs they control.
- **Core Mapping**:
    *   `controller_to_dids: Table<String, vector<String>>`: Maps a Controller DID (as a string) to a list of DID strings it controls.
- *Note 1: The `ObjectID` of a `DIDDocument` object is deterministically generated using `object::custom_object_id` with its unique string identifier (e.g., the Bech32 address of the associated account) as a seed. This eliminates the need for an explicit mapping table from identifier to `ObjectID` in `DIDRegistry`.*
- *Note 2: Regarding service providers (e.g., custodians) tracking the DIDs they manage: For scalability, we do not maintain a `service_registrations` mapping on-chain. Service providers like custodians should track the DIDs they service through off-chain mechanisms (e.g., listening to contract events, maintaining their own index databases).*

### 2.3 Data Structures

#### Constant Definitions

```move
// Verification relationship types
const VERIFICATION_RELATIONSHIP_AUTHENTICATION: u8 = 0;
const VERIFICATION_RELATIONSHIP_ASSERTION_METHOD: u8 = 1;
const VERIFICATION_RELATIONSHIP_CAPABILITY_INVOCATION: u8 = 2;
const VERIFICATION_RELATIONSHIP_CAPABILITY_DELEGATION: u8 = 3;
const VERIFICATION_RELATIONSHIP_KEY_AGREEMENT: u8 = 4;

// Verification method types
const VERIFICATION_METHOD_TYPE_ED25519: vector<u8> = b"Ed25519VerificationKey2020";
const VERIFICATION_METHOD_TYPE_SECP256K1: vector<u8> = b"EcdsaSecp256k1VerificationKey2019";
const VERIFICATION_METHOD_TYPE_SECP256R1: vector<u8> = b"EcdsaSecp256r1VerificationKey2019";
```

#### DID Identifier (`DID` struct)
```move
struct DID has store, copy, drop {
    method: String,     // DID method (e.g., "rooch", "key")
    identifier: String, // DID identifier (e.g., "0x123..." or specific string)
}
```

#### Verification Method ID (`VerificationMethodID` struct)
```move
struct VerificationMethodID has store, copy, drop {
    did: DID,
    fragment: String,  // Fragment part of the ID (e.g., "key-1")
}
```

#### Service ID (`ServiceID` struct)
```move
struct ServiceID has store, copy, drop {
    did: DID,
    fragment: String,  // Fragment part of the ID (e.g., "llm-gateway")
}
```

#### Verification Method (`VerificationMethod` struct)
```move
struct VerificationMethod has store, copy, drop {
    id: VerificationMethodID,
    type: String,      // Verification method type (e.g., "Ed25519VerificationKey2020")
    controller: DID,   // Controller of this verification method
    public_key_multibase: String, // Public key in multibase format (string type)
}
```

#### Service Definition (`Service` struct)
```move
struct Service has store, copy, drop {
    id: ServiceID,
    type: String,      // Service type (e.g., "LLMGatewayNIP9", "CustodianServiceCADOP")
    service_endpoint: String, // URL or identifier for the service
    properties: SimpleMap<String, String>, // Additional service properties
}
```

#### DID Document (`DIDDocument` struct - Core data of the Object)
```move
struct DIDDocument has key {
    id: DID, // DID subject identifier, usually corresponds to the ObjectID or its derived identifier
    controller: vector<DID>, // List of DIDs controlling this DID Document
    verification_methods: SimpleMap<String, VerificationMethod>, // fragment -> VerificationMethod
    authentication: vector<String>,          // fragments, client combines to did#fragment
    assertion_method: vector<String>,        // fragments
    capability_invocation: vector<String>,   // fragments
    capability_delegation: vector<String>,   // fragments
    key_agreement: vector<String>,           // fragments
    services: SimpleMap<String, Service>,        // fragment -> Service
    account_cap: AccountCap, // AccountCap of the associated Rooch account
    also_known_as: vector<String>, // List of aliases or other identifier URIs for the DID subject (W3C alsoKnownAs)
    // created_timestamp and updated_timestamp use the Object system's built-in timestamps
}
```

#### DID Registry (`DIDRegistry` struct - Core data of the named object)
```move
struct DIDRegistry has key {
    /// Controller DID string -> vector of DID strings it controls
    controller_to_dids: Table<String, vector<String>>,
}
```

### 2.4 Core Functions

#### DID Creation

##### `create_did_object_internal` (Internal Core Function)
This function is central to all DID object creation logic.
```move
fun create_did_object_internal(
    creator_account_signer: &signer,    // The Rooch account signer creating this DID Object (pays gas)
    doc_controller: DID,                // DID Document controller field value (e.g., the user's did:key)
    user_vm_pk_multibase: String,       // User public key (multibase format)
    user_vm_type: String,               // VM type, e.g., "Ed25519VerificationKey2020"
    user_vm_fragment: String,           // VM fragment, e.g., "key-1"
    user_vm_relationships: vector<u8>,  // Verification relationships this VM should have
    service_provider_controller_did: Option<DID>, // Service provider (e.g., custodian) DID
    service_vm_pk_multibase: Option<String>,      // Service VM public key
    service_vm_type: Option<String>,              // Service VM type
    service_vm_fragment: Option<String>           // Service VM fragment
): ObjectID
```
**Logic**:
1.  Create a new Rooch account and get its `AccountCap`.
2.  Generate a `DID` in the format `did:rooch:<address>` based on the new account address.
3.  Create the base `DIDDocument` structure with the specified `doc_controller`.
4.  Process the user's initial verification method (VM), setting its controller to `doc_controller`, and add it to the appropriate relationship lists based on `user_vm_relationships`. If a key type supports session keys and is added to `VERIFICATION_RELATIONSHIP_AUTHENTICATION`, it will automatically be registered as a Rooch session key.
5.  If service provider information is provided, create a service VM with its controller as `service_provider_controller_did`, and add its fragment to the `capability_invocation` relationship.
6.  Encapsulate `DIDDocument` as an Object, transferring ownership to the new account.
7.  Update the `controller_to_dids` mapping in `DIDRegistry`.
8.  Emit a `DIDCreatedEvent`.

##### User Self-Creation of DID (`create_did_object_for_self_entry` and `create_did_object_for_self`)
```move
public entry fun create_did_object_for_self_entry(
    creator_account_signer: &signer,        // User's own Rooch account signer
    account_public_key_multibase: String,   // User's account public key (Secp256k1)
)
// Calls internal:
public fun create_did_object_for_self(
    creator_account_signer: &signer,
    account_public_key_multibase: String,
) : ObjectID
```
**Features**:
-   User creates a DID using their own Rooch account and associated Secp256k1 public key.
-   Automatically verifies that the provided public key corresponds to the creator's account (via `verify_public_key_matches_account`).
-   The `doc_controller` is set to the newly created `did:rooch:<new_did_address>`.
-   `user_vm_relationships` include all major permissions (`AuthN`, `AssertM`, `CapInv`, `CapDel`).
-   No third-party service provider VM involved.

Public Key Verification Mechanism (`verify_public_key_matches_account`):
1.  Decode the incoming `account_public_key_multibase` using `multibase_codec::decode`.
2.  Get the validated Bitcoin address from the transaction context using `auth_validator::get_bitcoin_address_from_ctx_option()`.
3.  Verify that the decoded public key matches this Bitcoin address (`bitcoin_address::verify_bitcoin_address_with_public_key`).
4.  Verify that the Rooch address derived from this Bitcoin address (`bitcoin_address::to_rooch_address`) matches the address of `creator_account_signer`.

##### Custodian-Assisted DID Creation (`create_did_object_via_cadop_with_did_key_entry` and `create_did_object_via_cadop_with_did_key`)
```move
public entry fun create_did_object_via_cadop_with_did_key_entry(
    custodian_signer: &signer,              // Custodian's Rooch account, pays gas
    user_did_key_string: String,            // User's did:key string (e.g., "did:key:zABC...")
    custodian_service_pk_multibase: String, // Custodian's service public key for this user
    custodian_service_vm_type: String       // Custodian service VM type
)
// Calls internal:
public fun create_did_object_via_cadop_with_did_key(
    custodian_signer: &signer,
    user_did_key_string: String,
    custodian_service_pk_multibase: String,
    custodian_service_vm_type: String
): ObjectID
```
**Features** (following NIP-3 CADOP principles):
-   The custodian (`custodian_signer`) pays gas for creation assistance.
-   The user's identity is represented by `user_did_key_string` (a `did:key` string). The `doc_controller` is set to this `did:key`, so the user retains control.
-   **Key Parsing Logic**:
    1.  The identifier part of `user_did_key_string` is parsed using `multibase_key::decode_with_type` to extract the raw public key bytes and key type.
    2.  The raw public key bytes are then re-encoded using `multibase_codec::encode_base58btc` to create a standard `publicKeyMultibase` string for the user's initial verification method.
-   The user's initial VM gets `AuthN`, `CapDel`, `AssertM`, and `CapInv` permissions.
-   The custodian's service VM (using `custodian_service_pk_multibase`) is added, its controller being the custodian's own DID. This service VM only gets `CapabilityInvocation` permission.
-   The custodian's DID must exist and have a service of type "CadopCustodianService".

#### Multi-Key Type Support (Ed25519, Secp256k1, and Secp256r1)
The contract uniformly handles Ed25519, Secp256k1, and Secp256r1 keys through the `VERIFICATION_METHOD_TYPE_*` constants, `multibase_codec::decode` for standard multibase strings, and `multibase_key::decode_with_type` for `did:key` identifiers.

`internal_ensure_session_key` is the core helper function for session key registration:
```move
fun internal_ensure_session_key(
    did_document_data: &mut DIDDocument,
    vm_fragment: String,
    vm_public_key_multibase: String,
    vm_type: String,
)
```
It decodes the public key from `vm_public_key_multibase` using `multibase_codec::decode`, then generates an authentication key based on `vm_type`, and calls `session_key::create_session_key_internal` to register the session key for the DID's associated account.

- **For Ed25519 and Secp256k1**, it uses `session_key::ed25519_public_key_to_authentication_key` or `session_key::secp256k1_public_key_to_authentication_key`. Both of these functions derive the authentication key via `blake2b256(scheme_byte || public_key)`.
- **For Secp256r1**, it uses `session_key::secp256r1_public_key_to_authentication_key`. Note the different derivation logic: `scheme_byte || sha2_256(public_key)`.

The scope of the session key (`scopes_for_sk`) is set to allow operations on the DID's own account and modules under `rooch_framework`.

#### Bitcoin Address System Integration
Rooch's address system is based on Bitcoin addresses: Bitcoin private key → Bitcoin public key → Bitcoin address → Rooch address.
-   **Transaction-Level Validation**: The `AuthValidator` module verifies Bitcoin signatures during transaction validation and stores the validated Bitcoin address in the transaction context.
-   **Application-Level Validation**: The `did.move` module retrieves this Bitcoin address from the transaction context (`auth_validator::get_bitcoin_address_from_ctx_option()`) and performs a secondary verification against the user-provided public key (`bitcoin_address::verify_bitcoin_address_with_public_key`).
-   Supports various Bitcoin address formats like P2PKH, P2WPKH, P2TR, P2SH-P2WPKH.

#### NIP-3 (CADOP) Compatibility
Supports custodian-assisted DID creation via the `create_did_object_via_cadop_with_did_key_entry` function, ensuring users retain control through `did:key`, while custodians only gain service invocation permission.

### 2.5 Permissions and Security

#### Permission Validation System Design
Based on W3C DID specifications and NIP-1, a Session Key-based permission validation system is adopted.
1.  **DID Account Autonomy**: DID operations must be initiated by the DID's associated account via `did_signer: &signer`.
2.  **Session Key Authorization**: `did_signer` is actually authorized by a registered session key. `auth_validator::get_session_key_from_ctx_option()` is used to get the current transaction's session key (i.e., `authentication_key`).
3.  **Verification Relationship Control**: Permissions are based on a verification method's membership in specific verification relationships (`capabilityDelegation`, `capabilityInvocation`).

#### Permission Validation Flow
-   **`assert_authorized_for_capability_delegation(did_document_data: &DIDDocument, did_signer: &signer)`**:
    1.  Verify that the `did_signer`'s address is the associated account address of `did_document_data`.
    2.  Get the current transaction's session key.
    3.  Find the verification method fragment corresponding to this session key via `find_verification_method_by_session_key`.
    4.  Check if this fragment exists in the `did_document_data.capability_delegation` list.
-   **`assert_authorized_for_capability_invocation(...)`**: Similar flow, but checks the `capability_invocation` list.

These assertion functions are used to protect entry functions like verification method management and service management.
-   **Key and Verification Relationship Management** (e.g., `add_verification_method_entry`, `remove_verification_method_entry`, `add_to_verification_relationship_entry`, `remove_from_verification_relationship_entry`) requires `capabilityDelegation` permission.
-   **Service Management** (e.g., `add_service_entry`, `update_service_entry`, `remove_service_entry`) requires `capabilityInvocation` permission.

#### Mapping Authentication Key to Verification Method (`find_verification_method_by_session_key`)
This function iterates through all verification method fragments in `did_document_data.authentication`:
1.  Get the corresponding `VerificationMethod` object.
2.  Decode `public_key_multibase` using `multibase_codec::decode`.
3.  Convert the decoded public key to an `authentication_key` based on the VM's type (Ed25519, Secp256k1, or Secp256r1).
4.  If this `authentication_key` matches the current transaction's session key, return the fragment.

#### Key Security Points
1.  **Controller Permission Validation**: Ensures modifications to the DID document are authorized by a legitimate Controller via the correct verification method.
2.  **Session Key Management**: Ed25519, Secp256k1, and Secp256r1 `authentication` methods automatically register as session keys. Their permission scope needs to be reasonably set. The current scope is broad, allowing operations on the associated DID account and `rooch_framework`.
3.  **`AccountCap` Protection**: Although `DIDDocument` holds the `AccountCap`, its use is subject to internal logic and permission controls, preventing arbitrary use.
4.  **Address Validation Security**: A dual verification mechanism based on Bitcoin addresses prevents public key forgery.
5.  **Input Security**:
    *   Strictly verify the consistency between the public key of a `did:key` controller and the initial VM public key.
    *   Validate the format of various input strings (e.g., multibase public keys, DID strings).
6.  **State Consistency**:
    *   Ensure the `controller_to_dids` mapping in `DIDRegistry` is synchronized with the actual DID object states.
    *   Ensure session key registration and removal are synchronized with the state of verification methods in the `authentication` relationship.

## 3. Rooch DID Command-Line Tool (`rooch did`) Design

The `rooch did` command-line tool provides an interface to interact with the Rooch DID system.

### 3.1 Overview and Architecture

The CLI tool uses `clap` to parse arguments and calls corresponding Rust functions to communicate with the Rooch node, executing Move contract entry functions.

#### Command-Line Structure
Based on `crates/rooch/src/commands/did/mod.rs`, the main subcommands include:
```
rooch did
├── create            # DID creation related commands
├── manage            # DID management (verification methods, services, etc.)
├── query             # DID query functions
└── keygen            # Key generation for DID operations
```

### 3.2 Detailed Command Design

#### `rooch did create`
-   **`rooch did create self`**: User self-creates a DID.
    -   Calls `did::create_did_object_for_self_entry`.
    -   Parameters:
        -   `--account-public-key <PUBLIC_KEY>`: (Required) User's Secp256k1 public key (multibase format).
        -   Sender account specified implicitly or explicitly via wallet or `--sender`.
    -   Example: `rooch did create self --account-public-key z<Secp256k1_PK_Multibase>`
-   **`rooch did create cadop`**: Custodian-assisted DID creation (CADOP).
    -   Calls `did::create_did_object_via_cadop_with_did_key_entry`.
    -   Parameters:
        -   `--user-did-key <USER_DID_KEY_STRING>`: (Required) User's did:key string.
        -   `--custodian-service-key <CUSTODIAN_SERVICE_PK_MULTIBASE>`: (Required) Custodian's service public key for this user (multibase).
        -   `--custodian-key-type <TYPE_STRING>`: (Required) Custodian service key type (e.g., "Ed25519VerificationKey2020").
        -   Custodian account (sender) specified via wallet or `--sender`.
    -   Example: `rooch did create cadop --user-did-key "did:key:zExampleUser..." --custodian-service-key "zExampleServiceKey..." --custodian-key-type Ed25519VerificationKey2020 --sender <custodian_address>`

#### `rooch did manage`
Used for managing verification methods and services of a DID document. All management operations require identifying the DID to operate on, and authorization via the current wallet or session key.

-   **`rooch did manage add-vm`**: Add a verification method.
    -   Calls `did::add_verification_method_entry`.
    -   Parameters: `--fragment <FRAG>`, `--method-type <TYPE>`, `--public-key <PK_MULTIBASE>`, `--relationships <REL_LIST>` (comma-separated, e.g., `auth,assert,invoke,delegate,agree`).
    -   Example: `rooch did manage add-vm --fragment key-2 --method-type Ed25519VerificationKey2020 --public-key z... --relationships auth,assert`
-   **`rooch did manage remove-vm`**: Remove a verification method.
    -   Calls `did::remove_verification_method_entry`.
    -   Parameters: `--fragment <FRAG>`.
-   **`rooch did manage add-relationship`**: Add an existing verification method to a specified relationship.
    -   Calls `did::add_to_verification_relationship_entry`.
    -   Parameters: `--fragment <FRAG>`, `--relationship <REL_TYPE>` (single relationship type).
-   **`rooch did manage remove-relationship`**: Remove a verification method from a specified relationship.
    -   Calls `did::remove_from_verification_relationship_entry`.
    -   Parameters: `--fragment <FRAG>`, `--relationship <REL_TYPE>`.
-   **`rooch did manage add-service`**: Add a service.
    -   Calls `did::add_service_entry` or `did::add_service_with_properties_entry`.
    -   Parameters: `--fragment <FRAG>`, `--service-type <TYPE>`, `--endpoint <URL>`, `[--properties key1=val1,key2=val2]`.
-   **`rooch did manage update-service`**: Update a service.
    -   Calls `did::update_service_entry`.
    -   Parameters: `--fragment <FRAG>`, `--new-service-type <TYPE>`, `--new-endpoint <URL>`, `[--new-properties key1=val1]`.
-   **`rooch did manage remove-service`**: Remove a service.
    -   Calls `did::remove_service_entry`.
    -   Parameters: `--fragment <FRAG>`.

#### `rooch did query`
-   **`rooch did query address <ROOCH_ADDRESS>`**: Query the DID document associated with a Rooch address.
    -   Reads the object returned by `did::get_did_document_by_address(address)`.
-   **`rooch did query did <DID_STRING>`**: Query the DID document corresponding to a DID string.
    -   Reads the object returned by `did::get_did_document(did_str)`.
-   **`rooch did query object-id <OBJECT_ID>`**: Query the DID document corresponding to an ObjectID.
    -   Reads the object returned by `did::get_did_document_by_object_id(object_id)`.
-   **`rooch did query exists <IDENTIFIER>`**: Check if a DID exists (can be a Rooch address or DID string).
-   **`rooch did query controller <CONTROLLER_DID_STRING>`**: Query all DID strings controlled by a specific DID.
    -   Calls `did::get_dids_by_controller_string(controller_did_str)`.

#### `rooch did keygen`
-   **`rooch did keygen ed25519`**: Generate an Ed25519 key pair.
    -   Outputs private key, public key (multibase `z...` format), and corresponding `did:key` string.
-   **`rooch did keygen secp256k1`**: Generate a Secp256k1 key pair.
    -   Outputs private key, public key (multibase `z...` format), and corresponding `did:key` string.
-   **`rooch did keygen secp256r1`**: Generate a Secp256r1 (P-256) key pair.
    -   Outputs private key, public key (multibase `z...` format), and corresponding `did:key` string.

### 3.3 Usage Examples (Partial)
(Refer to `did.feature` and the command designs above)

```bash
# Create a personal DID (assuming the current wallet account public key is known or passed as a parameter)
rooch did create self --account-public-key z<Secp256k1_PK_Multibase_For_Current_Account>

# Query the newly created DID (assuming address is rooch1example...)
rooch did query address rooch1example...

# Add an Ed25519 verification method for authentication and assertion
rooch did manage add-vm --fragment key-ed25519 --method-type Ed25519VerificationKey2020 --public-key z<Ed25519_PK_Multibase> --relationships auth,assert

# Generate Ed25519 keys for CADOP
rooch did keygen ed25519
# (Assume output user_did_key: did:key:zUserKey...)

# Use another account as custodian for CADOP creation
rooch did create cadop --user-did-key did:key:zUserKey... --custodian-service-key zServiceKey... --custodian-key-type Ed25519VerificationKey2020 --sender <custodian_rooch_address>
```

### 3.4 Output Format
All commands typically output results in JSON format for programmatic processing and user readability.

## 4. Testing and Mocking Guide

In testing the Rooch DID system, especially for unit and integration tests of `did.move`, it's necessary to mock the retrieval behavior of `session key` and `Bitcoin address`. In a real environment, this data comes from the transaction validation process; in a test environment, mock methods must be provided.

### 4.1 Mock Method Details in `auth_validator.move`

The `auth_validator` module provides several `#[test_only]` functions to set a mock `TxValidateResult` into the transaction context. This result contains the `session_key` and `bitcoin_address` needed for DID function tests.

```move
#[test_only]
public fun set_tx_validate_result_for_testing(
    auth_validator_id: u64,
    auth_validator: Option<AuthValidator>,
    session_key: Option<vector<u8>>,
    bitcoin_address: BitcoinAddress,
)

#[test_only]
/// Create a simple TxValidateResult for basic testing (with a random Bitcoin address)
public fun set_simple_tx_validate_result_for_testing(session_key: Option<vector<u8>>)

#[test_only]
/// Create a TxValidateResult with a random Bitcoin address for testing
public fun set_random_tx_validate_result_for_testing(session_key: Option<vector<u8>>)
```
**Purpose**: These functions allow test code to easily construct a `TxValidateResult` (containing the required `session_key` and `bitcoin_address`) and set it into the transaction context for functions in `did.move` to read.

### 4.2 Usage Examples

#### Basic Mock Setup (Simulating Session Key)
```move
#[test]
fun test_mock_session_key_setup() {
    use rooch_framework::auth_validator;
    use rooch_framework::session_key;
    use moveos_std::multibase_codec;
    use std::option;
    use rooch_framework::genesis;

    genesis::init_for_test(); // Initialize framework for testing

    // Generate a test Ed25519 key and its authentication_key
    let test_ed25519_multibase_key = string::utf8(b"z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK"); // Example key
    let pk_bytes = option::destroy_some(multibase_codec::decode(&test_ed25519_multibase_key));
    let auth_key = session_key::ed25519_public_key_to_authentication_key(&pk_bytes);

    // Set this authentication_key as the session_key in the context
    auth_validator::set_simple_tx_validate_result_for_testing(option::some(auth_key));

    // Now, calls to auth_validator::get_session_key_from_ctx_option() in did.move will retrieve this auth_key
    let retrieved_key_opt = auth_validator::get_session_key_from_ctx_option();
    assert!(option::is_some(&retrieved_key_opt), 1001);
    assert!(option::destroy_some(retrieved_key_opt) == auth_key, 1002);
}
```

#### DID Permission Test Mock
Before testing DID operations requiring specific permissions (e.g., `add_verification_method_entry`), you need to:
1.  Create a test DID object.
2.  Ensure this DID object has a verification method in the `authentication` relationship, and this method has `capabilityDelegation` permission.
3.  Convert this verification method's public key to an `authentication_key`.
4.  Use `auth_validator::set_simple_tx_validate_result_for_testing` to set this `authentication_key` as the current transaction's `session_key`.
5.  Create a `signer` representing this DID's associated account.
6.  Call the DID entry function under test.

#### Bitcoin Address Validation Test
```move
#[test]
fun test_mock_bitcoin_address_setup() {
    use rooch_framework::auth_validator;
    use rooch_framework::bitcoin_address::{Self, BitcoinAddress};
    use rooch_framework::genesis;
    use std::option;

    genesis::init_for_test();

    let btc_addr = bitcoin_address::random_address_for_testing();
    let btc_addr_clone = btc_addr;

    auth_validator::set_tx_validate_result_for_testing(
        0,
        option::none(),
        option::none(), // No session key needed
        btc_addr
    );

    let retrieved_btc_addr_opt = auth_validator::get_bitcoin_address_from_ctx_option();
    assert!(option::is_some(&retrieved_btc_addr_opt), 2001);
    assert!(option::destroy_some(retrieved_btc_addr_opt) == btc_addr_clone, 2002);
}
```
This mock is crucial for testing functions like `did::verify_public_key_matches_account` that depend on fetching the Bitcoin address from the context.

#### Support for Different Key Types (Ed25519 / Secp256k1 / Secp256r1 for Session Key)
The mock setup process is similar for all supported key types:
1.  Get the public key (as a multibase string).
2.  Decode using `multibase_codec::decode`.
3.  Generate `authentication_key` using the corresponding function from the `session_key` module (`ed25519_...`, `secp256k1_...`, or `secp256r1_...`).
4.  Set this `authentication_key` into `TxValidateResult` using the mock functions.

### 4.3 Best Practices
-   **Independent Initialization**: Each test function should start with `genesis::init_for_test()`.
-   **Explicit Setup**: Clearly set the required `session_key` and `bitcoin_address` for each test case.
-   **Test Error Cases**: Test `#[expected_failure]` scenarios (like `ErrorNoSessionKeyInContext`, `ErrorSessionKeyNotFound`, `ErrorInsufficientPermission`) by not setting `session_key` or setting a mismatched `session_key`.

## 5. Security Considerations

### 5.1 Key Security
1.  **Private Key Protection**: Private keys generated by the CLI should be securely stored by the user. It is not recommended to pass private keys directly in command-line arguments in a production environment. Prefer using the Rooch keystore or external signers.
2.  **Session Keys**:
    *   The `did.move` contract automatically registers session keys for Ed25519, Secp256k1, and Secp256r1 keys in the `authentication` relationship. Their current scope is set to the DID's own account and `rooch_framework`; this scope should be periodically reviewed for appropriateness.
3.  **Permission Minimization**: Assign only the necessary verification relationships to verification methods for their tasks. Avoid over-authorization.

### 5.2 Access Control
1.  **On-Chain Validation**: All core permission validation logic for DID operations is executed in the `did.move` contract, ensuring on-chain enforcement.
2.  **Controller Importance**: The `controller` field of `DIDDocument` is crucial. Only DIDs listed as controllers can manage the DID document using keys with `capabilityDelegation` permission.
3.  **Event Auditing**: All significant changes to a DID document (e.g., creation, VM addition/removal, service addition/removal, relationship changes) emit events. Monitoring these events can be used for auditing and anomaly detection.

### 5.3 Standard Compliance and Interoperability
1.  **Adherence to W3C Specifications**: Strict adherence to W3C DID Core and DID Key Method specifications helps ensure compatibility with other DID ecosystems.
2.  **Input Validation**: Perform strict format and validity checks on all external inputs (e.g., public key multibase strings, DID strings, fragment formats) to prevent injection or parsing errors.

### 5.4 Gas and State Management
1.  **Gas Consumption**: Be mindful of the gas consumption of complex operations (e.g., functions involving multiple Table operations or extensive vector manipulations) to avoid potential out-of-gas issues.
2.  **State Bloat**: While DID documents can contain multiple verification methods and services, be aware of state bloat. For large amounts of data, consider off-chain storage with on-chain hashes/anchors.

## 6. Troubleshooting

### 6.1 Common Errors and Solutions

1.  **`ErrorSignerNotDIDAccount`**:
    *   **Cause**: The transaction signer's address does not match the Rooch account address associated with the target DID document. DID operations must be initiated by their own account.
    *   **Solution**: Ensure the transaction is signed using a key corresponding to the DID's associated account (usually authorized via a session key).

2.  **`ErrorNoSessionKeyInContext`**:
    *   **Cause**: A session key (`authentication_key`) could not be found in the current transaction context. All authorized DID operations rely on session keys.
    *   **Solution**: Ensure the transaction is sent via a valid session key registered to the DID's account.

3.  **`ErrorSessionKeyNotFound`**:
    *   **Cause**: The session key from the transaction context does not correspond to any public key of a verification method in the DID document's `authentication` relationship.
    *   **Solution**: Ensure the session key used corresponds to a verification method that is part of the `authentication` relationship.

4.  **`ErrorInsufficientPermission`**:
    *   **Cause**: The session key (and its associated verification method) used for authorization does not have the required permission for the operation (e.g., `capabilityDelegation` or `capabilityInvocation`).
    *   **Solution**: Check the DID document to ensure the verification method fragment used for signing is present in the correct verification relationship list.

5.  **`ErrorInvalidPublicKeyMultibaseFormat`**:
    *   **Cause**: The provided public key multibase string is invalid or cannot be decoded by `multibase_codec::decode`.
    *   **Solution**: Verify that the public key string conforms to a valid multibase encoding (e.g., base58btc starting with `z`).

### 6.2 Debugging Tips

1.  **View DID Document**: Use `rooch did query address <ADDRESS>` or `rooch did query did <DID_STRING>` to inspect the current state of the DID document.
2.  **Verify Permissions**: Manually check the `capability_delegation` and `capability_invocation` lists in the DID document.
3.  **Test Keys**: Use `rooch did keygen` to generate test keys and use their `did:key` or multibase public keys for operations.
4.  **View Transaction Events**: Checking events for a specific transaction hash can reveal events emitted by the `did.move` contract.

## 7. References

- [W3C DID Core Specification v1.0](https://www.w3.org/TR/did-core/)
- [W3C DID Key Method v1.0](https://w3c-ccg.github.io/did-method-key/)
- [Multibase Data Format Specification](https://datatracker.ietf.org/doc/html/draft-multiformats-multibase-03)
- [Multicodec Table](https://github.com/multiformats/multicodec/blob/master/table.csv)
- [NIP-1: Rooch DID Method Specification (Draft)](https://raw.githubusercontent.com/nuwa-protocol/NIPs/refs/heads/main/nips/nip-1.md) (as an important reference for Rooch DID design)
- [Rooch Network Documentation](https://rooch.network/docs)

## 8. Development Guide

This section provides guidance for developers wishing to extend the Rooch DID system.

### 8.1 Adding a New Verification Method Type

1.  **Move Contract (`did.move`)**:
    *   Add the new verification method type string in the constant definitions:
      ```move
      // const VERIFICATION_METHOD_TYPE_NEW_KEY_STANDARD: vector<u8> = b"NewKeyStandard2024";
      ```
    *   Update `add_authentication_method` and `internal_ensure_session_key` if the new type needs to be automatically registered as a session key. This will likely involve:
        *   Adding a new branch to handle the type.
        *   Calling a new function in the `session_key` module to convert its public key to an `authentication_key`.
    *   Update `find_verification_method_by_session_key`: If the new type is used for session keys, ensure it can be correctly matched.

2.  **Multibase Modules** (if new `did:key` support is needed):
    *   If the new key type needs a new multicodec prefix for `did:key`, it must be added to `multibase_key.move`.
    *   `multibase_codec.move` is generic and likely does not need changes unless a new base encoding (like base32) is required.

3.  **Session Key Module (`session_key.move`)** (if new authentication key derivation logic is needed):
    *   Add a new `new_key_type_public_key_to_authentication_key(&vector<u8>): vector<u8>` function.

4.  **Rust Type Definitions (`rooch-types`)**:
    *   Add support for the new type in Rust representations, ensuring consistency with the Move side.

5.  **CLI Tool (`rooch` commands)**:
    *   Update commands like `rooch did manage add-vm` to accept and process the new method type.
    *   Update `rooch did keygen` (if applicable) to generate keys of the new type.

6.  **Testing**:
    *   Add unit and integration tests covering the creation, usage, and permission validation of the new verification method type.

### 8.2 Extending the Permission Model

1.  **Move Contract (`did.move`)**:
    *   If a new verification relationship is needed, define it in the constants section:
      ```move
      // const VERIFICATION_RELATIONSHIP_NEW_PERMISSION: u8 = 5;
      ```
    *   Add a new `vector<String>` field to the `DIDDocument` struct to store the verification method fragments for this relationship.
    *   Update entry functions (`add_verification_method_entry`, `add_to_verification_relationship_entry`, etc.) to handle adding/removing from the new relationship.
    *   Create new permission assertion functions, e.g., `assert_authorized_for_new_operation(&DIDDocument, &signer)`, to implement validation logic for the new permission.

2.  **Rust Type Definitions (`rooch-types`)**:
    *   Update `VerificationRelationshipType` (or similar enum) to include the new relationship type.

3.  **CLI Tool (`rooch` commands)**:
    *   Update commands like `rooch did manage add-vm` and `add-relationship` to accept the new relationship type as input.

4.  **Testing**:
    *   Thoroughly test operations involving the new permission, including success and failure scenarios.

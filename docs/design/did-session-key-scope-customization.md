# DID Session Key Scope Customization Design

## Background

Currently, the DID module uses hardcoded wildcard scope (`*::*::*`) when creating session keys, which presents the following issues:

1. **Security Risk**: Wildcard permissions are too broad, violating the principle of least privilege
2. **Lack of Flexibility**: Different application scenarios require different permission scopes
3. **No Customization**: Users cannot customize session key access scope based on specific needs

## Solution Overview

Based on the existing scope string parsing capability in the `session_key.move` module, we add custom scope support to DID creation functions. Users can pass scope configurations through string arrays in the format `"address::module::function"`.

## Detailed Design

### 1. Modify DID Creation Function Signatures

#### 1.1 Internal Function Modification

```move
// Modify create_did_object_internal function
fun create_did_object_internal(
    creator_account_signer: &signer,
    doc_controller: DID,
    user_vm_pk_multibase: String,
    user_vm_type: String,
    user_vm_fragment: String,
    user_vm_relationships: vector<u8>,
    // New: custom session key scope string array
    custom_session_scope_strings: Option<vector<String>>,
    service_provider_controller_did: Option<DID>,
    service_vm_pk_multibase: Option<String>,
    service_vm_type: Option<String>,
    service_vm_fragment: Option<String>
): ObjectID
```

#### 1.2 Session Key Creation Function Modification

```move
// Modify internal_ensure_session_key function
fun internal_ensure_session_key(
    did_document_data: &mut DIDDocument,
    vm_fragment: String,
    vm_public_key_multibase: String,
    vm_type: String,
    // New: custom scope string array
    custom_scope_strings: Option<vector<String>>
) {
    // ... existing code ...
    
    // Parse scope strings or use default values
    let scopes_for_sk = if (option::is_some(&custom_scope_strings)) {
        let scope_strings = option::destroy_some(custom_scope_strings);
        parse_scope_strings_to_session_scopes(scope_strings)
    } else {
        // Maintain backward compatibility: use current default scope
        create_default_did_scopes(associated_address)
    };
    
    // ... rest of the code remains unchanged ...
}
```

### 2. New Helper Functions

#### 2.1 Scope String Parsing Function

```move
// Parse string array to SessionScope array
fun parse_scope_strings_to_session_scopes(scope_strings: vector<String>): vector<SessionScope> {
    let scopes = vector::empty<SessionScope>();
    let i = 0;
    
    while (i < vector::length(&scope_strings)) {
        let scope_str = *vector::borrow(&scope_strings, i);
        let scope = session_key::parse_scope_string(scope_str);
        vector::push_back(&mut scopes, scope);
        i = i + 1;
    };
    
    scopes
}
```

#### 2.2 Default Scope Creation Functions

```move
// Create default DID scope (maintain backward compatibility)
fun create_default_did_scopes(did_address: address): vector<SessionScope> {
    vector[
        session_key::new_session_scope(
            @rooch_framework,
            string::utf8(b"*"),
            string::utf8(b"*")
        ),
        session_key::new_session_scope(
            did_address,
            string::utf8(b"*"),
            string::utf8(b"*")
        )
    ]
}

// Create minimal privilege DID scope
fun create_minimal_did_scopes(): vector<SessionScope> {
    vector[
        session_key::new_session_scope(
            @rooch_framework,
            string::utf8(b"did"),
            string::utf8(b"*")
        ),
        session_key::new_session_scope(
            @rooch_framework,
            string::utf8(b"payment_channel"),
            string::utf8(b"*")
        ),
    ]
}
```

### 3. New Entry Functions

#### 3.1 DID Creation Function with Custom Scope

```move
// Create DID with custom scope
public entry fun create_did_object_for_self_with_custom_scopes_entry(
    creator_account_signer: &signer,
    account_public_key_multibase: String,
    session_scope_strings: vector<String>  // Format: "address::module::function"
) {
    let custom_scopes = if (vector::is_empty(&session_scope_strings)) {
        option::none<vector<String>>()
    } else {
        option::some(session_scope_strings)
    };
    
    create_did_object_for_self_internal_with_scopes(
        creator_account_signer,
        account_public_key_multibase,
        custom_scopes
    );
}

// Create DID with predefined scope template
public entry fun create_did_object_for_self_with_scope_template_entry(
    creator_account_signer: &signer,
    account_public_key_multibase: String,
    scope_template: u8  // 0: minimal, 1: default (wildcard), 2: custom (empty, user provides own)
) {
    let scope_strings = if (scope_template == 0) {
        // Minimal privilege template
        vector[
            string::utf8(b"0x3::did::add_verification_method_entry"),
            string::utf8(b"0x3::did::remove_verification_method_entry"),
            string::utf8(b"0x3::did::add_service_entry"),
            string::utf8(b"0x3::did::remove_service_entry"),
            string::utf8(b"0x3::did::update_service_entry"),
            string::utf8(b"0x3::did::add_to_verification_relationship_entry"),
            string::utf8(b"0x3::did::remove_from_verification_relationship_entry")
        ]
    } else if (scope_template == 1) {
        // Development mode: use wildcards (current behavior)
        let did_addr_str = address::to_bech32_string(signer::address_of(creator_account_signer));
        vector[
            string::utf8(b"0x3::*::*"),
            string::append_utf8(&mut did_addr_str, b"::*::*")
        ]
    } else {
        // template == 2: empty array, use default behavior
        vector::empty<String>()
    };
    
    let custom_scopes = if (vector::is_empty(&scope_strings)) {
        option::none<vector<String>>()
    } else {
        option::some(scope_strings)
    };
    
    create_did_object_for_self_internal_with_scopes(
        creator_account_signer,
        account_public_key_multibase,
        custom_scopes
    );
}
```

#### 3.2 Backward Compatible Original Function

```move
// Keep original function unchanged, use default scope
public entry fun create_did_object_for_self_entry(
    creator_account_signer: &signer,
    account_public_key_multibase: String,
) {
    create_did_object_for_self_internal_with_scopes(
        creator_account_signer,
        account_public_key_multibase,
        option::none<vector<String>>()  // Use default scope
    );
}
```

### 4. Also Applicable to CADOP Creation Functions

```move
// CADOP DID creation also supports custom scope
public entry fun create_did_object_via_cadop_with_did_key_and_scopes_entry(
    custodian_signer: &signer,
    user_did_key_string: String,
    custodian_service_pk_multibase: String,
    custodian_service_vm_type: String,
    session_scope_strings: vector<String>
) {
    // ... implement similar logic ...
}
```

## Usage Examples

### Example 1: Minimal Privilege DID

```move
// Create session key that can only perform basic DID operations
let minimal_scopes = vector[
    string::utf8(b"0x3::did::add_verification_method_entry"),
    string::utf8(b"0x3::did::remove_verification_method_entry"),
    string::utf8(b"0x3::did::add_service_entry")
];

create_did_object_for_self_with_custom_scopes_entry(
    &signer,
    public_key,
    minimal_scopes
);
```

### Example 2: Application-Specific Permissions

```move
// Create DID for DeFi application, allowing access to specific contracts
let defi_scopes = vector[
    string::utf8(b"0x3::did::*"),  // All DID operations
    string::utf8(b"0x123::defi_app::swap"),  // Specific DeFi function
    string::utf8(b"0x123::defi_app::add_liquidity")
];

create_did_object_for_self_with_custom_scopes_entry(
    &signer,
    public_key,
    defi_scopes
);
```

### Example 3: Using Predefined Templates

```move
// Use minimal privilege template
create_did_object_for_self_with_scope_template_entry(
    &signer,
    public_key,
    0  // Minimal privilege template
);

// Use development mode template (wildcards)
create_did_object_for_self_with_scope_template_entry(
    &signer,
    public_key,
    1  // Development mode template
);
```

## Implementation Plan

### Phase 1: Core Functionality Implementation
1. Modify `internal_ensure_session_key` function to support custom scope
2. Add scope string parsing helper functions
3. Update internal creation function signatures

### Phase 2: Entry Function Extension
1. Add custom scope entry functions
2. Add predefined template entry functions
3. Ensure backward compatibility

### Phase 3: Testing and Documentation
1. Write comprehensive unit tests
2. Update API documentation
3. Provide usage examples and best practice guides

## Security Considerations

1. **Input Validation**: Ensure scope string format is correct
2. **Privilege Minimization**: Recommend users prioritize minimal privilege templates
3. **Audit Support**: All scope configurations should be recorded in events
4. **Backward Compatibility**: Maintain compatibility of existing APIs, avoid breaking changes

## Architecture Benefits

This solution addresses the current limitations while maintaining system integrity:

### Current Architecture Understanding
- **Session Key Role**: Only verification methods added to the `authentication` relationship are automatically registered as Rooch session keys for transaction validation
- **Permission Verification Mechanism**:
  - `capabilityDelegation` for key management (add/remove verification methods, modify verification relationships)
  - `capabilityInvocation` for service management (add/remove/update services)
  - These permission checks are implemented through `assert_authorized_for_capability_*` functions
- **Current Scope Hardcoding Issue**: Using `*` wildcards in `internal_ensure_session_key`

### Solution Benefits
1. **Security**: Supports principle of least privilege
2. **Flexibility**: Supports full customization and predefined templates
3. **Compatibility**: Keeps existing APIs unchanged
4. **Usability**: Provides simple string format and predefined templates
5. **Application Diversity**: Different applications and users can create DIDs with appropriate permission scopes based on specific needs

## Summary

This solution introduces custom scope string arrays to resolve the overly broad permissions issue of DID session keys. The design considers:

- **Security**: Support for principle of least privilege
- **Flexibility**: Support for complete customization and predefined templates  
- **Compatibility**: Maintain existing API unchanged
- **Usability**: Provide simple string format and predefined templates

Through this solution, different applications and users can create DIDs with appropriate permission scopes based on specific requirements, improving both security and flexibility while maintaining the core DID functionality and permission model.

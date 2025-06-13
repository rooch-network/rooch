// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_framework::did {
    use std::vector;
    use std::string::{Self, String};
    use std::option::{Self, Option};
    use std::signer;
    use moveos_std::simple_map::{Self, SimpleMap};
    use moveos_std::table::{Self, Table};
    use moveos_std::account::{Self, AccountCap};
    use moveos_std::object::{Self, ObjectID};
    use moveos_std::address;
    use moveos_std::multibase_key;
    use moveos_std::multibase_codec;
    use moveos_std::event;
    use rooch_framework::session_key;
    use rooch_framework::auth_validator;
    use rooch_framework::bitcoin_address;

    friend rooch_framework::genesis;

    /// DID document does not exist (legacy or general not found)
    const ErrorDIDDocumentNotExist: u64 = 1;
    /// DID already exists (e.g., identifier already registered)
    const ErrorDIDAlreadyExists: u64 = 2;
    /// Unauthorized operation (generic, consider specific ErrorControllerPermissionDenied)
    const ErrorUnauthorized: u64 = 3;
    /// Verification method not found
    const ErrorVerificationMethodNotFound: u64 = 4;
    /// Verification method already exists
    const ErrorVerificationMethodAlreadyExists: u64 = 5;
    /// Service not found
    const ErrorServiceNotFound: u64 = 6;
    /// Service already exists
    const ErrorServiceAlreadyExists: u64 = 7;
    /// Verification method has expired
    const ErrorVerificationMethodExpired: u64 = 8;
    /// Invalid verification relationship
    const ErrorInvalidVerificationRelationship: u64 = 9;
    /// Verification method not in the relationship
    const ErrorVerificationMethodNotInRelationship: u64 = 10;
    /// Invalid signature (can be reused or made more specific)
    const ErrorInvalidSignature: u64 = 11;
    /// DIDRegistry is already initialized
    const ErrorDIDRegistryAlreadyInitialized: u64 = 12;
    /// DID Object not found for the given identifier
    const ErrorDIDObjectNotFound: u64 = 13;
    /// Associated AccountCap not found in DIDDocument when expected
    const ErrorAccountCapNotFound: u64 = 14;
    /// Permission denied based on controller check
    const ErrorControllerPermissionDenied: u64 = 15;
    /// Mismatch in length between property keys and values for a service
    const ErrorPropertyKeysValuesLengthMismatch: u64 = 16;
    /// Generic invalid argument
    const ErrorInvalidArgument: u64 = 17;
    /// No controllers specified during DID creation or update
    const ErrorNoControllersSpecified: u64 = 18;
    /// Verification method type is not supported for Rooch session key linkage (e.g., not Ed25519)
    const ErrorUnsupportedAuthKeyTypeForSessionKey: u64 = 19;
    /// The format of the publicKeyMultibase string is invalid or cannot be parsed
    const ErrorInvalidPublicKeyMultibaseFormat: u64 = 20;
    /// Failed to register key with the Rooch session key module
    const ErrorSessionKeyRegistrationFailed: u64 = 21;
    /// Invalid DID string format (should be "did:method:identifier")
    const ErrorInvalidDIDStringFormat: u64 = 22;
    /// For did:key controllers, the initial verification method public key must match the key in the DID identifier
    const ErrorDIDKeyControllerPublicKeyMismatch: u64 = 23;
    /// Multiple did:key controllers are not allowed during initial DID creation with a did:key controller
    const ErrorMultipleDIDKeyControllersNotAllowed: u64 = 24;
    /// Session key not found in DID document's authentication methods
    const ErrorSessionKeyNotFound: u64 = 25;
    /// Verification method has insufficient permission for the requested operation
    const ErrorInsufficientPermission: u64 = 26;
    /// The signer is not the DID's associated account
    const ErrorSignerNotDIDAccount: u64 = 27;
    /// No session key found in transaction context - all DID operations must use session keys
    const ErrorNoSessionKeyInContext: u64 = 28;
    /// Custodian does not have CADOP service
    const ErrorCustodianDoesNotHaveCADOPService: u64 = 29;
    /// Custodian DID document does not exist
    const ErrorCustodianDIDNotFound: u64 = 30;

    // Verification relationship types
    const VERIFICATION_RELATIONSHIP_AUTHENTICATION: u8 = 0;
    const VERIFICATION_RELATIONSHIP_ASSERTION_METHOD: u8 = 1;
    const VERIFICATION_RELATIONSHIP_CAPABILITY_INVOCATION: u8 = 2;
    const VERIFICATION_RELATIONSHIP_CAPABILITY_DELEGATION: u8 = 3;
    const VERIFICATION_RELATIONSHIP_KEY_AGREEMENT: u8 = 4;

     /// Get verification relationship constant for authentication
    public fun verification_relationship_authentication(): u8 {
        VERIFICATION_RELATIONSHIP_AUTHENTICATION
    }

    /// Get verification relationship constant for assertion method
    public fun verification_relationship_assertion_method(): u8 {
        VERIFICATION_RELATIONSHIP_ASSERTION_METHOD
    }

    /// Get verification relationship constant for capability invocation
    public fun verification_relationship_capability_invocation(): u8 {
        VERIFICATION_RELATIONSHIP_CAPABILITY_INVOCATION
    }

    /// Get verification relationship constant for capability delegation
    public fun verification_relationship_capability_delegation(): u8 {
        VERIFICATION_RELATIONSHIP_CAPABILITY_DELEGATION
    }

    /// Get verification relationship constant for key agreement
    public fun verification_relationship_key_agreement(): u8 {
        VERIFICATION_RELATIONSHIP_KEY_AGREEMENT
    }

    // Verification method types
    const VERIFICATION_METHOD_TYPE_ED25519: vector<u8> = b"Ed25519VerificationKey2020";
    const VERIFICATION_METHOD_TYPE_SECP256K1: vector<u8> = b"EcdsaSecp256k1VerificationKey2019";
    const VERIFICATION_METHOD_TYPE_SECP256R1: vector<u8> = b"EcdsaSecp256r1VerificationKey2019";


    /// DID identifier type
    struct DID has store, copy, drop {
        method: String,     // DID method (e.g., "rooch")
        identifier: String, // DID identifier (e.g., "0x123..." or specific string)
    }

    /// Verification method ID
    struct VerificationMethodID has store, copy, drop {
        did: DID,
        fragment: String,  // Fragment part of the ID (e.g., "key-1")
    }

    /// Service ID
    struct ServiceID has store, copy, drop {
        did: DID,
        fragment: String,  // Fragment part of the ID (e.g., "llm-gateway")
    }

    /// Verification method
    struct VerificationMethod has store, copy, drop {
        id: VerificationMethodID,
        type: String,      // Type of verification method (e.g., "Ed25519VerificationKey2020")
        controller: DID,   // Controller of this verification method
        public_key_multibase: String, // Public key in multibase format
    }

    /// Service definition
    struct Service has store, copy, drop {
        id: ServiceID,
        type: String,      // Type of service (e.g., "LLMGatewayNIP9", "CustodianServiceCADOP")
        service_endpoint: String, // URL or identifier for the service
        properties: SimpleMap<String, String>, // Additional service properties
    }

    /// DID Document containing all DID information. This is the data part of an Object.
    /// The DIDDocuemnt only has `key` ability, no `store`, so the user can not transfer it to other accounts.
    struct DIDDocument has key {
        id: DID,
        controller: vector<DID>,
        verification_methods: SimpleMap<String, VerificationMethod>,
        authentication: vector<String>,
        assertion_method: vector<String>,
        capability_invocation: vector<String>,
        capability_delegation: vector<String>,
        key_agreement: vector<String>,
        services: SimpleMap<String, Service>,
        also_known_as: vector<String>,
        account_cap: AccountCap,
        // Note: created_timestamp and updated_timestamp removed - use Object system timestamps instead
    }

    /// Registry to store mappings. This is a Named Object.
    struct DIDRegistry has key {
        /// Controller DID -> DID Document DID it controls
        controller_to_dids: Table<String, vector<String>>, 
    }

    // =================== Event Structures ===================

    #[event] 
    /// Event emitted when a new DID document is created
    struct DIDCreatedEvent has drop, copy, store {
        did: String,                           // The created DID
        object_id: ObjectID,                // Object ID of the DID document
        controller: vector<String>,            // Controllers of the DID
        creator_address: address,           // Address of the creator
    }

    #[event]
    /// Event emitted when a verification method is added to a DID document  
    struct VerificationMethodAddedEvent has drop, copy, store {
        did: String,                           // The DID that owns the verification method
        fragment: String,                   // Fragment identifier of the verification method
        method_type: String,                // Type of verification method
        controller: String,                    // Controller of the verification method
        verification_relationships: vector<u8>, // Verification relationships assigned
    }

    #[event]
    /// Event emitted when a verification method is removed from a DID document
    struct VerificationMethodRemovedEvent has drop, copy, store {
        did: String,                           // The DID that owned the verification method
        fragment: String,                   // Fragment identifier of the removed verification method
        method_type: String,                // Type of verification method that was removed
    }

    #[event]
    /// Event emitted when a verification relationship is modified
    struct VerificationRelationshipModifiedEvent has drop, copy, store {
        did: String,                           // The DID that owns the verification method
        fragment: String,                   // Fragment identifier of the verification method
        relationship_type: u8,              // Type of verification relationship
        operation: String,                  // Operation performed ("added" or "removed")
    }

    #[event]
    /// Event emitted when a service is added to a DID document
    struct ServiceAddedEvent has drop, copy, store {
        did: String,                           // The DID that owns the service
        fragment: String,                   // Fragment identifier of the service
        service_type: String,               // Type of service
        service_endpoint: String,           // Service endpoint URL
        properties_count: u64,              // Number of additional properties
    }

    #[event]
    /// Event emitted when a service is updated in a DID document
    struct ServiceUpdatedEvent has drop, copy, store {
        did: String,                           // The DID that owns the service
        fragment: String,                   // Fragment identifier of the service
        old_service_type: String,           // Previous service type
        new_service_type: String,           // New service type
        old_service_endpoint: String,       // Previous service endpoint
        new_service_endpoint: String,       // New service endpoint
        new_properties_count: u64,          // Number of new additional properties
    }

    #[event]
    /// Event emitted when a service is removed from a DID document
    struct ServiceRemovedEvent has drop, copy, store {
        did: String,                           // The DID that owned the service
        fragment: String,                   // Fragment identifier of the removed service
        service_type: String,               // Type of service that was removed
    }

    /// Returns the fixed ObjectID for the DIDRegistry.
    fun did_registry_id(): ObjectID {
        object::named_object_id<DIDRegistry>()
    }

    public(friend) fun genesis_init(){
        let registry_id = did_registry_id();
        assert!(!object::exists_object_with_type<DIDRegistry>(registry_id), ErrorDIDRegistryAlreadyInitialized);
        
        let registry_data = DIDRegistry {
            controller_to_dids: table::new<String, vector<String>>(),
        };

        let registry_object = object::new_named_object(registry_data);
        object::transfer_extend(registry_object, @rooch_framework);
    }

    /// Initialize the DID system
    /// Any account can call this function to initialize the DID system
    public entry fun init_did_registry() {
        genesis_init();
    }

    /// Borrows an immutable reference to the global DIDRegistry object's internal state.
    fun borrow_did_registry(): &DIDRegistry {
        let registry_object_ref = object::borrow_object<DIDRegistry>(did_registry_id());
        object::borrow(registry_object_ref)
    }

    /// Borrows a mutable reference to the global DIDRegistry object's internal state.
    fun borrow_mut_did_registry(): &mut DIDRegistry {
        let registry_mut_object_ref = object::borrow_mut_object_extend<DIDRegistry>(did_registry_id());
        object::borrow_mut(registry_mut_object_ref)
    }

    /// Resolves the ObjectID of a DIDDocument deterministically using its identifier string as a seed.
    fun resolve_did_object_id(did_identifier_str: &String): ObjectID {
        object::custom_object_id<String, DIDDocument>(*did_identifier_str)
    }

    /// Helper function to resolve DID ObjectID from signer address
    /// This eliminates code duplication across entry functions
    fun resolve_did_object_from_signer(did_signer: &signer): ObjectID {
        let signer_address = signer::address_of(did_signer);
        let did_identifier_str = address::to_bech32_string(signer_address);
        resolve_did_object_id(&did_identifier_str)
    }


    /// Internal function for creating DID objects. Not exposed as public entry.
    /// This function contains the core logic for DID creation and is called by
    /// the specialized public entry functions.
    fun create_did_object_internal(
        creator_account_signer: &signer,    // The Rooch account signer creating this DID Object (pays gas)
        doc_controller: DID,        // DID Document controller field value (NIP-1)
                                             // e.g.: User self-creation: ["did:rooch:<user_addr>"]
                                             // CADOP scenario: ["did:key:<user_pk_multibase>"]

        // User's initial verification method (VM) - usually the main key associated with the new did:rooch:<addr>
        user_vm_pk_multibase: String,       // User public key (multibase format)
        user_vm_type: String,               // VM type, e.g., "Ed25519VerificationKey2020"
        user_vm_fragment: String,           // VM fragment, e.g., "key-1"
        user_vm_relationships: vector<u8>,  // Verification relationships this VM should have
                                             // e.g.: User self-creation: [AuthN, AssertM, CapInv, CapDel]
                                             // CADOP scenario per NIP-3: [AuthN, CapDel]

        // Optional: Add service verification method for other service providers (e.g., custodians) at creation time
        // This service VM's controller is service_provider_controller_did
        // This service VM's fragment will be added to capabilityInvocation relationship
        service_provider_controller_did: Option<DID>, // Service provider (e.g., custodian) DID
        service_vm_pk_multibase: Option<String>,      // Service VM public key
        service_vm_type: Option<String>,              // Service VM type
        service_vm_fragment: Option<String>           // Service VM fragment
    ): ObjectID {
        let registry = borrow_mut_did_registry();


        let new_account_cap = account::create_account_and_return_cap();
        let did_address = account::account_cap_address(&new_account_cap);
        
        let did = new_rooch_did_by_address(did_address);
        let did_str = format_did(&did);
        
        let new_object_id = resolve_did_object_id(&did.identifier); 
        assert!(!object::exists_object_with_type<DIDDocument>(new_object_id), ErrorDIDAlreadyExists);
                
        // Create base DIDDocument structure
        let did_document_data = DIDDocument {
            id: did,
            controller: vector[doc_controller],
            verification_methods: simple_map::new<String, VerificationMethod>(),
            authentication: vector::empty<String>(),
            assertion_method: vector::empty<String>(),
            capability_invocation: vector::empty<String>(),
            capability_delegation: vector::empty<String>(),
            key_agreement: vector::empty<String>(),
            services: simple_map::new<String, Service>(),
            account_cap: new_account_cap,
            also_known_as: vector::empty<String>(),
        };

        // Process user's initial verification method
        let user_vm_id = VerificationMethodID {
            did: did,
            fragment: user_vm_fragment,
        };
        //The first verification method is the primary verification method, the controller is the doc_controller
        let user_vm = VerificationMethod {
            id: user_vm_id,
            type: user_vm_type,
            controller: doc_controller, 
            public_key_multibase: user_vm_pk_multibase,
        };

        simple_map::add(&mut did_document_data.verification_methods, user_vm_fragment, user_vm);

        // Add user VM to specified verification relationships
        let i = 0;
        while (i < vector::length(&user_vm_relationships)) {
            let relationship_type = *vector::borrow(&user_vm_relationships, i);
            
            if (relationship_type == VERIFICATION_RELATIONSHIP_AUTHENTICATION) {
                // Use the general authentication method that handles session key registration
                add_authentication_method(
                    &mut did_document_data,
                    user_vm_fragment,
                    user_vm_type,
                    user_vm_pk_multibase
                );
            } else if (relationship_type == VERIFICATION_RELATIONSHIP_ASSERTION_METHOD) {
                vector::push_back(&mut did_document_data.assertion_method, user_vm_fragment);
            } else if (relationship_type == VERIFICATION_RELATIONSHIP_CAPABILITY_INVOCATION) {
                vector::push_back(&mut did_document_data.capability_invocation, user_vm_fragment);
            } else if (relationship_type == VERIFICATION_RELATIONSHIP_CAPABILITY_DELEGATION) {
                vector::push_back(&mut did_document_data.capability_delegation, user_vm_fragment);
            } else if (relationship_type == VERIFICATION_RELATIONSHIP_KEY_AGREEMENT) {
                vector::push_back(&mut did_document_data.key_agreement, user_vm_fragment);
            };
            // Note: Invalid relationship types are silently ignored for flexibility
            
            i = i + 1;
        };

        // Process optional service provider verification method
        if (option::is_some(&service_provider_controller_did)) {
            let service_provider_did = option::extract(&mut service_provider_controller_did);
            let service_vm_pk = option::extract(&mut service_vm_pk_multibase);
            let service_vm_type_val = option::extract(&mut service_vm_type);
            let service_vm_frag = option::extract(&mut service_vm_fragment);

            let service_vm_id = VerificationMethodID {
                did: did,
                fragment: service_vm_frag,
            };
            let service_vm = VerificationMethod {
                id: service_vm_id,
                type: service_vm_type_val,
                controller: service_provider_did, // Service VM controlled by the service provider
                public_key_multibase: service_vm_pk,
            };

            simple_map::add(&mut did_document_data.verification_methods, service_vm_frag, service_vm);
            vector::push_back(&mut did_document_data.capability_invocation, service_vm_frag);
        };

        let did_object = object::new_with_id(did.identifier, did_document_data);
        object::transfer_extend(did_object, did_address);
        
        let doc_controller_did_str = format_did(&doc_controller);

        // Add the new DID to all its controllers' lists in the registry

        if (!table::contains(&registry.controller_to_dids, doc_controller_did_str)) {
            table::add(&mut registry.controller_to_dids, doc_controller_did_str, vector::empty<String>());
        };
        let controller_dids = table::borrow_mut(&mut registry.controller_to_dids, doc_controller_did_str);
        vector::push_back(controller_dids, did_str);
        
        
        let creator_address = signer::address_of(creator_account_signer);
        event::emit(DIDCreatedEvent {
            did: did_str,
            object_id: new_object_id,
            controller: vector[doc_controller_did_str],
            creator_address,
        });
        
        new_object_id
    }

    /// Create a DID for oneself using account key only.
    /// This function validates that the provided public key corresponds to the creator's account.
    /// Currently only supports Secp256k1 keys.
    public entry fun create_did_object_for_self_entry(
        creator_account_signer: &signer,        // User's own Rooch account signer
        account_public_key_multibase: String,   // User's account public key (Secp256k1)
    ) {
        create_did_object_for_self(
            creator_account_signer,
            account_public_key_multibase
        );
    } 

    /// Internal function for self DID creation.
    /// Validates that the provided public key matches the creator's account address.
    public fun create_did_object_for_self(
        creator_account_signer: &signer,
        account_public_key_multibase: String,
    ) : ObjectID {
        let creator_address = signer::address_of(creator_account_signer);
        let public_key_opt = multibase_codec::decode(&account_public_key_multibase);
        assert!(option::is_some(&public_key_opt), ErrorInvalidPublicKeyMultibaseFormat);
        let public_key = option::destroy_some(public_key_opt);

        // Validate that the provided public key corresponds to the creator's account
        verify_public_key_matches_account(creator_address, &public_key);
        
        let creator_did = new_rooch_did_by_address(creator_address);
        
        
        // Primary verification method uses the account's Secp256k1 key
        let primary_vm_fragment = string::utf8(b"account-key");
        let account_key_type = string::utf8(VERIFICATION_METHOD_TYPE_SECP256K1);
        let primary_vm_relationships = vector[
            VERIFICATION_RELATIONSHIP_AUTHENTICATION,
            VERIFICATION_RELATIONSHIP_ASSERTION_METHOD,
            VERIFICATION_RELATIONSHIP_CAPABILITY_INVOCATION,
            VERIFICATION_RELATIONSHIP_CAPABILITY_DELEGATION
        ];

        let did_object_id = create_did_object_internal(
            creator_account_signer,
            creator_did,
            account_public_key_multibase,
            account_key_type,
            primary_vm_fragment,
            primary_vm_relationships,
            option::none<DID>(),
            option::none<String>(),
            option::none<String>(),
            option::none<String>()
        );
        did_object_id
    }

    /// Verify that the provided public key corresponds to the given account address.
    /// This prevents users from providing incorrect public keys during DID creation.
    fun verify_public_key_matches_account(
        account_address: address,
        pk_bytes: &vector<u8>
    ) {
        // Get the Bitcoin address from current transaction context
        // This address was already validated during transaction authentication
        let bitcoin_address_opt = auth_validator::get_bitcoin_address_from_ctx_option();

        assert!(option::is_some(&bitcoin_address_opt), ErrorDIDKeyControllerPublicKeyMismatch);
        let bitcoin_address = option::destroy_some(bitcoin_address_opt);
        
        // Verify that the provided public key corresponds to the Bitcoin address
        assert!(
            bitcoin_address::verify_bitcoin_address_with_public_key(&bitcoin_address, pk_bytes),
            ErrorDIDKeyControllerPublicKeyMismatch
        );
        
        // Verify that the Bitcoin address corresponds to the account address
        let rooch_address_from_bitcoin = bitcoin_address::to_rooch_address(&bitcoin_address);
        assert!(
            rooch_address_from_bitcoin == account_address,
            ErrorDIDKeyControllerPublicKeyMismatch
        );
    }

    /// Create a DID via CADOP (Custodian-Assisted DID Onboarding Protocol) using did:key.
    /// The custodian assists in DID creation but the user retains control.
    /// Each user gets a unique service key from the custodian.
    /// The user's public key is extracted from their did:key string.
    public entry fun create_did_object_via_cadop_with_did_key_entry(
        custodian_signer: &signer,              // Custodian's Rooch account, pays gas
        user_did_key_string: String,            // User's did:key string (e.g., "did:key:zABC...")
        
        // Custodian generates a unique service key for this user
        custodian_service_pk_multibase: String, // Custodian's service public key for this user
        custodian_service_vm_type: String       // Custodian service VM type (Ed25519 or Secp256k1)
    ) {
        let _ = create_did_object_via_cadop_with_did_key(
            custodian_signer,
            user_did_key_string,
            custodian_service_pk_multibase,
            custodian_service_vm_type
        );
    }

    /// Internal function for CADOP DID creation with did:key.
    /// Returns the ObjectID of the created DID document for testing and verification.
    public fun create_did_object_via_cadop_with_did_key(
        custodian_signer: &signer,              // Custodian's Rooch account, pays gas
        user_did_key_string: String,            // User's did:key string (e.g., "did:key:zABC...")
        custodian_service_pk_multibase: String, // Custodian's service public key for this user
        custodian_service_vm_type: String       // Custodian service VM type (Ed25519 or Secp256k1)
    ): ObjectID {
        // Parse user's did:key
        let user_did_key = parse_did_string(&user_did_key_string);
        
        // Extract public key from did:key identifier
        // For did:key, the identifier contains multicodec prefix + raw public key
        assert!(user_did_key.method == string::utf8(b"key"), ErrorInvalidDIDStringFormat);
        
        // Extract raw public key from did:key identifier and re-encode as regular multibase
        let (key_type, raw_pk_bytes) = multibase_key::decode_with_type(&user_did_key.identifier);
        
        let (user_vm_type, user_vm_pk_multibase) = if (key_type == multibase_key::key_type_ed25519()) {
            (string::utf8(VERIFICATION_METHOD_TYPE_ED25519), multibase_codec::encode_base58btc(&raw_pk_bytes))
        } else if (key_type == multibase_key::key_type_secp256k1()) {
            (string::utf8(VERIFICATION_METHOD_TYPE_SECP256K1), multibase_codec::encode_base58btc(&raw_pk_bytes))
        } else if (key_type == multibase_key::key_type_ecdsar1()) {
            (string::utf8(VERIFICATION_METHOD_TYPE_SECP256R1), multibase_codec::encode_base58btc(&raw_pk_bytes))
        } else {
            abort ErrorUnsupportedAuthKeyTypeForSessionKey
        };
        
        // Derive custodian's DID from signer address
        let custodian_address = signer::address_of(custodian_signer);
        let custodian_did = new_rooch_did_by_address(custodian_address); 

        let custodian_did_str = address::to_bech32_string(custodian_address);
        let custodian_did_object_id = resolve_did_object_id(&custodian_did_str);
        
        // First check if custodian DID document exists
        assert!(
            object::exists_object_with_type<DIDDocument>(custodian_did_object_id), 
            ErrorCustodianDIDNotFound
        );
        
        // Then check if custodian has CADOP service
        let custodian_did_doc = get_did_document_by_object_id(custodian_did_object_id);
        let has_cadop_service = has_cadop_service_in_doc(custodian_did_doc);
        assert!(has_cadop_service, ErrorCustodianDoesNotHaveCADOPService);

        let doc_controller = user_did_key;
        let user_vm_relationships = vector[
            VERIFICATION_RELATIONSHIP_AUTHENTICATION,
            VERIFICATION_RELATIONSHIP_CAPABILITY_DELEGATION,
            VERIFICATION_RELATIONSHIP_ASSERTION_METHOD,
            VERIFICATION_RELATIONSHIP_CAPABILITY_INVOCATION,
        ]; 

        // Standardize user VM to Ed25519 (most common for did:key)
        let user_vm_fragment = string::utf8(b"account-key");

        // Generate unique service fragment for this user
        let custodian_service_vm_fragment = generate_service_fragment_for_user(&user_did_key_string);

        create_did_object_internal(
            custodian_signer,
            doc_controller,
            user_vm_pk_multibase,
            user_vm_type,
            user_vm_fragment,
            user_vm_relationships,
            option::some(custodian_did),
            option::some(custodian_service_pk_multibase),
            option::some(custodian_service_vm_type),
            option::some(custodian_service_vm_fragment)
        )
    } 

    /// Helper function to check if a DID document has a CADOP custodian service
    fun has_cadop_service_in_doc(did_doc: &DIDDocument): bool {
        let services = &did_doc.services;
        let service_keys = simple_map::keys(services);
        let i = 0;
        
        while (i < vector::length(&service_keys)) {
            let service_key = vector::borrow(&service_keys, i);
            let service = simple_map::borrow(services, service_key);
            if (service.type == string::utf8(b"CadopCustodianService")) {
                return true
            };
            i = i + 1;
        };
        
        false
    }

    /// Generate a unique service fragment for a user based on their DID
    fun generate_service_fragment_for_user(user_did_key_string: &String): String {
        let fragment = string::utf8(b"custodian-service-");
        
        // Extract a short identifier from the user's did:key
        // For did:key:zABC..., we take the first 8 characters after 'z'
        let did_bytes = string::bytes(user_did_key_string);
        let start_pos = 8; // Skip "did:key:z"
        let end_pos = if (vector::length(did_bytes) > start_pos + 8) {
            start_pos + 8
        } else {
            vector::length(did_bytes)
        };
        
        let i = start_pos;
        while (i < end_pos) {
            let byte_val = *vector::borrow(did_bytes, i);
            string::append_utf8(&mut fragment, vector[byte_val]);
            i = i + 1;
        };
        
        fragment
    }

    public entry fun add_verification_method_entry(
        did_signer: &signer,
        fragment: String,
        method_type: String,
        public_key_multibase: String,
        verification_relationships: vector<u8> 
    ) {
        // Use helper function to get authorized DID document
        let did_document_data = get_authorized_did_document_mut_for_delegation(did_signer);

        assert!(!simple_map::contains_key(&did_document_data.verification_methods, &fragment),
            ErrorVerificationMethodAlreadyExists);

        // Create and add the verification method
        let verification_method_id = VerificationMethodID {
            did: did_document_data.id,
            fragment: fragment,
        };

        let verification_method = VerificationMethod {
            id: verification_method_id,
            type: method_type,
            controller: did_document_data.id,
            public_key_multibase,
        };

        simple_map::add(&mut did_document_data.verification_methods, fragment, verification_method);

        // Add to specified verification relationships
        let i = 0;
        while (i < vector::length(&verification_relationships)) {
            let relationship_type = *vector::borrow(&verification_relationships, i);
            
            if (relationship_type == VERIFICATION_RELATIONSHIP_AUTHENTICATION) {
                // Use the general authentication method that handles all key types
                add_authentication_method(
                    did_document_data,
                    fragment,
                    method_type,
                    public_key_multibase
                );
            } else if (relationship_type == VERIFICATION_RELATIONSHIP_ASSERTION_METHOD) {
                if (!vector::contains(&did_document_data.assertion_method, &fragment)) {
                    vector::push_back(&mut did_document_data.assertion_method, fragment);
                };
            } else if (relationship_type == VERIFICATION_RELATIONSHIP_CAPABILITY_INVOCATION) {
                if (!vector::contains(&did_document_data.capability_invocation, &fragment)) {
                    vector::push_back(&mut did_document_data.capability_invocation, fragment);
                };
            } else if (relationship_type == VERIFICATION_RELATIONSHIP_CAPABILITY_DELEGATION) {
                if (!vector::contains(&did_document_data.capability_delegation, &fragment)) {
                    vector::push_back(&mut did_document_data.capability_delegation, fragment);
                };
            } else if (relationship_type == VERIFICATION_RELATIONSHIP_KEY_AGREEMENT) {
                if (!vector::contains(&did_document_data.key_agreement, &fragment)) {
                    vector::push_back(&mut did_document_data.key_agreement, fragment);
                };
            };
            // Note: Invalid relationship types are silently ignored for flexibility
            
            i = i + 1;
        };
        
        // Emit verification method added event
        event::emit(VerificationMethodAddedEvent {
            did: format_did(&did_document_data.id),
            fragment: fragment,
            method_type: method_type,
            controller: format_did(&did_document_data.id),
            verification_relationships: verification_relationships,
        });
    }

    public entry fun remove_verification_method_entry(
        did_signer: &signer,
        fragment: String
    ) {
        // Use helper function to get authorized DID document
        let did_document_data = get_authorized_did_document_mut_for_delegation(did_signer);

        assert!(simple_map::contains_key(&did_document_data.verification_methods, &fragment),
            ErrorVerificationMethodNotFound);

        // Store method type before removal for event
        let vm_to_remove = simple_map::borrow(&did_document_data.verification_methods, &fragment);
        let removed_method_type = vm_to_remove.type;

        if (vector::contains(&did_document_data.authentication, &fragment)) {
            internal_remove_session_key(did_signer, &vm_to_remove.public_key_multibase, &vm_to_remove.type);
        };

        remove_from_verification_relationship_internal(&mut did_document_data.authentication, &fragment);
        remove_from_verification_relationship_internal(&mut did_document_data.assertion_method, &fragment);
        remove_from_verification_relationship_internal(&mut did_document_data.capability_invocation, &fragment);
        remove_from_verification_relationship_internal(&mut did_document_data.capability_delegation, &fragment);
        remove_from_verification_relationship_internal(&mut did_document_data.key_agreement, &fragment);

        simple_map::remove(&mut did_document_data.verification_methods, &fragment);
        
        // Emit verification method removed event
        event::emit(VerificationMethodRemovedEvent {
            did: format_did(&did_document_data.id),
            fragment: fragment,
            method_type: removed_method_type,
        });
    }

    fun remove_from_verification_relationship_internal(
        relationship_vec: &mut vector<String>,
        fragment_to_remove: &String
    ) : bool {
        let removed = vector::remove_value(relationship_vec, fragment_to_remove);
        vector::length(&removed) > 0
    }

    public entry fun add_to_verification_relationship_entry(
        did_signer: &signer,
        fragment: String,
        relationship_type: u8
    ) {
        // Use helper function to get authorized DID document
        let did_document_data = get_authorized_did_document_mut_for_delegation(did_signer);

        assert!(simple_map::contains_key(&did_document_data.verification_methods, &fragment),
            ErrorVerificationMethodNotFound);

        if (relationship_type == VERIFICATION_RELATIONSHIP_AUTHENTICATION) {
            // Use the general authentication method that also registers it as a rooch session key
            let vm = *simple_map::borrow(&did_document_data.verification_methods, &fragment);
            add_authentication_method(
                did_document_data,
                fragment,
                vm.type,
                vm.public_key_multibase
            );
        } else {
            let target_relationship_vec_mut = if (relationship_type == VERIFICATION_RELATIONSHIP_ASSERTION_METHOD) {
                &mut did_document_data.assertion_method
            } else if (relationship_type == VERIFICATION_RELATIONSHIP_CAPABILITY_INVOCATION) {
                &mut did_document_data.capability_invocation
            } else if (relationship_type == VERIFICATION_RELATIONSHIP_CAPABILITY_DELEGATION) {
                &mut did_document_data.capability_delegation
            } else if (relationship_type == VERIFICATION_RELATIONSHIP_KEY_AGREEMENT) {
                &mut did_document_data.key_agreement
            } else {
                abort ErrorInvalidVerificationRelationship
            };

            // Add to the relationship if not already present
            if (!vector::contains(target_relationship_vec_mut, &fragment)) {
                vector::push_back(target_relationship_vec_mut, fragment);
            };
        };

        // Emit relationship modified event
        event::emit(VerificationRelationshipModifiedEvent {
            did: format_did(&did_document_data.id),
            fragment: fragment,
            relationship_type: relationship_type,
            operation: string::utf8(b"added"),
        });
    }

    public entry fun remove_from_verification_relationship_entry(
        did_signer: &signer,
        fragment: String,
        relationship_type: u8
    ) {
        // Derive DID identifier from signer address
        let signer_address = signer::address_of(did_signer);
        let did_identifier_str = address::to_bech32_string(signer_address);
        
        let object_id = resolve_did_object_id(&did_identifier_str);
        assert!(object::exists_object_with_type<DIDDocument>(object_id), ErrorDIDObjectNotFound);

        let did_doc_obj_ref_mut = object::borrow_mut_object_extend<DIDDocument>(object_id);
        let did_document_data = object::borrow_mut(did_doc_obj_ref_mut);

        assert_authorized_for_capability_delegation(did_document_data, did_signer);

        let target_relationship_vec_mut = if (relationship_type == VERIFICATION_RELATIONSHIP_AUTHENTICATION) {
            &mut did_document_data.authentication
        } else if (relationship_type == VERIFICATION_RELATIONSHIP_ASSERTION_METHOD) {
            &mut did_document_data.assertion_method
        } else if (relationship_type == VERIFICATION_RELATIONSHIP_CAPABILITY_INVOCATION) {
            &mut did_document_data.capability_invocation
        } else if (relationship_type == VERIFICATION_RELATIONSHIP_CAPABILITY_DELEGATION) {
            &mut did_document_data.capability_delegation
        } else if (relationship_type == VERIFICATION_RELATIONSHIP_KEY_AGREEMENT) {
            &mut did_document_data.key_agreement
        } else {
            abort ErrorInvalidVerificationRelationship
        };

        let original_len = vector::length(target_relationship_vec_mut);
        remove_from_verification_relationship_internal(target_relationship_vec_mut, &fragment);
        if (vector::length(target_relationship_vec_mut) < original_len) {
            // Emit verification relationship modified event only if removal was successful
            event::emit(VerificationRelationshipModifiedEvent {
                did: format_did(&did_document_data.id),
                fragment: fragment,
                relationship_type: relationship_type,
                operation: string::utf8(b"removed"),
            });
        }
    }

    fun add_service_internal(
        did_document_data: &mut DIDDocument,
        fragment: String,
        service_type: String,
        service_endpoint: String,
        properties: SimpleMap<String, String>
    ) {
        assert!(!simple_map::contains_key(&did_document_data.services, &fragment),
            ErrorServiceAlreadyExists);

        let service_id = ServiceID {
            did: did_document_data.id,
            fragment: fragment,
        };

        let service = Service {
            id: service_id,
            type: service_type,
            service_endpoint,
            properties,
        };

        let properties_count = (simple_map::length(&properties) as u64);
        simple_map::add(&mut did_document_data.services, fragment, service);
        
        // Emit service added event
        event::emit(ServiceAddedEvent {
            did: format_did(&did_document_data.id),
            fragment: fragment,
            service_type: service_type,
            service_endpoint: service_endpoint,
            properties_count,
        });
    }

    public entry fun add_service_entry(
        did_signer: &signer,
        fragment: String,
        service_type: String,
        service_endpoint: String
    ) {
        // Use helper function to get authorized DID document
        let did_document_data = get_authorized_did_document_mut_for_invocation(did_signer);

        let properties = simple_map::new<String, String>();
        add_service_internal(did_document_data, fragment, service_type, service_endpoint, properties);
    }

    public entry fun add_service_with_properties_entry(
        did_signer: &signer,
        fragment: String,
        service_type: String,
        service_endpoint: String,
        property_keys: vector<String>,
        property_values: vector<String>
    ) {
        assert!(vector::length(&property_keys) == vector::length(&property_values),
            ErrorPropertyKeysValuesLengthMismatch);

        let properties = simple_map::new<String, String>();
        let i = 0;
        let len = vector::length(&property_keys);
        while (i < len) {
            simple_map::add(&mut properties, *vector::borrow(&property_keys, i), *vector::borrow(&property_values, i));
            i = i + 1;
        };

        // Derive DID identifier from signer address
        let signer_address = signer::address_of(did_signer);
        let did_identifier_str = address::to_bech32_string(signer_address);
        
        let object_id = resolve_did_object_id(&did_identifier_str);
        assert!(object::exists_object_with_type<DIDDocument>(object_id), ErrorDIDObjectNotFound);

        let did_doc_obj_ref_mut = object::borrow_mut_object_extend<DIDDocument>(object_id);
        let did_document_data = object::borrow_mut(did_doc_obj_ref_mut);

        assert_authorized_for_capability_invocation(did_document_data, did_signer);
        add_service_internal(did_document_data, fragment, service_type, service_endpoint, properties);
    }

    public entry fun update_service_entry(
        did_signer: &signer,
        fragment: String,
        new_service_type: String,
        new_service_endpoint: String,
        new_property_keys: vector<String>,
        new_property_values: vector<String>
    ) {
        assert!(vector::length(&new_property_keys) == vector::length(&new_property_values),
            ErrorPropertyKeysValuesLengthMismatch);

        let new_properties = simple_map::new<String, String>();
        let i = 0;
        let len = vector::length(&new_property_keys);
        while (i < len) {
            simple_map::add(&mut new_properties, *vector::borrow(&new_property_keys, i), *vector::borrow(&new_property_values, i));
            i = i + 1;
        };

        // Derive DID identifier from signer address
        let signer_address = signer::address_of(did_signer);
        let did_identifier_str = address::to_bech32_string(signer_address);
        
        let object_id = resolve_did_object_id(&did_identifier_str);
        assert!(object::exists_object_with_type<DIDDocument>(object_id), ErrorDIDObjectNotFound);

        let did_doc_obj_ref_mut = object::borrow_mut_object_extend<DIDDocument>(object_id);
        let did_document_data = object::borrow_mut(did_doc_obj_ref_mut);

        assert_authorized_for_capability_invocation(did_document_data, did_signer);

        assert!(simple_map::contains_key(&did_document_data.services, &fragment),
            ErrorServiceNotFound);

        // Store old service data for event
        let old_service = simple_map::borrow(&did_document_data.services, &fragment);
        let old_service_type = old_service.type;
        let old_service_endpoint = old_service.service_endpoint;

        let service_id = ServiceID {
            did: did_document_data.id,
            fragment: fragment,
        };
        let updated_service = Service {
            id: service_id,
            type: new_service_type,
            service_endpoint: new_service_endpoint,
            properties: new_properties,
        };

        let new_properties_count = (simple_map::length(&new_properties) as u64);
        let (_,_old_service) = simple_map::upsert(&mut did_document_data.services, fragment, updated_service);
        
        // Emit service updated event
        event::emit(ServiceUpdatedEvent {
            did: format_did(&did_document_data.id),
            fragment: fragment,
            old_service_type,
            new_service_type,
            old_service_endpoint,
            new_service_endpoint,
            new_properties_count,
        });
    }

    public entry fun remove_service_entry(
        did_signer: &signer,
        fragment: String
    ) {
        // Use helper function to get authorized DID document
        let did_document_data = get_authorized_did_document_mut_for_invocation(did_signer);

        assert!(simple_map::contains_key(&did_document_data.services, &fragment),
            ErrorServiceNotFound);

        // Store service data for event before removal
        let service_to_remove = simple_map::borrow(&did_document_data.services, &fragment);
        let removed_service_type = service_to_remove.type;

        simple_map::remove(&mut did_document_data.services, &fragment);
        
        // Emit service removed event
        event::emit(ServiceRemovedEvent {
            did: format_did(&did_document_data.id),
            fragment: fragment,
            service_type: removed_service_type,
        });
    }

    public fun exists_did_document_by_identifier(identifier_str: String): bool {
        let object_id = resolve_did_object_id(&identifier_str);
        object::exists_object_with_type<DIDDocument>(object_id)
    }

    public fun exists_did_for_address(addr: address): bool {
        let did_identifier = address::to_bech32_string(addr);
        let object_id = resolve_did_object_id(&did_identifier);
        object::exists_object_with_type<DIDDocument>(object_id)
    } 

    /// Get all DID ObjectIDs controlled by a specific controller DID
    public fun get_dids_by_controller(controller_did: DID): vector<String> {
        let controller_did_str = format_did(&controller_did);
        get_dids_by_controller_string(controller_did_str)
    }

    public fun get_dids_by_controller_string(controller_did_str: String): vector<String> {
        if (!object::exists_object_with_type<DIDRegistry>(did_registry_id())){
            return vector::empty<String>()
        };
        let registry = borrow_did_registry();
        if (!table::contains(&registry.controller_to_dids, controller_did_str)) {
            vector::empty<String>()
        } else {
            *table::borrow(&registry.controller_to_dids, controller_did_str)
        }
    }

    public fun has_verification_relationship_in_doc(
        did_document_data: &DIDDocument,
        fragment: &String,
        relationship_type: u8
    ): bool {
        if (!simple_map::contains_key(&did_document_data.verification_methods, fragment)) {
            return false
        };

        let relationship_vec_ref = if (relationship_type == VERIFICATION_RELATIONSHIP_AUTHENTICATION) {
            &did_document_data.authentication
        } else if (relationship_type == VERIFICATION_RELATIONSHIP_ASSERTION_METHOD) {
            &did_document_data.assertion_method
        } else if (relationship_type == VERIFICATION_RELATIONSHIP_CAPABILITY_INVOCATION) {
            &did_document_data.capability_invocation
        } else if (relationship_type == VERIFICATION_RELATIONSHIP_CAPABILITY_DELEGATION) {
            &did_document_data.capability_delegation
        } else if (relationship_type == VERIFICATION_RELATIONSHIP_KEY_AGREEMENT) {
            &did_document_data.key_agreement
        } else {
            return false
        };
        vector::contains(relationship_vec_ref, fragment)
    }

    public fun is_verification_method_valid_in_doc(
        did_document_data: &DIDDocument,
        fragment: &String
    ): bool {
        if (!simple_map::contains_key(&did_document_data.verification_methods, fragment)) {
            return false
        };

        true
    }

    // =================== DID functions ===================

    public fun format_did(did: &DID): String {
        let did_string = string::utf8(b"did:");
        string::append(&mut did_string, did.method);
        string::append_utf8(&mut did_string, b":");
        string::append(&mut did_string, did.identifier);
        did_string
    }

    public fun format_verification_method_id(id: &VerificationMethodID): String {
        let id_string = format_did(&id.did);
        string::append_utf8(&mut id_string, b"#");
        string::append(&mut id_string, id.fragment);
        id_string
    }

    public fun format_service_id(id: &ServiceID): String {
        let id_string = format_did(&id.did);
        string::append_utf8(&mut id_string, b"#");
        string::append(&mut id_string, id.fragment);
        id_string
    }
 
    /// Create a DID struct from method and identifier parts
    /// This function only constructs a DID struct, it does NOT create a DID object on-chain
    public fun new_did_from_parts(method: String, identifier: String): DID {
        DID {
            method,
            identifier,
        }
    } 

    /// Create a Rooch DID struct from an address
    /// This function only constructs a DID struct, it does NOT create a DID object on-chain
    public fun new_rooch_did_by_address(addr: address): DID {
        let did_identifier = address::to_bech32_string(addr);
        DID {
            method: string::utf8(b"rooch"),
            identifier: did_identifier,
        }
    }

    /// Parse a DID string in the format "did:method:identifier" into a DID struct
    public fun parse_did_string(did_string: &String): DID {
        let colon_bytes = b":";
        let did_bytes = string::bytes(did_string);
        
        // Find positions of colons
        let colon_positions = vector::empty<u64>();
        let i = 0;
        while (i < vector::length(did_bytes)) {
            if (*vector::borrow(did_bytes, i) == *vector::borrow(&colon_bytes, 0)) {
                vector::push_back(&mut colon_positions, i);
            };
            i = i + 1;
        };
        
        // Should have exactly 2 colons: "did:method:identifier"
        assert!(vector::length(&colon_positions) >= 2, ErrorInvalidDIDStringFormat);
        
        let first_colon_pos = *vector::borrow(&colon_positions, 0);
        let second_colon_pos = *vector::borrow(&colon_positions, 1);
        
        // Extract "did" part (should be "did")
        let did_part = string::sub_string(did_string, 0, first_colon_pos);
        assert!(did_part == string::utf8(b"did"), ErrorInvalidDIDStringFormat);
        
        // Extract method part
        let method = string::sub_string(did_string, first_colon_pos + 1, second_colon_pos);
        
        // Extract identifier part (everything after second colon)
        let identifier = string::sub_string(did_string, second_colon_pos + 1, string::length(did_string));
        
        // Validate that method and identifier are not empty
        assert!(string::length(&method) > 0, ErrorInvalidDIDStringFormat);
        assert!(string::length(&identifier) > 0, ErrorInvalidDIDStringFormat);
        
        DID {
            method,
            identifier,
        }
    }

    /// Get the identifier from a DID
    public fun get_did_identifier_string(did: &DID): String {
        did.identifier
    }

    /// Get the method from a DID
    public fun get_did_method(did: &DID): String {
        did.method
    }

    // =================== Document getters ===================

    public fun get_did_document(did_str: String): &DIDDocument {
        let did = parse_did_string(&did_str);
        let object_id = resolve_did_object_id(&did.identifier);
        get_did_document_by_object_id(object_id)
    }

    /// Get DIDDocument by address
    public fun get_did_document_by_address(addr: address): &DIDDocument {
        let did_identifier = address::to_bech32_string(addr);
        let object_id = resolve_did_object_id(&did_identifier);
        get_did_document_by_object_id(object_id)
    }

    /// Get DIDDocument by ObjectID
    public fun get_did_document_by_object_id(object_id: ObjectID): &DIDDocument {
        assert!(object::exists_object_with_type<DIDDocument>(object_id), ErrorDIDDocumentNotExist);
        let did_doc_obj_ref = object::borrow_object<DIDDocument>(object_id);
        object::borrow(did_doc_obj_ref)
    }

    /// Get id from DIDDocument
    public fun doc_id(did_doc: &DIDDocument): &DID {
        &did_doc.id
    }

    /// Get controllers from DIDDocument
    public fun doc_controllers(did_doc: &DIDDocument): &vector<DID> {
        &did_doc.controller
    }

    /// Get verification methods from DIDDocument
    public fun doc_verification_methods(did_doc: &DIDDocument): &SimpleMap<String, VerificationMethod> {
        &did_doc.verification_methods
    }

    /// Get verification method by fragment
    public fun doc_verification_method(did_doc: &DIDDocument, fragment: &String): Option<VerificationMethod> {
        if (simple_map::contains_key(&did_doc.verification_methods, fragment)) {
            option::some(*simple_map::borrow(&did_doc.verification_methods, fragment))
        } else {
            option::none()
        }
    }

    public fun verification_method_id(vm: &VerificationMethod): &VerificationMethodID {
        &vm.id
    }

    public fun verification_method_type(vm: &VerificationMethod): &String {
        &vm.type
    }

    public fun verification_method_controller(vm: &VerificationMethod): &DID {
        &vm.controller
    }

    public fun verification_method_public_key_multibase(vm: &VerificationMethod): &String {
        &vm.public_key_multibase
    }

    /// Get authentication methods from DIDDocument
    public fun doc_authentication_methods(did_doc: &DIDDocument): &vector<String> {
        &did_doc.authentication
    }

    /// Get assertion methods from DIDDocument
    public fun doc_assertion_methods(did_doc: &DIDDocument): &vector<String> {
        &did_doc.assertion_method
    }

    /// Get capability invocation methods from DIDDocument
    public fun doc_capability_invocation_methods(did_doc: &DIDDocument): &vector<String> {
        &did_doc.capability_invocation
    }

    /// Get capability delegation methods from DIDDocument
    public fun doc_capability_delegation_methods(did_doc: &DIDDocument): &vector<String> {
        &did_doc.capability_delegation
    }

    /// Get key agreement methods from DIDDocument
    public fun doc_key_agreement_methods(did_doc: &DIDDocument): &vector<String> {
        &did_doc.key_agreement
    }

    /// Get services from DIDDocument
    public fun doc_services(did_doc: &DIDDocument): &SimpleMap<String, Service> {
        &did_doc.services
    }

    /// Get service by fragment
    public fun doc_service(did_doc: &DIDDocument, fragment: &String): Option<Service> {
        if (simple_map::contains_key(&did_doc.services, fragment)) {
            option::some(*simple_map::borrow(&did_doc.services, fragment))
        } else {
            option::none()
        }
    }

    public fun service_id(service: &Service): &ServiceID {
        &service.id
    }

    public fun service_type(service: &Service): &String {
        &service.type
    }

    public fun service_endpoint(service: &Service): &String {
        &service.service_endpoint
    }

    public fun service_properties(service: &Service): &SimpleMap<String, String> {
        &service.properties
    }

    /// Get also known as from DIDDocument
    public fun doc_also_known_as(did_doc: &DIDDocument): &vector<String> {
        &did_doc.also_known_as
    }

    /// Get created timestamp from Object system
    /// This accesses the Object's metadata created_at timestamp
    public fun get_created_timestamp_by_object_id(object_id: ObjectID): u64 {
        object::created_at(object_id)
    }

    /// Get updated timestamp from Object system  
    /// This accesses the Object's metadata updated_at timestamp
    public fun get_updated_timestamp_by_object_id(object_id: ObjectID): u64 {
        object::updated_at(object_id)
    }


    public fun get_did_address(did_doc: &DIDDocument): address {
        account::account_cap_address(&did_doc.account_cap)
    }

    // =================== Internal helper functions ===================

    /// Helper function to get a mutable reference to DIDDocument with capability delegation authorization
    /// This combines common patterns: resolve ObjectID, check existence, borrow mutable, and verify permissions
    fun get_authorized_did_document_mut_for_delegation(did_signer: &signer): &mut DIDDocument {
        let object_id = resolve_did_object_from_signer(did_signer);
        assert!(object::exists_object_with_type<DIDDocument>(object_id), ErrorDIDObjectNotFound);
        
        let did_doc_obj_ref_mut = object::borrow_mut_object_extend<DIDDocument>(object_id);
        let did_document_data = object::borrow_mut(did_doc_obj_ref_mut);
        
        assert_authorized_for_capability_delegation(did_document_data, did_signer);
        did_document_data
    }

    /// Helper function to get a mutable reference to DIDDocument with capability invocation authorization
    /// This combines common patterns: resolve ObjectID, check existence, borrow mutable, and verify permissions
    fun get_authorized_did_document_mut_for_invocation(did_signer: &signer): &mut DIDDocument {
        let object_id = resolve_did_object_from_signer(did_signer);
        assert!(object::exists_object_with_type<DIDDocument>(object_id), ErrorDIDObjectNotFound);
        
        let did_doc_obj_ref_mut = object::borrow_mut_object_extend<DIDDocument>(object_id);
        let did_document_data = object::borrow_mut(did_doc_obj_ref_mut);
        
        assert_authorized_for_capability_invocation(did_document_data, did_signer);
        did_document_data
    }
    
    /// Assert that the signer has capabilityDelegation permission for DID document management
    /// This includes key management and verification relationship modifications
    fun assert_authorized_for_capability_delegation(
        did_document_data: &DIDDocument,
        did_signer: &signer
    ) {
        let sender = signer::address_of(did_signer);
        let did_account_address = account::account_cap_address(&did_document_data.account_cap);
        
        // 1. Verify signer is the DID's associated account
        assert!(sender == did_account_address, ErrorSignerNotDIDAccount);
        
        // 2. Get current transaction's session key (authentication_key)
        let session_key_opt = auth_validator::get_session_key_from_ctx_option();
        assert!(option::is_some(&session_key_opt), ErrorNoSessionKeyInContext);
        
        let session_key = option::extract(&mut session_key_opt);
        
        // 3. Find the verification method corresponding to this session key
        let vm_fragment_opt = find_verification_method_by_session_key(did_document_data, &session_key);
        assert!(option::is_some(&vm_fragment_opt), ErrorSessionKeyNotFound);
        
        let vm_fragment = option::extract(&mut vm_fragment_opt);
        
        // 4. Check if this verification method has capabilityDelegation permission
        assert!(
            vector::contains(&did_document_data.capability_delegation, &vm_fragment),
            ErrorInsufficientPermission
        );
    }

    /// Assert that the signer has capabilityInvocation permission for service management
    fun assert_authorized_for_capability_invocation(
        did_document_data: &DIDDocument,
        did_signer: &signer
    ) {
        let sender = signer::address_of(did_signer);
        let did_account_address = account::account_cap_address(&did_document_data.account_cap);
        
        // 1. Verify signer is the DID's associated account
        assert!(sender == did_account_address, ErrorSignerNotDIDAccount);
        
        // 2. Get current transaction's session key (authentication_key)
        let session_key_opt = auth_validator::get_session_key_from_ctx_option();
        assert!(option::is_some(&session_key_opt), ErrorNoSessionKeyInContext);
        
        let session_key = option::extract(&mut session_key_opt);
        
        // 3. Find the verification method corresponding to this session key
        let vm_fragment_opt = find_verification_method_by_session_key(did_document_data, &session_key);
        assert!(option::is_some(&vm_fragment_opt), ErrorSessionKeyNotFound);
        
        let vm_fragment = option::extract(&mut vm_fragment_opt);
        
        // 4. Check if this verification method has capabilityInvocation permission
        assert!(
            vector::contains(&did_document_data.capability_invocation, &vm_fragment),
            ErrorInsufficientPermission
        );
    }

    /// Find the verification method fragment that corresponds to the given session key
    /// Returns None if no matching verification method is found
    public fun find_verification_method_by_session_key(
        did_document_data: &DIDDocument,
        session_key: &vector<u8>
    ): Option<String> {
        // Iterate through all verification methods in the authentication relationship
        let auth_methods = &did_document_data.authentication;
        let i = 0;
        
        while (i < vector::length(auth_methods)) {
            let fragment = vector::borrow(auth_methods, i);
            
            if (simple_map::contains_key(&did_document_data.verification_methods, fragment)) {
                let vm = simple_map::borrow(&did_document_data.verification_methods, fragment);
                let pk_bytes_opt = multibase_codec::decode(&vm.public_key_multibase);
                //This should never happen, but just in case
                assert!(option::is_some(&pk_bytes_opt), ErrorInvalidPublicKeyMultibaseFormat);
                let pk_bytes = option::destroy_some(pk_bytes_opt);

                let derived_auth_key = if (vm.type == string::utf8(VERIFICATION_METHOD_TYPE_ED25519)) {
                    session_key::ed25519_public_key_to_authentication_key(&pk_bytes)
                } else if (vm.type == string::utf8(VERIFICATION_METHOD_TYPE_SECP256K1)) {
                    session_key::secp256k1_public_key_to_authentication_key(&pk_bytes)
                } else {
                    session_key::secp256r1_public_key_to_authentication_key(&pk_bytes)
                };
                if (derived_auth_key == *session_key) {
                    return option::some(*fragment)
                };
            };
            i = i + 1;
        };
        option::none<String>()
    }

    /// Add a verification method to the authentication relationship and register it as a session key.
    /// This function supports only verification method types that can be registered as session keys:
    /// Ed25519, Secp256k1 and Secp256r1 verification methods.
    fun add_authentication_method(
        did_document_data: &mut DIDDocument,
        fragment: String,
        method_type: String,
        public_key_multibase: String
    ) {
        // Ensure the method type is supported for session keys
        assert!(
            method_type == string::utf8(VERIFICATION_METHOD_TYPE_ED25519) ||
            method_type == string::utf8(VERIFICATION_METHOD_TYPE_SECP256K1) ||
            method_type == string::utf8(VERIFICATION_METHOD_TYPE_SECP256R1),
            ErrorUnsupportedAuthKeyTypeForSessionKey
        );
        
        // 1. Add the verification method if it doesn't exist
        if (!simple_map::contains_key(&did_document_data.verification_methods, &fragment)) {
            let vm_id = VerificationMethodID {
                did: did_document_data.id,
                fragment: fragment,
            };
            
            let vm = VerificationMethod {
                id: vm_id,
                type: method_type,
                controller: did_document_data.id,
                public_key_multibase,
            };
            
            simple_map::add(&mut did_document_data.verification_methods, fragment, vm);
        };
        
        // 2. Add to authentication relationship if not already present
        if (!vector::contains(&did_document_data.authentication, &fragment)) {
            vector::push_back(&mut did_document_data.authentication, fragment);
            
            // 3. Register as session key
            internal_ensure_session_key(
                did_document_data,
                fragment,
                public_key_multibase,
                method_type
            );
        };
    }


    fun internal_remove_session_key(account_signer: &signer, vm_public_key_multibase: &String, vm_type: &String) {
        let pk_bytes_opt = multibase_codec::decode(vm_public_key_multibase);
        assert!(option::is_some(&pk_bytes_opt), ErrorInvalidPublicKeyMultibaseFormat);
        let pk_bytes = option::destroy_some(pk_bytes_opt);

        let auth_key_for_session = if (vm_type == &string::utf8(VERIFICATION_METHOD_TYPE_ED25519)) {
            session_key::ed25519_public_key_to_authentication_key(&pk_bytes)
        } else if (vm_type == &string::utf8(VERIFICATION_METHOD_TYPE_SECP256K1)) {
            session_key::secp256k1_public_key_to_authentication_key(&pk_bytes)
        } else if (vm_type == &string::utf8(VERIFICATION_METHOD_TYPE_SECP256R1)) {
            session_key::secp256r1_public_key_to_authentication_key(&pk_bytes)
        } else {
            abort ErrorUnsupportedAuthKeyTypeForSessionKey
        };

        session_key::remove_session_key(account_signer, auth_key_for_session);
    }

    /// Private helper function to register a verification method as a Rooch session key.
    /// This function supports both Ed25519 and Secp256k1 key types.
    fun internal_ensure_session_key(
        did_document_data: &mut DIDDocument,
        vm_fragment: String,
        vm_public_key_multibase: String,
        vm_type: String,
    ) {
        // Decode the raw public key (no multicodec prefix)
        let pk_bytes_opt = multibase_codec::decode(&vm_public_key_multibase);
        assert!(option::is_some(&pk_bytes_opt), ErrorInvalidPublicKeyMultibaseFormat);
        let pk_bytes = option::destroy_some(pk_bytes_opt);

        let associated_account_signer = account::create_signer_with_account_cap(&mut did_document_data.account_cap);

        let max_inactive_interval_for_sk = session_key::max_inactive_interval();

        let app_name = string::utf8(b"did_authentication_key:");
        string::append(&mut app_name, vm_fragment);
        let app_url = format_did(&did_document_data.id);

        let associated_address = signer::address_of(&associated_account_signer);
        
        // TODO: Consider limiting session key scope to specific modules/functions for better security
        // Current implementation uses wildcards for maximum flexibility during development
        let did_addr_scope = session_key::new_session_scope(
            associated_address,       
            string::utf8(b"*"),        
            string::utf8(b"*") 
        );
        let rooch_framework_scope = session_key::new_session_scope(
            @rooch_framework,
            string::utf8(b"*"),       
            string::utf8(b"*") 
        );
        let scopes_for_sk = vector[rooch_framework_scope, did_addr_scope];

        // Generate the authentication key based on the verification method type
        let auth_key_for_session = if (vm_type == string::utf8(VERIFICATION_METHOD_TYPE_ED25519)) {
            session_key::ed25519_public_key_to_authentication_key(&pk_bytes)
        } else if (vm_type == string::utf8(VERIFICATION_METHOD_TYPE_SECP256K1)) {
            session_key::secp256k1_public_key_to_authentication_key(&pk_bytes)
        } else { // Must be SECP256R1
            session_key::secp256r1_public_key_to_authentication_key(&pk_bytes)
        };

        session_key::create_session_key_internal(
            &associated_account_signer,
            app_name,
            app_url,
            auth_key_for_session,
            scopes_for_sk,
            max_inactive_interval_for_sk
        );
    }

    // =================== Test-only functions ===================
 

    #[test_only]
    /// Test-only function to check if verification method exists in document
    public fun test_verification_method_exists(did_document_data: &DIDDocument, fragment: &String): bool {
        simple_map::contains_key(&did_document_data.verification_methods, fragment)
    }

    #[test_only]
    /// Test-only function to check if service exists in document
    public fun test_service_exists(did_document_data: &DIDDocument, fragment: &String): bool {
        simple_map::contains_key(&did_document_data.services, fragment)
    } 

} 
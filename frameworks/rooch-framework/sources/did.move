// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_framework::did {
    use std::vector;
    use std::string::{Self, String};
    use std::option::{Self, Option};
    use std::error;
    use std::signer;
    use moveos_std::simple_map::{Self, SimpleMap};
    use moveos_std::table::{Self, Table};
    use moveos_std::account::{Self, AccountCap};
    use moveos_std::object::{Self, Object, ObjectID};
    use moveos_std::tx_context::{Self, TxContext};
    use moveos_std::timestamp;
    use moveos_std::core_addresses;
    use moveos_std::address;
    use moveos_std::multibase;
    use rooch_framework::session_key;

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

    // Verification relationship types
    const VERIFICATION_RELATIONSHIP_AUTHENTICATION: u8 = 0;
    const VERIFICATION_RELATIONSHIP_ASSERTION_METHOD: u8 = 1;
    const VERIFICATION_RELATIONSHIP_CAPABILITY_INVOCATION: u8 = 2;
    const VERIFICATION_RELATIONSHIP_CAPABILITY_DELEGATION: u8 = 3;
    const VERIFICATION_RELATIONSHIP_KEY_AGREEMENT: u8 = 4;

    // Verification method types
    const VERIFICATION_METHOD_TYPE_ED25519: vector<u8> = b"Ed25519VerificationKey2020";

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
        account_cap: AccountCap,
        also_known_as: vector<String>,
        created_timestamp: u64,
        updated_timestamp: u64,
    }

    /// Registry to store mappings. This is a Named Object.
    struct DIDRegistry has key {
        controller_to_dids: Table<DID, vector<ObjectID>>, // Controller DID -> DID Document ObjectIDs it controls
    }

    /// Returns the fixed ObjectID for the DIDRegistry.
    fun did_registry_id(): ObjectID {
        object::named_object_id<DIDRegistry>()
    }

    /// Initialize the DID system - called only once at genesis or by system upgrade.
    /// The signer must be a system reserved account.
    public entry fun init_did_registry(system_signer: &signer) {
        core_addresses::assert_system_reserved(system_signer);
        let registry_id = did_registry_id();
        assert!(!object::exists_object_with_type<DIDRegistry>(registry_id), ErrorDIDRegistryAlreadyInitialized);
        
        let registry_data = DIDRegistry {
            controller_to_dids: table::new<DID, vector<ObjectID>>(),
        };
        // Named objects are created in the context of the sender, then transferred.
        let registry_object = object::new_named_object(registry_data);
        object::transfer_extend(registry_object, @rooch_framework);
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

    /// Create a new DID Object.
    /// The method is fixed to "rooch".
    /// The identifier is derived from the newly created associated Rooch account address.
    public fun create_did_object(
        // We keep the creator account signer for future use, but it is not used in this function
        _creator_account_signer: &signer,
        initial_controllers: vector<DID>,
        initial_vm_type: String,
        initial_vm_pk_multibase: String,
        initial_vm_fragment: String
    ): ObjectID {
        let registry = borrow_mut_did_registry();
        assert!(vector::length(&initial_controllers) > 0, error::invalid_argument(ErrorNoControllersSpecified));

        // Validate did:key controllers according to NIP-1
        validate_did_key_controllers(&initial_controllers, &initial_vm_pk_multibase);

        let new_account_cap = account::create_account_and_return_cap();
        let new_rooch_address = account::account_cap_address(&new_account_cap);
        
        let did_method_val = string::utf8(b"rooch");
        let did_identifier_string = address::to_bech32_string(new_rooch_address);
        
        let new_object_id = resolve_did_object_id(&did_identifier_string);
        assert!(!object::exists_object_with_type<DIDDocument>(new_object_id), error::already_exists(ErrorDIDAlreadyExists));
        
        let did = DID {
            method: did_method_val,
            identifier: did_identifier_string,
        };

        let initial_vm_fragment_value = initial_vm_fragment;

        let initial_vm_id = VerificationMethodID {
            did: did,
            fragment: initial_vm_fragment_value,
        };
        let initial_vm = VerificationMethod {
            id: initial_vm_id,
            type: initial_vm_type,
            controller: did,
            public_key_multibase: initial_vm_pk_multibase,
        };

        let now = timestamp::now_seconds();

        // Create base DIDDocument structure
        let did_document_data = DIDDocument {
            id: did,
            controller: initial_controllers,
            verification_methods: simple_map::new<String, VerificationMethod>(),
            authentication: vector::empty<String>(),
            assertion_method: vector::empty<String>(),
            capability_invocation: vector::empty<String>(),
            capability_delegation: vector::empty<String>(),
            key_agreement: vector::empty<String>(),
            services: simple_map::new<String, Service>(),
            account_cap: new_account_cap,
            also_known_as: vector::empty<String>(),
            created_timestamp: now,
            updated_timestamp: now,
        };

        // Populate verification methods and relationships based on type
        if (initial_vm_type == string::utf8(VERIFICATION_METHOD_TYPE_ED25519)) {
            // For Ed25519: use specialized function that handles authentication and session key
            add_ed25519_authentication_method(
                &mut did_document_data,
                initial_vm_fragment_value,
                initial_vm_pk_multibase
            );
            
            // Add to other relationships as needed
            vector::push_back(&mut did_document_data.assertion_method, initial_vm_fragment_value);
            vector::push_back(&mut did_document_data.capability_invocation, initial_vm_fragment_value);
            vector::push_back(&mut did_document_data.capability_delegation, initial_vm_fragment_value);
        } else {
            // For non-Ed25519: use standard verification method creation
            simple_map::add(&mut did_document_data.verification_methods, initial_vm_fragment_value, initial_vm);
            
            // Add to all default relationships
            vector::push_back(&mut did_document_data.authentication, initial_vm_fragment_value);
            vector::push_back(&mut did_document_data.assertion_method, initial_vm_fragment_value);
            vector::push_back(&mut did_document_data.capability_invocation, initial_vm_fragment_value);
            vector::push_back(&mut did_document_data.capability_delegation, initial_vm_fragment_value);
        };

        let did_object = object::new_with_id(new_object_id, did_document_data);
        object::transfer_extend(did_object, new_rooch_address);
        
        // Add the new DID to all its controllers' lists in the registry
        let i = 0;
        while (i < vector::length(&initial_controllers)) {
            let controller_did = *vector::borrow(&initial_controllers, i);
            if (!table::contains(&registry.controller_to_dids, controller_did)) {
                table::add(&mut registry.controller_to_dids, controller_did, vector::empty<ObjectID>());
            };
            let controller_dids = table::borrow_mut(&mut registry.controller_to_dids, controller_did);
            vector::push_back(controller_dids, new_object_id);
            i = i + 1;
        };
        
        new_object_id
    }

    public entry fun create_did_object_entry(
        creator_account_signer: &signer,
        initial_controller_did_strings: vector<String>,
        initial_vm_type: String,
        initial_vm_pk_multibase: String,
        initial_vm_fragment: String
    ) {
        assert!(vector::length(&initial_controller_did_strings) > 0, error::invalid_argument(ErrorNoControllersSpecified));

        let initial_controllers = vector::empty<DID>();
        let i = 0;
        while (i < vector::length(&initial_controller_did_strings)) {
            let did_string = vector::borrow(&initial_controller_did_strings, i);
            let parsed_did = parse_did_string(did_string);
            vector::push_back(&mut initial_controllers, parsed_did);
            i = i + 1;
        };

        let _ = create_did_object(
            creator_account_signer,
            initial_controllers,
            initial_vm_type,
            initial_vm_pk_multibase,
            initial_vm_fragment
        );
    }

    public entry fun add_verification_method_entry(
        did_identifier_str: String,
        fragment: String,
        method_type: String,
        public_key_multibase: String
    ) {
        let object_id = resolve_did_object_id(&did_identifier_str);
        assert!(object::exists_object_with_type<DIDDocument>(object_id), error::not_found(ErrorDIDObjectNotFound));
        
        let did_doc_obj_ref_mut = object::borrow_mut_object_extend<DIDDocument>(object_id);
        let did_document_data = object::borrow_mut(did_doc_obj_ref_mut);

        assert_authorized_controller(&did_document_data.controller);

        assert!(!simple_map::contains_key(&did_document_data.verification_methods, &fragment),
            error::already_exists(ErrorVerificationMethodAlreadyExists));

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
        did_document_data.updated_timestamp = timestamp::now_seconds();
    }

    public entry fun remove_verification_method_entry(
        did_identifier_str: String,
        fragment: String
    ) {
        let object_id = resolve_did_object_id(&did_identifier_str);
        assert!(object::exists_object_with_type<DIDDocument>(object_id), error::not_found(ErrorDIDObjectNotFound));
        
        let did_doc_obj_ref_mut = object::borrow_mut_object_extend<DIDDocument>(object_id);
        let did_document_data = object::borrow_mut(did_doc_obj_ref_mut);

        assert_authorized_controller(&did_document_data.controller);

        assert!(simple_map::contains_key(&did_document_data.verification_methods, &fragment),
            error::not_found(ErrorVerificationMethodNotFound));

        if (vector::contains(&did_document_data.authentication, &fragment)) {
            let vm_to_remove = simple_map::borrow(&did_document_data.verification_methods, &fragment);
            // If the verification method is an Ed25519 key, we need to remove the session key
            if (vm_to_remove.type == string::utf8(VERIFICATION_METHOD_TYPE_ED25519)) {
                let pk_bytes_opt = multibase::decode_ed25519_key(&vm_to_remove.public_key_multibase);
                assert!(option::is_some(&pk_bytes_opt), ErrorInvalidPublicKeyMultibaseFormat);
                let pk_bytes = option::destroy_some(pk_bytes_opt);

                // Use the public function from session_key module to derive the auth key
                let auth_key_for_session = session_key::ed25519_public_key_to_authentication_key(&pk_bytes);

                let associated_account_signer = account::create_signer_with_account_cap(&mut did_document_data.account_cap);
                
                session_key::remove_session_key(&associated_account_signer, auth_key_for_session);
            };
        };

        remove_from_verification_relationship_internal(&mut did_document_data.authentication, &fragment);
        remove_from_verification_relationship_internal(&mut did_document_data.assertion_method, &fragment);
        remove_from_verification_relationship_internal(&mut did_document_data.capability_invocation, &fragment);
        remove_from_verification_relationship_internal(&mut did_document_data.capability_delegation, &fragment);
        remove_from_verification_relationship_internal(&mut did_document_data.key_agreement, &fragment);

        simple_map::remove(&mut did_document_data.verification_methods, &fragment);
        did_document_data.updated_timestamp = timestamp::now_seconds();
    }

    fun remove_from_verification_relationship_internal(
        relationship_vec: &mut vector<String>,
        fragment_to_remove: &String
    ) : bool {
        let removed = vector::remove_value(relationship_vec, fragment_to_remove);
        vector::length(&removed) > 0
    }

    public entry fun add_to_verification_relationship_entry(
        did_identifier_str: String,
        fragment: String,
        relationship_type: u8
    ) {
        let object_id = resolve_did_object_id(&did_identifier_str);
        assert!(object::exists_object_with_type<DIDDocument>(object_id), error::not_found(ErrorDIDObjectNotFound));
        
        let did_doc_obj_ref_mut = object::borrow_mut_object_extend<DIDDocument>(object_id);
        let did_document_data = object::borrow_mut(did_doc_obj_ref_mut);

        assert_authorized_controller(&did_document_data.controller);

        assert!(simple_map::contains_key(&did_document_data.verification_methods, &fragment),
            error::not_found(ErrorVerificationMethodNotFound));

        let target_relationship_vec_mut = if (relationship_type == VERIFICATION_RELATIONSHIP_AUTHENTICATION) {
            // Special handling for AUTHENTICATION: if this is an Ed25519 verification method,
            // use the specialized function that also registers it as a rooch session key
            let vm = *simple_map::borrow(&did_document_data.verification_methods, &fragment);
            if (vm.type == string::utf8(VERIFICATION_METHOD_TYPE_ED25519)) {
                add_ed25519_authentication_method(
                    did_document_data,
                    fragment,
                    vm.public_key_multibase
                );
                // Return early since add_ed25519_authentication_method handles the relationship addition
                did_document_data.updated_timestamp = timestamp::now_seconds();
                return
            };

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
            abort error::invalid_argument(ErrorInvalidVerificationRelationship)
        };

        if (!vector::contains(target_relationship_vec_mut, &fragment)) {
            vector::push_back(target_relationship_vec_mut, fragment);
            did_document_data.updated_timestamp = timestamp::now_seconds();
        }
    }

    public entry fun remove_from_verification_relationship_entry(
        did_identifier_str: String,
        fragment: String,
        relationship_type: u8
    ) {
        let object_id = resolve_did_object_id(&did_identifier_str);
        assert!(object::exists_object_with_type<DIDDocument>(object_id), error::not_found(ErrorDIDObjectNotFound));

        let did_doc_obj_ref_mut = object::borrow_mut_object_extend<DIDDocument>(object_id);
        let did_document_data = object::borrow_mut(did_doc_obj_ref_mut);

        assert_authorized_controller(&did_document_data.controller);

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
            abort error::invalid_argument(ErrorInvalidVerificationRelationship)
        };

        let original_len = vector::length(target_relationship_vec_mut);
        remove_from_verification_relationship_internal(target_relationship_vec_mut, &fragment);
        if (vector::length(target_relationship_vec_mut) < original_len) {
            did_document_data.updated_timestamp = timestamp::now_seconds();
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
            error::already_exists(ErrorServiceAlreadyExists));

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

        simple_map::add(&mut did_document_data.services, fragment, service);
        did_document_data.updated_timestamp = timestamp::now_seconds();
    }

    public entry fun add_service_entry(
        did_identifier_str: String,
        fragment: String,
        service_type: String,
        service_endpoint: String
    ) {
        let object_id = resolve_did_object_id(&did_identifier_str);
        assert!(object::exists_object_with_type<DIDDocument>(object_id), error::not_found(ErrorDIDObjectNotFound));

        let did_doc_obj_ref_mut = object::borrow_mut_object_extend<DIDDocument>(object_id);
        let did_document_data = object::borrow_mut(did_doc_obj_ref_mut);

        assert_authorized_controller(&did_document_data.controller);

        let properties = simple_map::new<String, String>();
        add_service_internal(did_document_data, fragment, service_type, service_endpoint, properties);
    }

    public entry fun add_service_with_properties_entry(
        did_identifier_str: String,
        fragment: String,
        service_type: String,
        service_endpoint: String,
        property_keys: vector<String>,
        property_values: vector<String>
    ) {
        assert!(vector::length(&property_keys) == vector::length(&property_values),
            error::invalid_argument(ErrorPropertyKeysValuesLengthMismatch));

        let properties = simple_map::new<String, String>();
        let i = 0;
        let len = vector::length(&property_keys);
        while (i < len) {
            simple_map::add(&mut properties, *vector::borrow(&property_keys, i), *vector::borrow(&property_values, i));
            i = i + 1;
        };

        let object_id = resolve_did_object_id(&did_identifier_str);
        assert!(object::exists_object_with_type<DIDDocument>(object_id), error::not_found(ErrorDIDObjectNotFound));

        let did_doc_obj_ref_mut = object::borrow_mut_object_extend<DIDDocument>(object_id);
        let did_document_data = object::borrow_mut(did_doc_obj_ref_mut);

        assert_authorized_controller(&did_document_data.controller);
        add_service_internal(did_document_data, fragment, service_type, service_endpoint, properties);
    }

    public entry fun update_service_entry(
        did_identifier_str: String,
        fragment: String,
        new_service_type: String,
        new_service_endpoint: String,
        new_property_keys: vector<String>,
        new_property_values: vector<String>
    ) {
        assert!(vector::length(&new_property_keys) == vector::length(&new_property_values),
            error::invalid_argument(ErrorPropertyKeysValuesLengthMismatch));

        let new_properties = simple_map::new<String, String>();
        let i = 0;
        let len = vector::length(&new_property_keys);
        while (i < len) {
            simple_map::add(&mut new_properties, *vector::borrow(&new_property_keys, i), *vector::borrow(&new_property_values, i));
            i = i + 1;
        };

        let object_id = resolve_did_object_id(&did_identifier_str);
        assert!(object::exists_object_with_type<DIDDocument>(object_id), error::not_found(ErrorDIDObjectNotFound));

        let did_doc_obj_ref_mut = object::borrow_mut_object_extend<DIDDocument>(object_id);
        let did_document_data = object::borrow_mut(did_doc_obj_ref_mut);

        assert_authorized_controller(&did_document_data.controller);

        assert!(simple_map::contains_key(&did_document_data.services, &fragment),
            error::not_found(ErrorServiceNotFound));

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

        let (_,_old_service) = simple_map::upsert(&mut did_document_data.services, fragment, updated_service);
        did_document_data.updated_timestamp = timestamp::now_seconds();
    }

    public entry fun remove_service_entry(
        did_identifier_str: String,
        fragment: String
    ) {
        let object_id = resolve_did_object_id(&did_identifier_str);
        assert!(object::exists_object_with_type<DIDDocument>(object_id), error::not_found(ErrorDIDObjectNotFound));
        
        let did_doc_obj_ref_mut = object::borrow_mut_object_extend<DIDDocument>(object_id);
        let did_document_data = object::borrow_mut(did_doc_obj_ref_mut);

        assert_authorized_controller(&did_document_data.controller);

        assert!(simple_map::contains_key(&did_document_data.services, &fragment),
            error::not_found(ErrorServiceNotFound));

        simple_map::remove(&mut did_document_data.services, &fragment);
        did_document_data.updated_timestamp = timestamp::now_seconds();
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

    public fun get_did_for_address(addr: address): DID {
        assert!(exists_did_for_address(addr), error::not_found(ErrorDIDDocumentNotExist));
        let did_identifier = address::to_bech32_string(addr);
        DID {
            method: string::utf8(b"rooch"),
            identifier: did_identifier,
        }
    }

    /// Get all DID ObjectIDs controlled by a specific controller DID
    public fun get_dids_by_controller(controller_did: DID): vector<ObjectID> {
        if (!object::exists_object_with_type<DIDRegistry>(did_registry_id())){
            return vector::empty<ObjectID>()
        };
        let registry = borrow_did_registry();
        if (!table::contains(&registry.controller_to_dids, controller_did)) {
            vector::empty<ObjectID>()
        } else {
            *table::borrow(&registry.controller_to_dids, controller_did)
        }
    }

    public fun get_dids_by_controller_string(controller_did_str: String): vector<ObjectID> {
        let controller_did = parse_did_string(&controller_did_str);
        get_dids_by_controller(controller_did)
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

    public fun create_did_from_parts(method: String, identifier: String): DID {
        DID {
            method,
            identifier,
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
        assert!(vector::length(&colon_positions) >= 2, error::invalid_argument(ErrorInvalidDIDStringFormat));
        
        let first_colon_pos = *vector::borrow(&colon_positions, 0);
        let second_colon_pos = *vector::borrow(&colon_positions, 1);
        
        // Extract "did" part (should be "did")
        let did_part = string::sub_string(did_string, 0, first_colon_pos);
        assert!(did_part == string::utf8(b"did"), error::invalid_argument(ErrorInvalidDIDStringFormat));
        
        // Extract method part
        let method = string::sub_string(did_string, first_colon_pos + 1, second_colon_pos);
        
        // Extract identifier part (everything after second colon)
        let identifier = string::sub_string(did_string, second_colon_pos + 1, string::length(did_string));
        
        // Validate that method and identifier are not empty
        assert!(string::length(&method) > 0, error::invalid_argument(ErrorInvalidDIDStringFormat));
        assert!(string::length(&identifier) > 0, error::invalid_argument(ErrorInvalidDIDStringFormat));
        
        DID {
            method,
            identifier,
        }
    }

    // This is a complex check potentially requiring resolving controller DIDs.
    fun assert_authorized_controller(controllers: &vector<DID>) {
        let _sender = tx_context::sender(); // Sender is now fetched here for auth logic
        // TODO: Implement full controller and capabilityDelegation check using _sender against controllers.
        // This is a CRITICAL security function.
        // For now, this function is a placeholder and DOES NOT provide any real security.
        // Actual implementation would involve:
        // 1. For each controller_did in `controllers`:
        //    a. Resolve the controller_did to its DIDDocument object.
        //    b. Check if `_sender` corresponds to any verification method in the controller_did's document
        //       that is listed in its `capabilityDelegation` relationship.
        //    c. If such a valid, non-expired verification method is found, authorization is granted.
        // 2. If no controller grants authorization, abort with ErrorControllerPermissionDenied.
        // assert!(vector::length(controllers) > 0, error::permission_denied(ErrorControllerPermissionDenied)); // Ensure this NON-SECURE placeholder is removed or replaced
    }

    // New private helper function to register a VM as a Rooch session key
    fun internal_ensure_rooch_session_key(
        did_document_data: &mut DIDDocument,
        vm_fragment: String,
        vm_type: String,
        vm_public_key_multibase: String,
    ) {
        assert!(vm_type == string::utf8(VERIFICATION_METHOD_TYPE_ED25519), ErrorUnsupportedAuthKeyTypeForSessionKey);

        let pk_bytes_opt = multibase::decode_ed25519_key(&vm_public_key_multibase);
        assert!(option::is_some(&pk_bytes_opt), ErrorInvalidPublicKeyMultibaseFormat);
        let pk_bytes = option::destroy_some(pk_bytes_opt);

        let associated_account_signer = account::create_signer_with_account_cap(&mut did_document_data.account_cap);

        let max_inactive_interval_for_sk = session_key::max_inactive_interval();

        let app_name = string::utf8(b"did_authentication_key:");
        string::append(&mut app_name, vm_fragment);
        let app_url = format_did(&did_document_data.id);

        let associated_address = signer::address_of(&associated_account_signer);
        
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

        // Use the public function from session_key module to derive the auth key
        let auth_key_for_session = session_key::ed25519_public_key_to_authentication_key(&pk_bytes);

        session_key::create_session_key(
            &associated_account_signer,
            app_name,
            app_url,
            auth_key_for_session, // Use the derived auth_key from session_key module
            scopes_for_sk,
            max_inactive_interval_for_sk
        );
    }

    /// Validates did:key controllers according to NIP-1 requirements.
    /// If any controller is did:key, there must be exactly one such controller,
    /// and its public key must match initial_vm_pk_multibase.
    fun validate_did_key_controllers(
        controllers: &vector<DID>,
        initial_vm_pk_multibase: &String
    ) {
        let i = 0;
        let did_key_controller_count = 0;
        let did_key_controller_opt = option::none<DID>();

        while (i < vector::length(controllers)) {
            let controller = vector::borrow(controllers, i);
            if (controller.method == string::utf8(b"key")) {
                did_key_controller_count = did_key_controller_count + 1;
                // Store the first (and should be only) did:key controller found
                if (option::is_none(&did_key_controller_opt)) {
                    did_key_controller_opt = option::some(*controller);
                }
            };
            i = i + 1;
        };

        if (did_key_controller_count > 0) {
            // If there's any did:key controller, there must be exactly one.
            assert!(did_key_controller_count == 1, error::invalid_argument(ErrorMultipleDIDKeyControllersNotAllowed));
            
            assert!(option::is_some(&did_key_controller_opt), error::internal(0)); // Should be some if count is 1
            let did_key_controller = option::destroy_some(did_key_controller_opt);
            
            // For did:key, the identifier should be the multibase-encoded public key
            let identifier = &did_key_controller.identifier;
            let identifier_bytes = string::bytes(identifier);
            assert!(vector::length(identifier_bytes) > 0, error::invalid_argument(ErrorInvalidDIDStringFormat));
            
            let first_byte = *vector::borrow(identifier_bytes, 0);
            assert!(first_byte == 122, error::invalid_argument(ErrorInvalidDIDStringFormat)); // 'z' for base58btc
            
            let controller_pk_multibase = *identifier;
            
            let controller_pk_opt = multibase::decode_ed25519_key(&controller_pk_multibase);
            let initial_pk_opt = multibase::decode_ed25519_key(initial_vm_pk_multibase);
            
            assert!(option::is_some(&controller_pk_opt), error::invalid_argument(ErrorInvalidPublicKeyMultibaseFormat));
            assert!(option::is_some(&initial_pk_opt), error::invalid_argument(ErrorInvalidPublicKeyMultibaseFormat));
            
            let controller_pk_bytes = option::destroy_some(controller_pk_opt);
            let initial_pk_bytes = option::destroy_some(initial_pk_opt);
            
            assert!(controller_pk_bytes == initial_pk_bytes, error::invalid_argument(ErrorDIDKeyControllerPublicKeyMismatch));
        }
        // If did_key_controller_count is 0, no specific validation for did:key is needed here.
    }

    /// Add an Ed25519 verification method to the authentication relationship
    /// and automatically register it as a rooch session key.
    /// This function makes explicit the special property of Ed25519 authentication methods.
    fun add_ed25519_authentication_method(
        did_document_data: &mut DIDDocument,
        fragment: String,
        public_key_multibase: String
    ) {
        // 1. Add the verification method if it doesn't exist
        if (!simple_map::contains_key(&did_document_data.verification_methods, &fragment)) {
            let vm_id = VerificationMethodID {
                did: did_document_data.id,
                fragment: fragment,
            };
            
            let vm = VerificationMethod {
                id: vm_id,
                type: string::utf8(VERIFICATION_METHOD_TYPE_ED25519),
                controller: did_document_data.id,
                public_key_multibase,
            };
            
            simple_map::add(&mut did_document_data.verification_methods, fragment, vm);
        };
        
        // 2. Add to authentication relationship if not already present
        if (!vector::contains(&did_document_data.authentication, &fragment)) {
            vector::push_back(&mut did_document_data.authentication, fragment);
        };
        
        // 3. Register as rooch session key (special feature of Ed25519 authentication methods)
        internal_ensure_rooch_session_key(
            did_document_data,
            fragment,
            string::utf8(VERIFICATION_METHOD_TYPE_ED25519),
            public_key_multibase
        );
    }
} 
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
    use rooch_framework::multibase;

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

    // Verification relationship types
    const VERIFICATION_RELATIONSHIP_AUTHENTICATION: u8 = 0;
    const VERIFICATION_RELATIONSHIP_ASSERTION_METHOD: u8 = 1;
    const VERIFICATION_RELATIONSHIP_CAPABILITY_INVOCATION: u8 = 2;
    const VERIFICATION_RELATIONSHIP_CAPABILITY_DELEGATION: u8 = 3;
    const VERIFICATION_RELATIONSHIP_KEY_AGREEMENT: u8 = 4;

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
        expires: Option<u64>, // Optional expiration timestamp in seconds
    }

    /// Service definition
    struct Service has store, copy, drop {
        id: ServiceID,
        type: String,      // Type of service (e.g., "LLMGatewayNIP9", "CustodianServiceCADOP")
        service_endpoint: String, // URL or identifier for the service
        properties: SimpleMap<String, String>, // Additional service properties
    }

    /// DID Document containing all DID information. This is the data part of an Object.
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
        address_to_did: Table<address, DID>,
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
            address_to_did: table::new<address, DID>(),
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
        creator_account_signer: &signer,
        initial_controllers: vector<DID>,
        initial_vm_type: String,
        initial_vm_pk_multibase: String,
        initial_vm_fragment: String
    ): ObjectID {
        let registry = borrow_mut_did_registry();
        assert!(vector::length(&initial_controllers) > 0, error::invalid_argument(ErrorNoControllersSpecified));

        let new_account_cap = account::create_account_and_return_cap();
        let new_rooch_address = account::account_cap_address(&new_account_cap);
        
        let did_method_val = string::utf8(b"rooch");
        let did_identifier = address::to_bech32_string(new_rooch_address);
        assert!(!table::contains(&registry.address_to_did, new_rooch_address), error::already_exists(ErrorDIDAlreadyExists));
        
        let did = DID {
            method: did_method_val,
            identifier: did_identifier,
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
            expires: option::none<u64>(),
        };

        let verification_methods = simple_map::new<String, VerificationMethod>();
        simple_map::add(&mut verification_methods, initial_vm_fragment_value, initial_vm);

        let auth_rels_fragment = vector::singleton(initial_vm_fragment_value);

        let now = timestamp::now_seconds();

        let did_document_data = DIDDocument {
            id: did,
            controller: initial_controllers,
            verification_methods,
            authentication: auth_rels_fragment,
            assertion_method: auth_rels_fragment,
            capability_invocation: auth_rels_fragment,
            capability_delegation: auth_rels_fragment,
            key_agreement: auth_rels_fragment,
            services: simple_map::new<String, Service>(),
            account_cap: new_account_cap,
            also_known_as: vector::empty<String>(),
            created_timestamp: now,
            updated_timestamp: now,
        };

        let new_object_id = resolve_did_object_id(&did_identifier);
        let did_object = object::new_with_id(new_object_id, did_document_data);
        object::transfer_extend(did_object, new_rooch_address);
        
        new_object_id
    }

    public entry fun create_did_object_entry(
        creator_account_signer: &signer,
        initial_controller_did_methods: vector<String>,
        initial_controller_did_identifiers: vector<String>,
        initial_vm_type: String,
        initial_vm_pk_multibase: String,
        initial_vm_fragment: String
    ) {
        assert!(vector::length(&initial_controller_did_methods) == vector::length(&initial_controller_did_identifiers),
            error::invalid_argument(ErrorPropertyKeysValuesLengthMismatch));
        assert!(vector::length(&initial_controller_did_methods) > 0, error::invalid_argument(ErrorNoControllersSpecified));

        let initial_controllers = vector::empty<DID>();
        let i = 0;
        while (i < vector::length(&initial_controller_did_methods)) {
            vector::push_back(&mut initial_controllers, DID {
                method: *vector::borrow(&initial_controller_did_methods, i),
                identifier: *vector::borrow(&initial_controller_did_identifiers, i),
            });
            i = i + 1;
        };

        create_did_object(
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
        public_key_multibase: String,
        duration_to_expiry_seconds: u64
    ) {
        let object_id = resolve_did_object_id(&did_identifier_str);
        assert!(object::exists_object_with_type<DIDDocument>(object_id), error::not_found(ErrorDIDObjectNotFound));
        
        let did_doc_obj_ref_mut = object::borrow_mut_object_extend<DIDDocument>(object_id);
        let did_document_data = object::borrow_mut(did_doc_obj_ref_mut);

        assert_authorized_controller(&did_document_data.controller);

        assert!(!simple_map::contains_key(&did_document_data.verification_methods, &fragment),
            error::already_exists(ErrorVerificationMethodAlreadyExists));

        let expires_opt = if (duration_to_expiry_seconds > 0) {
            option::some(timestamp::now_seconds() + duration_to_expiry_seconds)
        } else {
            option::none()
        };

        let verification_method_id = VerificationMethodID {
            did: did_document_data.id,
            fragment: fragment,
        };

        let verification_method = VerificationMethod {
            id: verification_method_id,
            type: method_type,
            controller: did_document_data.id,
            public_key_multibase,
            expires: expires_opt,
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
            // Specific logic for AUTHENTICATION: ensure key type is Ed25519 and register as Rooch session key
            let vm = simple_map::borrow(&did_document_data.verification_methods, &fragment);
            assert!(vm.type == string::utf8(b"Ed25519VerificationKey2020"), ErrorUnsupportedAuthKeyTypeForSessionKey);
            
            // Attempt to register/ensure this key as a Rooch session key for the associated account
            internal_ensure_rooch_session_key(did_document_data, vm.type, vm.public_key_multibase, vm.expires);

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
        if (!object::exists_object_with_type<DIDRegistry>(did_registry_id())){
            return false
        };
        let registry = borrow_did_registry();
        table::contains(&registry.address_to_did, addr)
    }

    public fun get_did_for_address(addr: address): DID {
        assert!(exists_did_for_address(addr), error::not_found(ErrorDIDDocumentNotExist));
        let registry = borrow_did_registry();
        *table::borrow(&registry.address_to_did, addr)
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

        let verification_method = simple_map::borrow(&did_document_data.verification_methods, fragment);

        if (option::is_some(&verification_method.expires)) {
            let expires_timestamp_ref = option::borrow(&verification_method.expires);
            let now = timestamp::now_seconds();
            if (now > *expires_timestamp_ref) {
                return false
            };
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

    // This is a complex check potentially requiring resolving controller DIDs.
    fun assert_authorized_controller(controllers: &vector<DID>) {
        let _sender = tx_context::sender(); // Sender is now fetched here for auth logic
        // TODO: Implement full controller and capabilityDelegation check using _sender against controllers.
        // This is a CRITICAL security function.
        // For now, as a NON-SECURE placeholder, we might just check if the controller list isn't empty.
        // This placeholder DOES NOT provide any real security.
        // assert!(vector::length(controllers) > 0, error::permission_denied(ErrorControllerPermissionDenied));
        // Remove or replace the above assert with actual logic.
    }

    // New private helper function to register a VM as a Rooch session key
    fun internal_ensure_rooch_session_key(
        did_document_data: &mut DIDDocument,
        vm_type: String,
        vm_public_key_multibase: String,
        vm_expires: Option<u64>
    ) {
        // This assertion is a safeguard as the caller should already check this.
        assert!(vm_type == string::utf8(b"Ed25519VerificationKey2020"), ErrorUnsupportedAuthKeyTypeForSessionKey);

        // 1. Parse publicKeyMultibase to get raw Ed25519 public key bytes.
        let pk_bytes_opt = multibase::decode_ed25519_key(&vm_public_key_multibase);
        assert!(option::is_some(&pk_bytes_opt), ErrorInvalidPublicKeyMultibaseFormat);
        let _pk_bytes = option::destroy_some(pk_bytes_opt); // pk_bytes is now available

        // 2. Get AccountCap and create signer for the associated Rooch account.
        let _associated_account_signer = account::create_signer_with_account_cap(&mut did_document_data.account_cap);

        // 3. Determine expiration for the session key.
        let _session_key_expiration_timestamp_opt = vm_expires;

        // 4. Call the Rooch session key module to register the key.
        // THIS IS A PLACEHOLDER for the actual API call to `rooch_framework::session_key`.
        // The exact function name, parameters (especially for scopes like "all access" or specific module/function calls),
        // and return values need to be determined from the `session_key` module.
        // For example, it might be:
        // rooch_framework::session_key::register_ed25519_session_key(
        //     &associated_account_signer,
        //     pk_bytes,
        //     string::utf8(b"did_auth_key"), // Example application name
        //     option::none(), // Example: no specific app URL
        //     vector::empty(), // Example: all scopes or specific scope needed
        //     session_key_expiration_timestamp_opt
        // );
        // Using a simplified placeholder for now, assuming it returns a boolean for success:
        // let registration_successful = rooch_framework::session_key::placeholder_register_did_auth_key(
        //     &associated_account_signer,
        //     pk_bytes,
        //     session_key_expiration_timestamp_opt
        // );
        // assert!(registration_successful, ErrorSessionKeyRegistrationFailed);
        // TODO: Re-enable and implement session key registration when session_key module is ready.
    }
} 
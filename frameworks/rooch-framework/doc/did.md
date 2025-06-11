
<a name="0x3_did"></a>

# Module `0x3::did`



-  [Struct `DID`](#0x3_did_DID)
-  [Struct `VerificationMethodID`](#0x3_did_VerificationMethodID)
-  [Struct `ServiceID`](#0x3_did_ServiceID)
-  [Struct `VerificationMethod`](#0x3_did_VerificationMethod)
-  [Struct `Service`](#0x3_did_Service)
-  [Resource `DIDDocument`](#0x3_did_DIDDocument)
-  [Resource `DIDRegistry`](#0x3_did_DIDRegistry)
-  [Struct `DIDCreatedEvent`](#0x3_did_DIDCreatedEvent)
-  [Struct `VerificationMethodAddedEvent`](#0x3_did_VerificationMethodAddedEvent)
-  [Struct `VerificationMethodRemovedEvent`](#0x3_did_VerificationMethodRemovedEvent)
-  [Struct `VerificationRelationshipModifiedEvent`](#0x3_did_VerificationRelationshipModifiedEvent)
-  [Struct `ServiceAddedEvent`](#0x3_did_ServiceAddedEvent)
-  [Struct `ServiceUpdatedEvent`](#0x3_did_ServiceUpdatedEvent)
-  [Struct `ServiceRemovedEvent`](#0x3_did_ServiceRemovedEvent)
-  [Constants](#@Constants_0)
-  [Function `verification_relationship_authentication`](#0x3_did_verification_relationship_authentication)
-  [Function `verification_relationship_assertion_method`](#0x3_did_verification_relationship_assertion_method)
-  [Function `verification_relationship_capability_invocation`](#0x3_did_verification_relationship_capability_invocation)
-  [Function `verification_relationship_capability_delegation`](#0x3_did_verification_relationship_capability_delegation)
-  [Function `verification_relationship_key_agreement`](#0x3_did_verification_relationship_key_agreement)
-  [Function `genesis_init`](#0x3_did_genesis_init)
-  [Function `init_did_registry`](#0x3_did_init_did_registry)
-  [Function `create_did_object_for_self_entry`](#0x3_did_create_did_object_for_self_entry)
-  [Function `create_did_object_for_self`](#0x3_did_create_did_object_for_self)
-  [Function `create_did_object_via_cadop_with_did_key_entry`](#0x3_did_create_did_object_via_cadop_with_did_key_entry)
-  [Function `create_did_object_via_cadop_with_did_key`](#0x3_did_create_did_object_via_cadop_with_did_key)
-  [Function `add_verification_method_entry`](#0x3_did_add_verification_method_entry)
-  [Function `remove_verification_method_entry`](#0x3_did_remove_verification_method_entry)
-  [Function `add_to_verification_relationship_entry`](#0x3_did_add_to_verification_relationship_entry)
-  [Function `remove_from_verification_relationship_entry`](#0x3_did_remove_from_verification_relationship_entry)
-  [Function `add_service_entry`](#0x3_did_add_service_entry)
-  [Function `add_service_with_properties_entry`](#0x3_did_add_service_with_properties_entry)
-  [Function `update_service_entry`](#0x3_did_update_service_entry)
-  [Function `remove_service_entry`](#0x3_did_remove_service_entry)
-  [Function `exists_did_document_by_identifier`](#0x3_did_exists_did_document_by_identifier)
-  [Function `exists_did_for_address`](#0x3_did_exists_did_for_address)
-  [Function `get_dids_by_controller`](#0x3_did_get_dids_by_controller)
-  [Function `get_dids_by_controller_string`](#0x3_did_get_dids_by_controller_string)
-  [Function `has_verification_relationship_in_doc`](#0x3_did_has_verification_relationship_in_doc)
-  [Function `is_verification_method_valid_in_doc`](#0x3_did_is_verification_method_valid_in_doc)
-  [Function `format_did`](#0x3_did_format_did)
-  [Function `format_verification_method_id`](#0x3_did_format_verification_method_id)
-  [Function `format_service_id`](#0x3_did_format_service_id)
-  [Function `new_did_from_parts`](#0x3_did_new_did_from_parts)
-  [Function `new_rooch_did_by_address`](#0x3_did_new_rooch_did_by_address)
-  [Function `parse_did_string`](#0x3_did_parse_did_string)
-  [Function `get_did_identifier_string`](#0x3_did_get_did_identifier_string)
-  [Function `get_did_method`](#0x3_did_get_did_method)
-  [Function `get_did_document`](#0x3_did_get_did_document)
-  [Function `get_did_document_by_object_id`](#0x3_did_get_did_document_by_object_id)
-  [Function `get_did_identifier`](#0x3_did_get_did_identifier)
-  [Function `get_controllers`](#0x3_did_get_controllers)
-  [Function `get_verification_methods`](#0x3_did_get_verification_methods)
-  [Function `get_verification_method`](#0x3_did_get_verification_method)
-  [Function `get_verification_method_id`](#0x3_did_get_verification_method_id)
-  [Function `get_verification_method_type`](#0x3_did_get_verification_method_type)
-  [Function `get_verification_method_controller`](#0x3_did_get_verification_method_controller)
-  [Function `get_verification_method_public_key_multibase`](#0x3_did_get_verification_method_public_key_multibase)
-  [Function `get_authentication_methods`](#0x3_did_get_authentication_methods)
-  [Function `get_assertion_methods`](#0x3_did_get_assertion_methods)
-  [Function `get_capability_invocation_methods`](#0x3_did_get_capability_invocation_methods)
-  [Function `get_capability_delegation_methods`](#0x3_did_get_capability_delegation_methods)
-  [Function `get_key_agreement_methods`](#0x3_did_get_key_agreement_methods)
-  [Function `get_services`](#0x3_did_get_services)
-  [Function `get_service`](#0x3_did_get_service)
-  [Function `get_service_id`](#0x3_did_get_service_id)
-  [Function `get_service_type`](#0x3_did_get_service_type)
-  [Function `get_service_endpoint`](#0x3_did_get_service_endpoint)
-  [Function `get_service_properties`](#0x3_did_get_service_properties)
-  [Function `get_also_known_as`](#0x3_did_get_also_known_as)
-  [Function `get_created_timestamp_by_object_id`](#0x3_did_get_created_timestamp_by_object_id)
-  [Function `get_updated_timestamp_by_object_id`](#0x3_did_get_updated_timestamp_by_object_id)
-  [Function `get_created_timestamp`](#0x3_did_get_created_timestamp)
-  [Function `get_updated_timestamp`](#0x3_did_get_updated_timestamp)
-  [Function `get_did_created_timestamp`](#0x3_did_get_did_created_timestamp)
-  [Function `get_did_updated_timestamp`](#0x3_did_get_did_updated_timestamp)
-  [Function `get_did_address`](#0x3_did_get_did_address)
-  [Function `find_verification_method_by_session_key`](#0x3_did_find_verification_method_by_session_key)


<pre><code><b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="">0x1::signer</a>;
<b>use</b> <a href="">0x1::string</a>;
<b>use</b> <a href="">0x1::vector</a>;
<b>use</b> <a href="">0x2::account</a>;
<b>use</b> <a href="">0x2::address</a>;
<b>use</b> <a href="">0x2::event</a>;
<b>use</b> <a href="">0x2::multibase</a>;
<b>use</b> <a href="">0x2::object</a>;
<b>use</b> <a href="">0x2::simple_map</a>;
<b>use</b> <a href="">0x2::table</a>;
<b>use</b> <a href="auth_validator.md#0x3_auth_validator">0x3::auth_validator</a>;
<b>use</b> <a href="bitcoin_address.md#0x3_bitcoin_address">0x3::bitcoin_address</a>;
<b>use</b> <a href="session_key.md#0x3_session_key">0x3::session_key</a>;
</code></pre>



<a name="0x3_did_DID"></a>

## Struct `DID`

DID identifier type


<pre><code><b>struct</b> <a href="did.md#0x3_did_DID">DID</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x3_did_VerificationMethodID"></a>

## Struct `VerificationMethodID`

Verification method ID


<pre><code><b>struct</b> <a href="did.md#0x3_did_VerificationMethodID">VerificationMethodID</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x3_did_ServiceID"></a>

## Struct `ServiceID`

Service ID


<pre><code><b>struct</b> <a href="did.md#0x3_did_ServiceID">ServiceID</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x3_did_VerificationMethod"></a>

## Struct `VerificationMethod`

Verification method


<pre><code><b>struct</b> <a href="did.md#0x3_did_VerificationMethod">VerificationMethod</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x3_did_Service"></a>

## Struct `Service`

Service definition


<pre><code><b>struct</b> <a href="did.md#0x3_did_Service">Service</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x3_did_DIDDocument"></a>

## Resource `DIDDocument`

DID Document containing all DID information. This is the data part of an Object.
The DIDDocuemnt only has <code>key</code> ability, no <code>store</code>, so the user can not transfer it to other accounts.


<pre><code><b>struct</b> <a href="did.md#0x3_did_DIDDocument">DIDDocument</a> <b>has</b> key
</code></pre>



<a name="0x3_did_DIDRegistry"></a>

## Resource `DIDRegistry`

Registry to store mappings. This is a Named Object.


<pre><code><b>struct</b> <a href="did.md#0x3_did_DIDRegistry">DIDRegistry</a> <b>has</b> key
</code></pre>



<a name="0x3_did_DIDCreatedEvent"></a>

## Struct `DIDCreatedEvent`

Event emitted when a new DID document is created


<pre><code>#[<a href="">event</a>]
<b>struct</b> <a href="did.md#0x3_did_DIDCreatedEvent">DIDCreatedEvent</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x3_did_VerificationMethodAddedEvent"></a>

## Struct `VerificationMethodAddedEvent`

Event emitted when a verification method is added to a DID document


<pre><code>#[<a href="">event</a>]
<b>struct</b> <a href="did.md#0x3_did_VerificationMethodAddedEvent">VerificationMethodAddedEvent</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x3_did_VerificationMethodRemovedEvent"></a>

## Struct `VerificationMethodRemovedEvent`

Event emitted when a verification method is removed from a DID document


<pre><code>#[<a href="">event</a>]
<b>struct</b> <a href="did.md#0x3_did_VerificationMethodRemovedEvent">VerificationMethodRemovedEvent</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x3_did_VerificationRelationshipModifiedEvent"></a>

## Struct `VerificationRelationshipModifiedEvent`

Event emitted when a verification relationship is modified


<pre><code>#[<a href="">event</a>]
<b>struct</b> <a href="did.md#0x3_did_VerificationRelationshipModifiedEvent">VerificationRelationshipModifiedEvent</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x3_did_ServiceAddedEvent"></a>

## Struct `ServiceAddedEvent`

Event emitted when a service is added to a DID document


<pre><code>#[<a href="">event</a>]
<b>struct</b> <a href="did.md#0x3_did_ServiceAddedEvent">ServiceAddedEvent</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x3_did_ServiceUpdatedEvent"></a>

## Struct `ServiceUpdatedEvent`

Event emitted when a service is updated in a DID document


<pre><code>#[<a href="">event</a>]
<b>struct</b> <a href="did.md#0x3_did_ServiceUpdatedEvent">ServiceUpdatedEvent</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x3_did_ServiceRemovedEvent"></a>

## Struct `ServiceRemovedEvent`

Event emitted when a service is removed from a DID document


<pre><code>#[<a href="">event</a>]
<b>struct</b> <a href="did.md#0x3_did_ServiceRemovedEvent">ServiceRemovedEvent</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x3_did_ErrorInvalidSignature"></a>

Invalid signature (can be reused or made more specific)


<pre><code><b>const</b> <a href="did.md#0x3_did_ErrorInvalidSignature">ErrorInvalidSignature</a>: u64 = 11;
</code></pre>



<a name="0x3_did_ErrorAccountCapNotFound"></a>

Associated AccountCap not found in DIDDocument when expected


<pre><code><b>const</b> <a href="did.md#0x3_did_ErrorAccountCapNotFound">ErrorAccountCapNotFound</a>: u64 = 14;
</code></pre>



<a name="0x3_did_ErrorControllerPermissionDenied"></a>

Permission denied based on controller check


<pre><code><b>const</b> <a href="did.md#0x3_did_ErrorControllerPermissionDenied">ErrorControllerPermissionDenied</a>: u64 = 15;
</code></pre>



<a name="0x3_did_ErrorCustodianDIDNotFound"></a>

Custodian DID document does not exist


<pre><code><b>const</b> <a href="did.md#0x3_did_ErrorCustodianDIDNotFound">ErrorCustodianDIDNotFound</a>: u64 = 30;
</code></pre>



<a name="0x3_did_ErrorCustodianDoesNotHaveCADOPService"></a>

Custodian does not have CADOP service


<pre><code><b>const</b> <a href="did.md#0x3_did_ErrorCustodianDoesNotHaveCADOPService">ErrorCustodianDoesNotHaveCADOPService</a>: u64 = 29;
</code></pre>



<a name="0x3_did_ErrorDIDAlreadyExists"></a>

DID already exists (e.g., identifier already registered)


<pre><code><b>const</b> <a href="did.md#0x3_did_ErrorDIDAlreadyExists">ErrorDIDAlreadyExists</a>: u64 = 2;
</code></pre>



<a name="0x3_did_ErrorDIDDocumentNotExist"></a>

DID document does not exist (legacy or general not found)


<pre><code><b>const</b> <a href="did.md#0x3_did_ErrorDIDDocumentNotExist">ErrorDIDDocumentNotExist</a>: u64 = 1;
</code></pre>



<a name="0x3_did_ErrorDIDKeyControllerPublicKeyMismatch"></a>

For did:key controllers, the initial verification method public key must match the key in the DID identifier


<pre><code><b>const</b> <a href="did.md#0x3_did_ErrorDIDKeyControllerPublicKeyMismatch">ErrorDIDKeyControllerPublicKeyMismatch</a>: u64 = 23;
</code></pre>



<a name="0x3_did_ErrorDIDObjectNotFound"></a>

DID Object not found for the given identifier


<pre><code><b>const</b> <a href="did.md#0x3_did_ErrorDIDObjectNotFound">ErrorDIDObjectNotFound</a>: u64 = 13;
</code></pre>



<a name="0x3_did_ErrorDIDRegistryAlreadyInitialized"></a>

DIDRegistry is already initialized


<pre><code><b>const</b> <a href="did.md#0x3_did_ErrorDIDRegistryAlreadyInitialized">ErrorDIDRegistryAlreadyInitialized</a>: u64 = 12;
</code></pre>



<a name="0x3_did_ErrorInsufficientPermission"></a>

Verification method has insufficient permission for the requested operation


<pre><code><b>const</b> <a href="did.md#0x3_did_ErrorInsufficientPermission">ErrorInsufficientPermission</a>: u64 = 26;
</code></pre>



<a name="0x3_did_ErrorInvalidArgument"></a>

Generic invalid argument


<pre><code><b>const</b> <a href="did.md#0x3_did_ErrorInvalidArgument">ErrorInvalidArgument</a>: u64 = 17;
</code></pre>



<a name="0x3_did_ErrorInvalidDIDStringFormat"></a>

Invalid DID string format (should be "did:method:identifier")


<pre><code><b>const</b> <a href="did.md#0x3_did_ErrorInvalidDIDStringFormat">ErrorInvalidDIDStringFormat</a>: u64 = 22;
</code></pre>



<a name="0x3_did_ErrorInvalidPublicKeyMultibaseFormat"></a>

The format of the publicKeyMultibase string is invalid or cannot be parsed


<pre><code><b>const</b> <a href="did.md#0x3_did_ErrorInvalidPublicKeyMultibaseFormat">ErrorInvalidPublicKeyMultibaseFormat</a>: u64 = 20;
</code></pre>



<a name="0x3_did_ErrorInvalidVerificationRelationship"></a>

Invalid verification relationship


<pre><code><b>const</b> <a href="did.md#0x3_did_ErrorInvalidVerificationRelationship">ErrorInvalidVerificationRelationship</a>: u64 = 9;
</code></pre>



<a name="0x3_did_ErrorMultipleDIDKeyControllersNotAllowed"></a>

Multiple did:key controllers are not allowed during initial DID creation with a did:key controller


<pre><code><b>const</b> <a href="did.md#0x3_did_ErrorMultipleDIDKeyControllersNotAllowed">ErrorMultipleDIDKeyControllersNotAllowed</a>: u64 = 24;
</code></pre>



<a name="0x3_did_ErrorNoControllersSpecified"></a>

No controllers specified during DID creation or update


<pre><code><b>const</b> <a href="did.md#0x3_did_ErrorNoControllersSpecified">ErrorNoControllersSpecified</a>: u64 = 18;
</code></pre>



<a name="0x3_did_ErrorNoSessionKeyInContext"></a>

No session key found in transaction context - all DID operations must use session keys


<pre><code><b>const</b> <a href="did.md#0x3_did_ErrorNoSessionKeyInContext">ErrorNoSessionKeyInContext</a>: u64 = 28;
</code></pre>



<a name="0x3_did_ErrorPropertyKeysValuesLengthMismatch"></a>

Mismatch in length between property keys and values for a service


<pre><code><b>const</b> <a href="did.md#0x3_did_ErrorPropertyKeysValuesLengthMismatch">ErrorPropertyKeysValuesLengthMismatch</a>: u64 = 16;
</code></pre>



<a name="0x3_did_ErrorServiceAlreadyExists"></a>

Service already exists


<pre><code><b>const</b> <a href="did.md#0x3_did_ErrorServiceAlreadyExists">ErrorServiceAlreadyExists</a>: u64 = 7;
</code></pre>



<a name="0x3_did_ErrorServiceNotFound"></a>

Service not found


<pre><code><b>const</b> <a href="did.md#0x3_did_ErrorServiceNotFound">ErrorServiceNotFound</a>: u64 = 6;
</code></pre>



<a name="0x3_did_ErrorSessionKeyNotFound"></a>

Session key not found in DID document's authentication methods


<pre><code><b>const</b> <a href="did.md#0x3_did_ErrorSessionKeyNotFound">ErrorSessionKeyNotFound</a>: u64 = 25;
</code></pre>



<a name="0x3_did_ErrorSessionKeyRegistrationFailed"></a>

Failed to register key with the Rooch session key module


<pre><code><b>const</b> <a href="did.md#0x3_did_ErrorSessionKeyRegistrationFailed">ErrorSessionKeyRegistrationFailed</a>: u64 = 21;
</code></pre>



<a name="0x3_did_ErrorSignerNotDIDAccount"></a>

The signer is not the DID's associated account


<pre><code><b>const</b> <a href="did.md#0x3_did_ErrorSignerNotDIDAccount">ErrorSignerNotDIDAccount</a>: u64 = 27;
</code></pre>



<a name="0x3_did_ErrorUnauthorized"></a>

Unauthorized operation (generic, consider specific ErrorControllerPermissionDenied)


<pre><code><b>const</b> <a href="did.md#0x3_did_ErrorUnauthorized">ErrorUnauthorized</a>: u64 = 3;
</code></pre>



<a name="0x3_did_ErrorUnsupportedAuthKeyTypeForSessionKey"></a>

Verification method type is not supported for Rooch session key linkage (e.g., not Ed25519)


<pre><code><b>const</b> <a href="did.md#0x3_did_ErrorUnsupportedAuthKeyTypeForSessionKey">ErrorUnsupportedAuthKeyTypeForSessionKey</a>: u64 = 19;
</code></pre>



<a name="0x3_did_ErrorVerificationMethodAlreadyExists"></a>

Verification method already exists


<pre><code><b>const</b> <a href="did.md#0x3_did_ErrorVerificationMethodAlreadyExists">ErrorVerificationMethodAlreadyExists</a>: u64 = 5;
</code></pre>



<a name="0x3_did_ErrorVerificationMethodExpired"></a>

Verification method has expired


<pre><code><b>const</b> <a href="did.md#0x3_did_ErrorVerificationMethodExpired">ErrorVerificationMethodExpired</a>: u64 = 8;
</code></pre>



<a name="0x3_did_ErrorVerificationMethodNotFound"></a>

Verification method not found


<pre><code><b>const</b> <a href="did.md#0x3_did_ErrorVerificationMethodNotFound">ErrorVerificationMethodNotFound</a>: u64 = 4;
</code></pre>



<a name="0x3_did_ErrorVerificationMethodNotInRelationship"></a>

Verification method not in the relationship


<pre><code><b>const</b> <a href="did.md#0x3_did_ErrorVerificationMethodNotInRelationship">ErrorVerificationMethodNotInRelationship</a>: u64 = 10;
</code></pre>



<a name="0x3_did_VERIFICATION_METHOD_TYPE_ED25519"></a>



<pre><code><b>const</b> <a href="did.md#0x3_did_VERIFICATION_METHOD_TYPE_ED25519">VERIFICATION_METHOD_TYPE_ED25519</a>: <a href="">vector</a>&lt;u8&gt; = [69, 100, 50, 53, 53, 49, 57, 86, 101, 114, 105, 102, 105, 99, 97, 116, 105, 111, 110, 75, 101, 121, 50, 48, 50, 48];
</code></pre>



<a name="0x3_did_VERIFICATION_METHOD_TYPE_SECP256K1"></a>



<pre><code><b>const</b> <a href="did.md#0x3_did_VERIFICATION_METHOD_TYPE_SECP256K1">VERIFICATION_METHOD_TYPE_SECP256K1</a>: <a href="">vector</a>&lt;u8&gt; = [69, 99, 100, 115, 97, 83, 101, 99, 112, 50, 53, 54, 107, 49, 86, 101, 114, 105, 102, 105, 99, 97, 116, 105, 111, 110, 75, 101, 121, 50, 48, 49, 57];
</code></pre>



<a name="0x3_did_VERIFICATION_METHOD_TYPE_SECP256R1"></a>



<pre><code><b>const</b> <a href="did.md#0x3_did_VERIFICATION_METHOD_TYPE_SECP256R1">VERIFICATION_METHOD_TYPE_SECP256R1</a>: <a href="">vector</a>&lt;u8&gt; = [69, 99, 100, 115, 97, 83, 101, 99, 112, 50, 53, 54, 114, 49, 86, 101, 114, 105, 102, 105, 99, 97, 116, 105, 111, 110, 75, 101, 121, 50, 48, 49, 57];
</code></pre>



<a name="0x3_did_VERIFICATION_RELATIONSHIP_ASSERTION_METHOD"></a>



<pre><code><b>const</b> <a href="did.md#0x3_did_VERIFICATION_RELATIONSHIP_ASSERTION_METHOD">VERIFICATION_RELATIONSHIP_ASSERTION_METHOD</a>: u8 = 1;
</code></pre>



<a name="0x3_did_VERIFICATION_RELATIONSHIP_AUTHENTICATION"></a>



<pre><code><b>const</b> <a href="did.md#0x3_did_VERIFICATION_RELATIONSHIP_AUTHENTICATION">VERIFICATION_RELATIONSHIP_AUTHENTICATION</a>: u8 = 0;
</code></pre>



<a name="0x3_did_VERIFICATION_RELATIONSHIP_CAPABILITY_DELEGATION"></a>



<pre><code><b>const</b> <a href="did.md#0x3_did_VERIFICATION_RELATIONSHIP_CAPABILITY_DELEGATION">VERIFICATION_RELATIONSHIP_CAPABILITY_DELEGATION</a>: u8 = 3;
</code></pre>



<a name="0x3_did_VERIFICATION_RELATIONSHIP_CAPABILITY_INVOCATION"></a>



<pre><code><b>const</b> <a href="did.md#0x3_did_VERIFICATION_RELATIONSHIP_CAPABILITY_INVOCATION">VERIFICATION_RELATIONSHIP_CAPABILITY_INVOCATION</a>: u8 = 2;
</code></pre>



<a name="0x3_did_VERIFICATION_RELATIONSHIP_KEY_AGREEMENT"></a>



<pre><code><b>const</b> <a href="did.md#0x3_did_VERIFICATION_RELATIONSHIP_KEY_AGREEMENT">VERIFICATION_RELATIONSHIP_KEY_AGREEMENT</a>: u8 = 4;
</code></pre>



<a name="0x3_did_verification_relationship_authentication"></a>

## Function `verification_relationship_authentication`

Get verification relationship constant for authentication


<pre><code><b>public</b> <b>fun</b> <a href="did.md#0x3_did_verification_relationship_authentication">verification_relationship_authentication</a>(): u8
</code></pre>



<a name="0x3_did_verification_relationship_assertion_method"></a>

## Function `verification_relationship_assertion_method`

Get verification relationship constant for assertion method


<pre><code><b>public</b> <b>fun</b> <a href="did.md#0x3_did_verification_relationship_assertion_method">verification_relationship_assertion_method</a>(): u8
</code></pre>



<a name="0x3_did_verification_relationship_capability_invocation"></a>

## Function `verification_relationship_capability_invocation`

Get verification relationship constant for capability invocation


<pre><code><b>public</b> <b>fun</b> <a href="did.md#0x3_did_verification_relationship_capability_invocation">verification_relationship_capability_invocation</a>(): u8
</code></pre>



<a name="0x3_did_verification_relationship_capability_delegation"></a>

## Function `verification_relationship_capability_delegation`

Get verification relationship constant for capability delegation


<pre><code><b>public</b> <b>fun</b> <a href="did.md#0x3_did_verification_relationship_capability_delegation">verification_relationship_capability_delegation</a>(): u8
</code></pre>



<a name="0x3_did_verification_relationship_key_agreement"></a>

## Function `verification_relationship_key_agreement`

Get verification relationship constant for key agreement


<pre><code><b>public</b> <b>fun</b> <a href="did.md#0x3_did_verification_relationship_key_agreement">verification_relationship_key_agreement</a>(): u8
</code></pre>



<a name="0x3_did_genesis_init"></a>

## Function `genesis_init`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="did.md#0x3_did_genesis_init">genesis_init</a>()
</code></pre>



<a name="0x3_did_init_did_registry"></a>

## Function `init_did_registry`

Initialize the DID system
Any account can call this function to initialize the DID system


<pre><code><b>public</b> entry <b>fun</b> <a href="did.md#0x3_did_init_did_registry">init_did_registry</a>()
</code></pre>



<a name="0x3_did_create_did_object_for_self_entry"></a>

## Function `create_did_object_for_self_entry`

Create a DID for oneself using account key only.
This function validates that the provided public key corresponds to the creator's account.
Currently only supports Secp256k1 keys.


<pre><code><b>public</b> entry <b>fun</b> <a href="did.md#0x3_did_create_did_object_for_self_entry">create_did_object_for_self_entry</a>(creator_account_signer: &<a href="">signer</a>, account_public_key_multibase: <a href="_String">string::String</a>)
</code></pre>



<a name="0x3_did_create_did_object_for_self"></a>

## Function `create_did_object_for_self`

Internal function for self DID creation.
Validates that the provided public key matches the creator's account address.


<pre><code><b>public</b> <b>fun</b> <a href="did.md#0x3_did_create_did_object_for_self">create_did_object_for_self</a>(creator_account_signer: &<a href="">signer</a>, account_public_key_multibase: <a href="_String">string::String</a>): <a href="_ObjectID">object::ObjectID</a>
</code></pre>



<a name="0x3_did_create_did_object_via_cadop_with_did_key_entry"></a>

## Function `create_did_object_via_cadop_with_did_key_entry`

Create a DID via CADOP (Custodian-Assisted DID Onboarding Protocol) using did:key.
The custodian assists in DID creation but the user retains control.
Each user gets a unique service key from the custodian.
The user's public key is extracted from their did:key string.


<pre><code><b>public</b> entry <b>fun</b> <a href="did.md#0x3_did_create_did_object_via_cadop_with_did_key_entry">create_did_object_via_cadop_with_did_key_entry</a>(custodian_signer: &<a href="">signer</a>, user_did_key_string: <a href="_String">string::String</a>, custodian_service_pk_multibase: <a href="_String">string::String</a>, custodian_service_vm_type: <a href="_String">string::String</a>)
</code></pre>



<a name="0x3_did_create_did_object_via_cadop_with_did_key"></a>

## Function `create_did_object_via_cadop_with_did_key`

Internal function for CADOP DID creation with did:key.
Returns the ObjectID of the created DID document for testing and verification.


<pre><code><b>public</b> <b>fun</b> <a href="did.md#0x3_did_create_did_object_via_cadop_with_did_key">create_did_object_via_cadop_with_did_key</a>(custodian_signer: &<a href="">signer</a>, user_did_key_string: <a href="_String">string::String</a>, custodian_service_pk_multibase: <a href="_String">string::String</a>, custodian_service_vm_type: <a href="_String">string::String</a>): <a href="_ObjectID">object::ObjectID</a>
</code></pre>



<a name="0x3_did_add_verification_method_entry"></a>

## Function `add_verification_method_entry`



<pre><code><b>public</b> entry <b>fun</b> <a href="did.md#0x3_did_add_verification_method_entry">add_verification_method_entry</a>(did_signer: &<a href="">signer</a>, fragment: <a href="_String">string::String</a>, method_type: <a href="_String">string::String</a>, public_key_multibase: <a href="_String">string::String</a>, verification_relationships: <a href="">vector</a>&lt;u8&gt;)
</code></pre>



<a name="0x3_did_remove_verification_method_entry"></a>

## Function `remove_verification_method_entry`



<pre><code><b>public</b> entry <b>fun</b> <a href="did.md#0x3_did_remove_verification_method_entry">remove_verification_method_entry</a>(did_signer: &<a href="">signer</a>, fragment: <a href="_String">string::String</a>)
</code></pre>



<a name="0x3_did_add_to_verification_relationship_entry"></a>

## Function `add_to_verification_relationship_entry`



<pre><code><b>public</b> entry <b>fun</b> <a href="did.md#0x3_did_add_to_verification_relationship_entry">add_to_verification_relationship_entry</a>(did_signer: &<a href="">signer</a>, fragment: <a href="_String">string::String</a>, relationship_type: u8)
</code></pre>



<a name="0x3_did_remove_from_verification_relationship_entry"></a>

## Function `remove_from_verification_relationship_entry`



<pre><code><b>public</b> entry <b>fun</b> <a href="did.md#0x3_did_remove_from_verification_relationship_entry">remove_from_verification_relationship_entry</a>(did_signer: &<a href="">signer</a>, fragment: <a href="_String">string::String</a>, relationship_type: u8)
</code></pre>



<a name="0x3_did_add_service_entry"></a>

## Function `add_service_entry`



<pre><code><b>public</b> entry <b>fun</b> <a href="did.md#0x3_did_add_service_entry">add_service_entry</a>(did_signer: &<a href="">signer</a>, fragment: <a href="_String">string::String</a>, service_type: <a href="_String">string::String</a>, service_endpoint: <a href="_String">string::String</a>)
</code></pre>



<a name="0x3_did_add_service_with_properties_entry"></a>

## Function `add_service_with_properties_entry`



<pre><code><b>public</b> entry <b>fun</b> <a href="did.md#0x3_did_add_service_with_properties_entry">add_service_with_properties_entry</a>(did_signer: &<a href="">signer</a>, fragment: <a href="_String">string::String</a>, service_type: <a href="_String">string::String</a>, service_endpoint: <a href="_String">string::String</a>, property_keys: <a href="">vector</a>&lt;<a href="_String">string::String</a>&gt;, property_values: <a href="">vector</a>&lt;<a href="_String">string::String</a>&gt;)
</code></pre>



<a name="0x3_did_update_service_entry"></a>

## Function `update_service_entry`



<pre><code><b>public</b> entry <b>fun</b> <a href="did.md#0x3_did_update_service_entry">update_service_entry</a>(did_signer: &<a href="">signer</a>, fragment: <a href="_String">string::String</a>, new_service_type: <a href="_String">string::String</a>, new_service_endpoint: <a href="_String">string::String</a>, new_property_keys: <a href="">vector</a>&lt;<a href="_String">string::String</a>&gt;, new_property_values: <a href="">vector</a>&lt;<a href="_String">string::String</a>&gt;)
</code></pre>



<a name="0x3_did_remove_service_entry"></a>

## Function `remove_service_entry`



<pre><code><b>public</b> entry <b>fun</b> <a href="did.md#0x3_did_remove_service_entry">remove_service_entry</a>(did_signer: &<a href="">signer</a>, fragment: <a href="_String">string::String</a>)
</code></pre>



<a name="0x3_did_exists_did_document_by_identifier"></a>

## Function `exists_did_document_by_identifier`



<pre><code><b>public</b> <b>fun</b> <a href="did.md#0x3_did_exists_did_document_by_identifier">exists_did_document_by_identifier</a>(identifier_str: <a href="_String">string::String</a>): bool
</code></pre>



<a name="0x3_did_exists_did_for_address"></a>

## Function `exists_did_for_address`



<pre><code><b>public</b> <b>fun</b> <a href="did.md#0x3_did_exists_did_for_address">exists_did_for_address</a>(addr: <b>address</b>): bool
</code></pre>



<a name="0x3_did_get_dids_by_controller"></a>

## Function `get_dids_by_controller`

Get all DID ObjectIDs controlled by a specific controller DID


<pre><code><b>public</b> <b>fun</b> <a href="did.md#0x3_did_get_dids_by_controller">get_dids_by_controller</a>(controller_did: <a href="did.md#0x3_did_DID">did::DID</a>): <a href="">vector</a>&lt;<a href="_ObjectID">object::ObjectID</a>&gt;
</code></pre>



<a name="0x3_did_get_dids_by_controller_string"></a>

## Function `get_dids_by_controller_string`



<pre><code><b>public</b> <b>fun</b> <a href="did.md#0x3_did_get_dids_by_controller_string">get_dids_by_controller_string</a>(controller_did_str: <a href="_String">string::String</a>): <a href="">vector</a>&lt;<a href="_ObjectID">object::ObjectID</a>&gt;
</code></pre>



<a name="0x3_did_has_verification_relationship_in_doc"></a>

## Function `has_verification_relationship_in_doc`



<pre><code><b>public</b> <b>fun</b> <a href="did.md#0x3_did_has_verification_relationship_in_doc">has_verification_relationship_in_doc</a>(did_document_data: &<a href="did.md#0x3_did_DIDDocument">did::DIDDocument</a>, fragment: &<a href="_String">string::String</a>, relationship_type: u8): bool
</code></pre>



<a name="0x3_did_is_verification_method_valid_in_doc"></a>

## Function `is_verification_method_valid_in_doc`



<pre><code><b>public</b> <b>fun</b> <a href="did.md#0x3_did_is_verification_method_valid_in_doc">is_verification_method_valid_in_doc</a>(did_document_data: &<a href="did.md#0x3_did_DIDDocument">did::DIDDocument</a>, fragment: &<a href="_String">string::String</a>): bool
</code></pre>



<a name="0x3_did_format_did"></a>

## Function `format_did`



<pre><code><b>public</b> <b>fun</b> <a href="did.md#0x3_did_format_did">format_did</a>(<a href="did.md#0x3_did">did</a>: &<a href="did.md#0x3_did_DID">did::DID</a>): <a href="_String">string::String</a>
</code></pre>



<a name="0x3_did_format_verification_method_id"></a>

## Function `format_verification_method_id`



<pre><code><b>public</b> <b>fun</b> <a href="did.md#0x3_did_format_verification_method_id">format_verification_method_id</a>(id: &<a href="did.md#0x3_did_VerificationMethodID">did::VerificationMethodID</a>): <a href="_String">string::String</a>
</code></pre>



<a name="0x3_did_format_service_id"></a>

## Function `format_service_id`



<pre><code><b>public</b> <b>fun</b> <a href="did.md#0x3_did_format_service_id">format_service_id</a>(id: &<a href="did.md#0x3_did_ServiceID">did::ServiceID</a>): <a href="_String">string::String</a>
</code></pre>



<a name="0x3_did_new_did_from_parts"></a>

## Function `new_did_from_parts`

Create a DID struct from method and identifier parts
This function only constructs a DID struct, it does NOT create a DID object on-chain


<pre><code><b>public</b> <b>fun</b> <a href="did.md#0x3_did_new_did_from_parts">new_did_from_parts</a>(method: <a href="_String">string::String</a>, identifier: <a href="_String">string::String</a>): <a href="did.md#0x3_did_DID">did::DID</a>
</code></pre>



<a name="0x3_did_new_rooch_did_by_address"></a>

## Function `new_rooch_did_by_address`

Create a Rooch DID struct from an address
This function only constructs a DID struct, it does NOT create a DID object on-chain


<pre><code><b>public</b> <b>fun</b> <a href="did.md#0x3_did_new_rooch_did_by_address">new_rooch_did_by_address</a>(addr: <b>address</b>): <a href="did.md#0x3_did_DID">did::DID</a>
</code></pre>



<a name="0x3_did_parse_did_string"></a>

## Function `parse_did_string`

Parse a DID string in the format "did:method:identifier" into a DID struct


<pre><code><b>public</b> <b>fun</b> <a href="did.md#0x3_did_parse_did_string">parse_did_string</a>(did_string: &<a href="_String">string::String</a>): <a href="did.md#0x3_did_DID">did::DID</a>
</code></pre>



<a name="0x3_did_get_did_identifier_string"></a>

## Function `get_did_identifier_string`

Get the identifier from a DID


<pre><code><b>public</b> <b>fun</b> <a href="did.md#0x3_did_get_did_identifier_string">get_did_identifier_string</a>(<a href="did.md#0x3_did">did</a>: &<a href="did.md#0x3_did_DID">did::DID</a>): <a href="_String">string::String</a>
</code></pre>



<a name="0x3_did_get_did_method"></a>

## Function `get_did_method`

Get the method from a DID


<pre><code><b>public</b> <b>fun</b> <a href="did.md#0x3_did_get_did_method">get_did_method</a>(<a href="did.md#0x3_did">did</a>: &<a href="did.md#0x3_did_DID">did::DID</a>): <a href="_String">string::String</a>
</code></pre>



<a name="0x3_did_get_did_document"></a>

## Function `get_did_document`

Get DIDDocument by address


<pre><code><b>public</b> <b>fun</b> <a href="did.md#0x3_did_get_did_document">get_did_document</a>(addr: <b>address</b>): &<a href="did.md#0x3_did_DIDDocument">did::DIDDocument</a>
</code></pre>



<a name="0x3_did_get_did_document_by_object_id"></a>

## Function `get_did_document_by_object_id`

Get DIDDocument by ObjectID


<pre><code><b>public</b> <b>fun</b> <a href="did.md#0x3_did_get_did_document_by_object_id">get_did_document_by_object_id</a>(object_id: <a href="_ObjectID">object::ObjectID</a>): &<a href="did.md#0x3_did_DIDDocument">did::DIDDocument</a>
</code></pre>



<a name="0x3_did_get_did_identifier"></a>

## Function `get_did_identifier`

Get DID identifier from DIDDocument


<pre><code><b>public</b> <b>fun</b> <a href="did.md#0x3_did_get_did_identifier">get_did_identifier</a>(did_doc: &<a href="did.md#0x3_did_DIDDocument">did::DIDDocument</a>): &<a href="did.md#0x3_did_DID">did::DID</a>
</code></pre>



<a name="0x3_did_get_controllers"></a>

## Function `get_controllers`

Get controllers from DIDDocument


<pre><code><b>public</b> <b>fun</b> <a href="did.md#0x3_did_get_controllers">get_controllers</a>(did_doc: &<a href="did.md#0x3_did_DIDDocument">did::DIDDocument</a>): &<a href="">vector</a>&lt;<a href="did.md#0x3_did_DID">did::DID</a>&gt;
</code></pre>



<a name="0x3_did_get_verification_methods"></a>

## Function `get_verification_methods`

Get verification methods from DIDDocument


<pre><code><b>public</b> <b>fun</b> <a href="did.md#0x3_did_get_verification_methods">get_verification_methods</a>(did_doc: &<a href="did.md#0x3_did_DIDDocument">did::DIDDocument</a>): &<a href="_SimpleMap">simple_map::SimpleMap</a>&lt;<a href="_String">string::String</a>, <a href="did.md#0x3_did_VerificationMethod">did::VerificationMethod</a>&gt;
</code></pre>



<a name="0x3_did_get_verification_method"></a>

## Function `get_verification_method`

Get verification method by fragment


<pre><code><b>public</b> <b>fun</b> <a href="did.md#0x3_did_get_verification_method">get_verification_method</a>(did_doc: &<a href="did.md#0x3_did_DIDDocument">did::DIDDocument</a>, fragment: &<a href="_String">string::String</a>): <a href="_Option">option::Option</a>&lt;<a href="did.md#0x3_did_VerificationMethod">did::VerificationMethod</a>&gt;
</code></pre>



<a name="0x3_did_get_verification_method_id"></a>

## Function `get_verification_method_id`



<pre><code><b>public</b> <b>fun</b> <a href="did.md#0x3_did_get_verification_method_id">get_verification_method_id</a>(vm: &<a href="did.md#0x3_did_VerificationMethod">did::VerificationMethod</a>): &<a href="did.md#0x3_did_VerificationMethodID">did::VerificationMethodID</a>
</code></pre>



<a name="0x3_did_get_verification_method_type"></a>

## Function `get_verification_method_type`



<pre><code><b>public</b> <b>fun</b> <a href="did.md#0x3_did_get_verification_method_type">get_verification_method_type</a>(vm: &<a href="did.md#0x3_did_VerificationMethod">did::VerificationMethod</a>): &<a href="_String">string::String</a>
</code></pre>



<a name="0x3_did_get_verification_method_controller"></a>

## Function `get_verification_method_controller`



<pre><code><b>public</b> <b>fun</b> <a href="did.md#0x3_did_get_verification_method_controller">get_verification_method_controller</a>(vm: &<a href="did.md#0x3_did_VerificationMethod">did::VerificationMethod</a>): &<a href="did.md#0x3_did_DID">did::DID</a>
</code></pre>



<a name="0x3_did_get_verification_method_public_key_multibase"></a>

## Function `get_verification_method_public_key_multibase`



<pre><code><b>public</b> <b>fun</b> <a href="did.md#0x3_did_get_verification_method_public_key_multibase">get_verification_method_public_key_multibase</a>(vm: &<a href="did.md#0x3_did_VerificationMethod">did::VerificationMethod</a>): &<a href="_String">string::String</a>
</code></pre>



<a name="0x3_did_get_authentication_methods"></a>

## Function `get_authentication_methods`

Get authentication methods from DIDDocument


<pre><code><b>public</b> <b>fun</b> <a href="did.md#0x3_did_get_authentication_methods">get_authentication_methods</a>(did_doc: &<a href="did.md#0x3_did_DIDDocument">did::DIDDocument</a>): &<a href="">vector</a>&lt;<a href="_String">string::String</a>&gt;
</code></pre>



<a name="0x3_did_get_assertion_methods"></a>

## Function `get_assertion_methods`

Get assertion methods from DIDDocument


<pre><code><b>public</b> <b>fun</b> <a href="did.md#0x3_did_get_assertion_methods">get_assertion_methods</a>(did_doc: &<a href="did.md#0x3_did_DIDDocument">did::DIDDocument</a>): &<a href="">vector</a>&lt;<a href="_String">string::String</a>&gt;
</code></pre>



<a name="0x3_did_get_capability_invocation_methods"></a>

## Function `get_capability_invocation_methods`

Get capability invocation methods from DIDDocument


<pre><code><b>public</b> <b>fun</b> <a href="did.md#0x3_did_get_capability_invocation_methods">get_capability_invocation_methods</a>(did_doc: &<a href="did.md#0x3_did_DIDDocument">did::DIDDocument</a>): &<a href="">vector</a>&lt;<a href="_String">string::String</a>&gt;
</code></pre>



<a name="0x3_did_get_capability_delegation_methods"></a>

## Function `get_capability_delegation_methods`

Get capability delegation methods from DIDDocument


<pre><code><b>public</b> <b>fun</b> <a href="did.md#0x3_did_get_capability_delegation_methods">get_capability_delegation_methods</a>(did_doc: &<a href="did.md#0x3_did_DIDDocument">did::DIDDocument</a>): &<a href="">vector</a>&lt;<a href="_String">string::String</a>&gt;
</code></pre>



<a name="0x3_did_get_key_agreement_methods"></a>

## Function `get_key_agreement_methods`

Get key agreement methods from DIDDocument


<pre><code><b>public</b> <b>fun</b> <a href="did.md#0x3_did_get_key_agreement_methods">get_key_agreement_methods</a>(did_doc: &<a href="did.md#0x3_did_DIDDocument">did::DIDDocument</a>): &<a href="">vector</a>&lt;<a href="_String">string::String</a>&gt;
</code></pre>



<a name="0x3_did_get_services"></a>

## Function `get_services`

Get services from DIDDocument


<pre><code><b>public</b> <b>fun</b> <a href="did.md#0x3_did_get_services">get_services</a>(did_doc: &<a href="did.md#0x3_did_DIDDocument">did::DIDDocument</a>): &<a href="_SimpleMap">simple_map::SimpleMap</a>&lt;<a href="_String">string::String</a>, <a href="did.md#0x3_did_Service">did::Service</a>&gt;
</code></pre>



<a name="0x3_did_get_service"></a>

## Function `get_service`

Get service by fragment


<pre><code><b>public</b> <b>fun</b> <a href="did.md#0x3_did_get_service">get_service</a>(did_doc: &<a href="did.md#0x3_did_DIDDocument">did::DIDDocument</a>, fragment: &<a href="_String">string::String</a>): <a href="_Option">option::Option</a>&lt;<a href="did.md#0x3_did_Service">did::Service</a>&gt;
</code></pre>



<a name="0x3_did_get_service_id"></a>

## Function `get_service_id`



<pre><code><b>public</b> <b>fun</b> <a href="did.md#0x3_did_get_service_id">get_service_id</a>(service: &<a href="did.md#0x3_did_Service">did::Service</a>): &<a href="did.md#0x3_did_ServiceID">did::ServiceID</a>
</code></pre>



<a name="0x3_did_get_service_type"></a>

## Function `get_service_type`



<pre><code><b>public</b> <b>fun</b> <a href="did.md#0x3_did_get_service_type">get_service_type</a>(service: &<a href="did.md#0x3_did_Service">did::Service</a>): &<a href="_String">string::String</a>
</code></pre>



<a name="0x3_did_get_service_endpoint"></a>

## Function `get_service_endpoint`



<pre><code><b>public</b> <b>fun</b> <a href="did.md#0x3_did_get_service_endpoint">get_service_endpoint</a>(service: &<a href="did.md#0x3_did_Service">did::Service</a>): &<a href="_String">string::String</a>
</code></pre>



<a name="0x3_did_get_service_properties"></a>

## Function `get_service_properties`



<pre><code><b>public</b> <b>fun</b> <a href="did.md#0x3_did_get_service_properties">get_service_properties</a>(service: &<a href="did.md#0x3_did_Service">did::Service</a>): &<a href="_SimpleMap">simple_map::SimpleMap</a>&lt;<a href="_String">string::String</a>, <a href="_String">string::String</a>&gt;
</code></pre>



<a name="0x3_did_get_also_known_as"></a>

## Function `get_also_known_as`

Get also known as from DIDDocument


<pre><code><b>public</b> <b>fun</b> <a href="did.md#0x3_did_get_also_known_as">get_also_known_as</a>(did_doc: &<a href="did.md#0x3_did_DIDDocument">did::DIDDocument</a>): &<a href="">vector</a>&lt;<a href="_String">string::String</a>&gt;
</code></pre>



<a name="0x3_did_get_created_timestamp_by_object_id"></a>

## Function `get_created_timestamp_by_object_id`

Get created timestamp from Object system
This accesses the Object's metadata created_at timestamp


<pre><code><b>public</b> <b>fun</b> <a href="did.md#0x3_did_get_created_timestamp_by_object_id">get_created_timestamp_by_object_id</a>(object_id: <a href="_ObjectID">object::ObjectID</a>): u64
</code></pre>



<a name="0x3_did_get_updated_timestamp_by_object_id"></a>

## Function `get_updated_timestamp_by_object_id`

Get updated timestamp from Object system
This accesses the Object's metadata updated_at timestamp


<pre><code><b>public</b> <b>fun</b> <a href="did.md#0x3_did_get_updated_timestamp_by_object_id">get_updated_timestamp_by_object_id</a>(object_id: <a href="_ObjectID">object::ObjectID</a>): u64
</code></pre>



<a name="0x3_did_get_created_timestamp"></a>

## Function `get_created_timestamp`

Get created timestamp for a DID document by address


<pre><code><b>public</b> <b>fun</b> <a href="did.md#0x3_did_get_created_timestamp">get_created_timestamp</a>(addr: <b>address</b>): u64
</code></pre>



<a name="0x3_did_get_updated_timestamp"></a>

## Function `get_updated_timestamp`

Get updated timestamp for a DID document by address


<pre><code><b>public</b> <b>fun</b> <a href="did.md#0x3_did_get_updated_timestamp">get_updated_timestamp</a>(addr: <b>address</b>): u64
</code></pre>



<a name="0x3_did_get_did_created_timestamp"></a>

## Function `get_did_created_timestamp`

Get created timestamp from DIDDocument reference
This is a convenience function that extracts the address and calls get_created_timestamp


<pre><code><b>public</b> <b>fun</b> <a href="did.md#0x3_did_get_did_created_timestamp">get_did_created_timestamp</a>(did_doc: &<a href="did.md#0x3_did_DIDDocument">did::DIDDocument</a>): u64
</code></pre>



<a name="0x3_did_get_did_updated_timestamp"></a>

## Function `get_did_updated_timestamp`

Get updated timestamp from DIDDocument reference
This is a convenience function that extracts the address and calls get_updated_timestamp


<pre><code><b>public</b> <b>fun</b> <a href="did.md#0x3_did_get_did_updated_timestamp">get_did_updated_timestamp</a>(did_doc: &<a href="did.md#0x3_did_DIDDocument">did::DIDDocument</a>): u64
</code></pre>



<a name="0x3_did_get_did_address"></a>

## Function `get_did_address`



<pre><code><b>public</b> <b>fun</b> <a href="did.md#0x3_did_get_did_address">get_did_address</a>(did_doc: &<a href="did.md#0x3_did_DIDDocument">did::DIDDocument</a>): <b>address</b>
</code></pre>



<a name="0x3_did_find_verification_method_by_session_key"></a>

## Function `find_verification_method_by_session_key`

Find the verification method fragment that corresponds to the given session key
Returns None if no matching verification method is found


<pre><code><b>public</b> <b>fun</b> <a href="did.md#0x3_did_find_verification_method_by_session_key">find_verification_method_by_session_key</a>(did_document_data: &<a href="did.md#0x3_did_DIDDocument">did::DIDDocument</a>, <a href="session_key.md#0x3_session_key">session_key</a>: &<a href="">vector</a>&lt;u8&gt;): <a href="_Option">option::Option</a>&lt;<a href="_String">string::String</a>&gt;
</code></pre>

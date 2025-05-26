// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// Move Object
/// For more details, please refer to https://rooch.network/docs/developer-guides/object
module moveos_std::object {
    use std::hash;
    use std::option::{Self, Option};
    use std::string;
    use std::string::String;
    use std::vector;
    use moveos_std::hex;
    use moveos_std::signer;
    use moveos_std::tx_context;
    use moveos_std::bcs;
    use moveos_std::type_info;
    use moveos_std::address;

    friend moveos_std::account;
    friend moveos_std::module_store;
    friend moveos_std::event;
    friend moveos_std::table;
    friend moveos_std::linked_table;
    friend moveos_std::type_table;
    friend moveos_std::bag;
    friend moveos_std::genesis;
    friend moveos_std::timestamp;

    /// The Object or dynamic field already exists
    const ErrorAlreadyExists: u64 = 1;
    /// Can not found the Object or dynamic field
    const ErrorNotFound: u64 = 2;
    const ErrorInvalidOwnerAddress: u64 = 3;
    const ErrorObjectOwnerNotMatch: u64 = 4;
    const ErrorObjectNotShared: u64 = 5;
    ///Can not take out the object which is bound to the account
    const ErrorObjectIsBound: u64 = 6;
    /// The object or field is already borrowed
    const ErrorObjectAlreadyBorrowed: u64 = 7;
    /// The dynamic fields is not empty
    const ErrorFieldsNotEmpty: u64 = 8;
    const ErrorObjectFrozen: u64 = 9;
    /// The type of the object or field is mismatch
    const ErrorTypeMismatch: u64 = 10;
    /// The child object level is too deep
    const ErrorChildObjectTooDeep: u64 = 11;
    /// The object has no parent
    const ErrorWithoutParent: u64 = 12;
    /// The parent object is not match
    const ErrorParentNotMatch: u64 = 13;
    /// The object runtime error
    const ErrorObjectRuntimeError: u64 = 14;
    /// The object or field is already taken out or embedded in other struct
    const ErrorObjectAlreadyTakenOutOrEmbeded: u64 = 15;
    /// The hex string is invalid
    const ErrorInvalidHex: u64 = 16;

    const SYSTEM_OWNER_ADDRESS: address = @0x0;

    const SHARED_OBJECT_FLAG_MASK: u8 = 1;
    const FROZEN_OBJECT_FLAG_MASK: u8 = 1 << 1;

    const SPARSE_MERKLE_PLACEHOLDER_HASH: address = @0x5350415253455f4d45524b4c455f504c414345484f4c4445525f484153480000;

    #[data_struct]
    /// ObjectID is a unique identifier for the Object
    struct ObjectID has store, copy, drop {
        path: vector<address>,
    }

    /// Check if the object_id has parent
    /// The object_id has parent means the object_id is not the root object_id
    public fun has_parent(object_id: &ObjectID): bool {
        !vector::is_empty(&object_id.path)
    }

    public fun parent_id(object_id: &ObjectID): ObjectID {
        let path = object_id.path;
        assert!(!vector::is_empty(&path), ErrorWithoutParent);
        vector::pop_back(&mut path);
        ObjectID { path: path }
    }

    public(friend) fun child_id(parent_id: ObjectID, key: address): ObjectID {
        let path = parent_id.path;
        vector::push_back(&mut path, key);
        ObjectID { path }
    }

    fun into_parent_id_and_key(object_id: ObjectID): (ObjectID, address) {
        let path = object_id.path;
        assert!(!vector::is_empty(&path), ErrorWithoutParent);
        let key = vector::pop_back(&mut path);
        (ObjectID { path: path }, key)
    }

    /// Check if the `parent` is the parent of the `child`
    public fun is_parent(parent: &ObjectID, child: &ObjectID): bool {
        let parent_path = parent.path;
        let child_path = child.path;
        let parent_len = vector::length(&parent_path);
        let child_len = vector::length(&child_path);
        if (parent_len >= child_len) {
            return false
        };
        vector::pop_back(&mut child_path);
        parent_path == child_path
    }

    public fun is_root(object_id: &ObjectID): bool {
        vector::is_empty(&object_id.path)
    }

    /// Generate a new ObjectID from an address
    public(friend) fun address_to_object_id(address: address): ObjectID {
        ObjectID { path: vector::singleton(address) }
    }

    public fun named_object_id<T>(): ObjectID {
        address_to_object_id(
            address::from_bytes(
                hash::sha3_256(
                    std::string::into_bytes(type_info::type_name<T>())
                )
            )
        )
    }

    public fun account_named_object_id<T>(account: address): ObjectID {
        let bytes = bcs::to_bytes(&account);
        vector::append(&mut bytes, *std::string::bytes(&type_info::type_name<T>()));
        address_to_object_id(
            address::from_bytes(
                hash::sha3_256(bytes)
            )
        )
    }

    public fun custom_object_id<ID: store + copy + drop, T>(id: ID): ObjectID {
        address_to_object_id(derive_object_key<ID, T>(id))
    }

    public fun custom_object_id_with_parent<ID: store + copy + drop, T>(parent_id: ObjectID, id: ID): ObjectID {
        let child = derive_object_key<ID, T>(id);
        let path = parent_id.path;
        vector::push_back(&mut path, child);
        ObjectID { path }
    }

    /// the ObjectI::to_string() format is the same as ObjectID::to_str() in Rust
    public fun to_string(id: &ObjectID): String{
        let bytes = vector::empty<u8>();
        let i = 0;

        // Flatten all addresses into a single byte vector
        while (i < vector::length(&id.path)) {
            let addr = *vector::borrow(&id.path, i);
            let addr_bytes = bcs::to_bytes(&addr);
            vector::append(&mut bytes, addr_bytes);
            i = i + 1;
        };

        // Convert to hex string with "0x" prefix
        let hex_value = hex::encode(bytes);
        let hex_str = string::utf8(b"0x");
        string::append(&mut hex_str, string::utf8(hex_value));
        hex_str
    }

    public fun from_string(str: &String): ObjectID{
        let bytes = *string::bytes(str);

        // Strip "0x" prefix if present
        if (vector::length(&bytes) >= 2 && vector::slice(&bytes, 0, 2) == b"0x") {
            bytes = vector::slice(&bytes, 2, vector::length(&bytes));
        };

        // Handle empty string (root object)
        if (vector::is_empty(&bytes)) {
            return ObjectID { path: vector::empty() }
        };

        // Pad with zeros if too short
        let hex_len = vector::length(&bytes);
        if (hex_len < address::length() * 2) {
            let padded = vector::empty<u8>();
            let i = 0;
            while (i < address::length() * 2 - hex_len) {
                vector::append(&mut padded, b"0");
                i = i + 1;
            };
            vector::append(&mut padded, bytes);
            bytes = padded;
        };

        // Convert hex string to bytes and create address
        let addr_bytes = hex::decode(&bytes);
        let path = create_address_from_bytes(addr_bytes);

        ObjectID { path }
    }

    /// Create addresses from bytes
    fun create_address_from_bytes(bytes: vector<u8>): vector<address> {
        assert!(vector::length(&bytes) >= address::length(), ErrorInvalidHex);
        let addresses = vector::empty<address>();

        while (vector::length(&bytes) >= address::length()) {
            let addr_bytes = vector::slice(&bytes, 0, address::length());
            let addr = address::from_bytes(addr_bytes);
            vector::push_back(&mut addresses, addr);
            bytes = vector::slice(&bytes, address::length(), vector::length(&bytes));
        };
        addresses

    }

    /// Object<T> is a pointer type to the Object in storage, It has `key` and `store` ability.
    struct Object<phantom T> has key, store {
        id: ObjectID,
    }

    /// The dynamic field
    struct DynamicField<Name, Value> has key, store {
        name: Name,
        value: Value
    }

    #[private_generics(T)]
    /// Create a new Object, Add the Object to the global object storage and return the Object
    public fun new<T: key>(value: T): Object<T> {
        let id = derive_object_id();
        new_with_object_id(id, value)
    }

    #[private_generics(T)]
    /// Create a new object with custom ID, the ObjectID is generated by the `id` and type_name of `T`
    /// The caller must ensure that the `id` is unique
    public fun new_with_id<ID: store + copy + drop, T: key>(id: ID, value: T): Object<T> {
        let id = custom_object_id<ID, T>(id);
        new_with_object_id(id, value)
    }

    #[private_generics(T)]
    /// Create a new named Object, the ObjectID is generated by the type_name of `T`
    public fun new_named_object<T: key>(value: T): Object<T> {
        let id = named_object_id<T>();
        new_with_object_id(id, value)
    }

    #[private_generics(T)]
    /// Create a new account named object, the ObjectID is generated by the account address and type_name of `T`
    public fun new_account_named_object<T: key>(account: address, value: T): Object<T> {
        let id = account_named_object_id<T>(account);
        new_with_object_id(id, value)
    }

    public(friend) fun new_with_object_id<T: key>(id: ObjectID, value: T): Object<T> {
        let (parent, key) = into_parent_id_and_key(id);
        native_add_field<T>(parent, key, value)
    }

    #[private_generics(P, T)]
    /// Create a new object under the parent object
    public fun new_with_parent<P: key, T: key>(parent: &mut Object<P>, value: T): Object<T> {
        let child_key = tx_context::fresh_address();
        new_with_parent_and_key(parent, child_key, value)
    }

    #[private_generics(P, T)]
    /// Create a new object under the parent object with custom ID, the ObjectID is generated by `custom_object_id_with_parent`
    public fun new_with_parent_and_id<P: key, ID: store + copy + drop, T: key>(parent: &mut Object<P>, id: ID, value: T): Object<T> {
        let child_key = derive_object_key<ID, T>(id);
        new_with_parent_and_key(parent, child_key, value)
    }

    public(friend) fun new_with_parent_and_key<P: key, T: key>(parent: &mut Object<P>, child_key: address, value: T): Object<T>{
        // Currently, the child object level is limited to 2
        assert!(vector::length(&parent.id.path) < 2, ErrorChildObjectTooDeep);
        native_add_field(parent.id, child_key, value)
    }

    /// Borrow the object value
    public fun borrow<T: key>(self: &Object<T>): &T {
        let (parent, key) = into_parent_id_and_key(self.id);
        native_borrow_field<T>(parent, key)
    }

    /// Borrow the object mutable value
    public fun borrow_mut<T: key>(self: &mut Object<T>): &mut T {
        let (parent, key) = into_parent_id_and_key(self.id);
        native_borrow_mut_field<T>(parent, key)
    }

    /// Check if the object with `object_id` exists in the global object storage
    public fun exists_object(object_id: ObjectID): bool {
        let (parent, key) = into_parent_id_and_key(object_id);
        native_contains_field(parent, key)
    }

    /// Check if the object exists in the global object storage and the type of the object is `T`
    public fun exists_object_with_type<T: key>(object_id: ObjectID): bool {
        let (parent, key) = into_parent_id_and_key(object_id);
        native_contains_field_with_value_type<T>(parent, key)
    }

    /// Borrow Object from object store by object_id
    /// Any one can borrow an `&Object<T>` from the global object storage
    /// Except the object is embedded in other struct
    public fun borrow_object<T: key>(object_id: ObjectID): &Object<T> {
        native_borrow_object<T>(object_id)
    }

    /// Borrow mut Object by `owner` and `object_id`
    public fun borrow_mut_object<T: key>(owner: &signer, object_id: ObjectID): &mut Object<T> {
        let owner_address = signer::address_of(owner);
        let obj_ref = native_borrow_mut_object<T>(object_id);
        assert!(owner_address == owner(obj_ref), ErrorObjectOwnerNotMatch);
        obj_ref
    }

    #[private_generics(T)]
    /// Borrow mut Object by `object_id`, Only the module of `T` can borrow the `Object<T>` with object_id.
    /// Except the object is frozen or is embedded in other struct
    public fun borrow_mut_object_extend<T: key>(object_id: ObjectID): &mut Object<T> {
        let obj_ref = native_borrow_mut_object<T>(object_id);
        assert!(!is_frozen(obj_ref), ErrorObjectFrozen);
        obj_ref
    }

    /// Take out the Object by `owner` and `object_id`
    /// The `T` must have `key + store` ability.
    public fun take_object<T: key + store>(owner: &signer, object_id: ObjectID): Object<T> {
        let owner_address = signer::address_of(owner);
        let obj = native_take_object<T>(object_id);
        assert!(owner_address == owner(&obj), ErrorObjectOwnerNotMatch);
        obj
    }

    #[private_generics(T)]
    /// Take out the Object by `object_id`
    /// This function is for developer to extend, Only the module of `T` can call this function.
    public fun take_object_extend<T: key>(object_id: ObjectID): Object<T> {
        let obj = native_take_object<T>(object_id);
        assert!(!is_frozen(&obj), ErrorObjectFrozen);
        obj
    }

    /// Borrow mut Shared Object by object_id
    public fun borrow_mut_object_shared<T: key>(object_id: ObjectID): &mut Object<T> {
        let obj_ref = native_borrow_mut_object<T>(object_id);
        assert!(is_shared(obj_ref), ErrorObjectNotShared);
        obj_ref
    }


    #[private_generics(T)]
    /// Remove the object from the global storage, and return the object value
    /// This function is only can be called by the module of `T`.
    /// The caller must ensure that the dynamic fields are empty before delete the Object
    public fun remove<T: key>(self: Object<T>): T {
        let size = field_size(&self);
        assert!(size == 0, ErrorFieldsNotEmpty);
        remove_unchecked(self)
    }

    /// Remove the object from the global storage, and return the object value
    /// Do not check if the dynamic fields are empty
    public(friend) fun remove_unchecked<T: key>(self: Object<T>): T {
        let Object{id} = self;
        let (parent, key) = into_parent_id_and_key(id);
        let value = native_remove_field<T>(parent, key);
        value
    }

    /// Make the Object shared, Any one can get the &mut Object<T> from shared object
    /// The module of `T` can call `take_object_extend` to take out the shared object, then remove the shared object.
    public fun to_shared<T: key>(self: Object<T>) {
        native_to_shared_object(self);
    }

    public fun is_shared<T: key>(self: &Object<T>): bool {
        let flag = native_object_flag(self.id);
        flag & SHARED_OBJECT_FLAG_MASK == SHARED_OBJECT_FLAG_MASK
    }

    /// Make the Object frozen, No one can not get the &mut Object<T> from frozen object
    public fun to_frozen<T: key>(self: Object<T>) {
        native_to_frozen_object(self);
    }

    public fun is_frozen<T: key>(self: &Object<T>): bool {
        let flag = native_object_flag(self.id);
        flag & FROZEN_OBJECT_FLAG_MASK == FROZEN_OBJECT_FLAG_MASK
    }

    /// Transfer the object to the new owner
    /// Only the `T` with `store` can be directly transferred.
    public fun transfer<T: key + store>(self: Object<T>, new_owner: address) {
        assert!(new_owner != SYSTEM_OWNER_ADDRESS, ErrorInvalidOwnerAddress);
        native_transfer_object(self, new_owner);
    }

    #[private_generics(T)]
    /// Transfer the object to the new owner
    /// This function is for the module of `T` to extend the `transfer` function.
    public fun transfer_extend<T: key>(self: Object<T>, new_owner: address) {
        assert!(new_owner != SYSTEM_OWNER_ADDRESS, ErrorInvalidOwnerAddress);
        native_transfer_object(self, new_owner);
    }

    public fun id<T>(self: &Object<T>): ObjectID {
        self.id
    }

    public fun owner<T: key>(self: &Object<T>): address {
        native_object_owner(self.id)
    }

    public fun is_system_owned<T: key>(self: &Object<T>): bool {
        owner(self) == SYSTEM_OWNER_ADDRESS
    }

    public fun is_user_owned<T: key>(self: &Object<T>): bool {
        owner(self) != SYSTEM_OWNER_ADDRESS
    }

    // === Object Storage ===

    /// The global root object id is `[]`
    fun root_object_id(): ObjectID {
        ObjectID { path: vector::empty() }
    }

    // === Object Dynamic Fields ===

    #[private_generics(T)]
    /// Add a dynamic field to the object. Aborts if an field for this
    /// key already exists. The field itself is not stored in the
    /// object, and cannot be discovered from it.
    public fun add_field<T: key, Name: copy + drop + store, Value: store>(obj: &mut Object<T>, name: Name, val: Value) {
        add_field_internal<Name, Value>(obj.id, name, val);
    }

    // Add field and wrap the key and value to DynamicField
    public(friend) fun add_field_internal<Name: copy + drop + store, Value>(obj_id: ObjectID, name: Name, value: Value) {
        let key = derive_field_key(name);
        let field = DynamicField{
            name,
            value
        };
        let obj = native_add_field<DynamicField<Name,Value>>(obj_id, key, field);
        //Drop the Object, so the field can not access via Object pointer.
        let Object{id:_} = obj;
    }

    /// Acquire an immutable reference to the value which `key` maps to.
    /// Aborts if there is no field for `key`.
    public fun borrow_field<T: key, Name: copy + drop + store, Value: store>(obj: &Object<T>, name: Name): &Value {
        borrow_field_internal<Name, Value>(obj.id, name)
    }

    /// Borrow FieldValue and return the val of FieldValue
    public(friend) fun borrow_field_internal<Name: copy + drop + store, Value>(obj_id: ObjectID, name: Name): &Value {
        let field_key = derive_field_key(name);
        &native_borrow_field<DynamicField<Name, Value>>(obj_id, field_key).value
    }

    /// Direct field access based on field_key and return field value reference.
    public(friend) fun borrow_field_with_key_internal<Name: copy + drop + store, Value>(obj_id: ObjectID, field_key: address): (&Name, &Value) {
        let df = native_borrow_field<DynamicField<Name, Value>>(obj_id, field_key);
        (&df.name, &df.value)
    }

    /// Acquire an immutable reference to the value which `key` maps to.
    /// Returns specified default value if there is no field for `key`.
    public fun borrow_field_with_default<T: key, Name: copy + drop + store, Value: store>(obj: &Object<T>, name: Name, default: &Value): &Value {
        borrow_field_with_default_internal<Name, Value>(obj.id, name, default)
    }

    /// Acquire an immutable reference to the value which `key` maps to.
    /// Returns specified default value if there is no field for `key`.
    fun borrow_field_with_default_internal<Name: copy + drop + store, Value: store>(obj_id: ObjectID, name: Name, default: &Value): &Value {
        if (!contains_field_internal<Name>(obj_id, name)) {
            default
        } else {
            borrow_field_internal(obj_id, name)
        }
    }

    #[private_generics(T)]
    /// Acquire a mutable reference to the value which `key` maps to.
    /// Aborts if there is no field for `key`.
    public fun borrow_mut_field<T: key, Name: copy + drop + store, Value: store>(obj: &mut Object<T>, name: Name): &mut Value {
        borrow_mut_field_internal<Name, Value>(obj.id, name)
    }

    /// Acquire a mutable reference to the value which `key` maps to.
    /// Aborts if there is no field for `key`.
    public(friend) fun borrow_mut_field_internal<Name: copy + drop + store, Value>(obj_id: ObjectID, name: Name): &mut Value {
        let field_key = derive_field_key(name);
        &mut native_borrow_mut_field<DynamicField<Name, Value>>(obj_id, field_key).value
    }

    /// Obtain a mutable reference to the value associated with `field_key`.
    /// Will abort if no field exists for the given `field_key`.
    public(friend) fun borrow_mut_field_with_key_internal<Name: copy + drop + store, Value>(obj_id: ObjectID, field_key: address): (&Name, &mut Value) {
        let df = native_borrow_mut_field<DynamicField<Name, Value>>(obj_id, field_key);
        (&df.name, &mut df.value)
    }

    #[private_generics(T)]
    /// Acquire a mutable reference to the value which `key` maps to.
    /// Insert the pair (`key`, `default`) first if there is no field for `key`.
    public fun borrow_mut_field_with_default<T: key, Name: copy + drop + store, Value: store + drop>(
        obj: &mut Object<T>,
        name: Name,
        default: Value
    ): &mut Value {
        borrow_mut_field_with_default_internal<T, Name, Value>(obj.id, name, default)
    }

    fun borrow_mut_field_with_default_internal<T: key, Name: copy + drop + store, Value: store + drop>(
        obj_id: ObjectID,
        name: Name,
        default: Value
    ): &mut Value {
        if (!contains_field_internal<Name>(obj_id, name)) {
            add_field_internal<Name, Value>(obj_id, name, default)
        };
        borrow_mut_field_internal(obj_id, name)
    }

    #[private_generics(T)]
    /// Insert the pair (`key`, `value`) if there is no field for `key`.
    /// update the value of the field for `key` to `value` otherwise
    public fun upsert_field<T: key, Name: copy + drop + store, Value: store + drop>(obj: &mut Object<T>, name: Name, value: Value) {
        upsert_field_internal<T, Name, Value>(obj.id, name, value)
    }

    fun upsert_field_internal<T: key, Name: copy + drop + store, Value: store + drop>(obj_id: ObjectID, name: Name, value: Value) {
        if (!contains_field_internal<Name>(obj_id, copy name)) {
            add_field_internal<Name, Value>(obj_id, name, value)
        } else {
            let ref = borrow_mut_field_internal(obj_id, name);
            *ref = value;
        };
    }

    #[private_generics(T)]
    /// Remove from `object` and return the value which `key` maps to.
    /// Aborts if there is no field for `key`.
    public fun remove_field<T: key, Name: copy + drop + store, Value: store>(obj: &mut Object<T>, name: Name): Value {
        remove_field_internal<T, Name, Value>(obj.id, name)
    }

    public(friend) fun remove_field_internal<T: key, Name: copy + drop + store, Value>(obj_id: ObjectID, name: Name): Value {
        let key = derive_field_key(name);
        let DynamicField { name:_, value } = native_remove_field<DynamicField<Name, Value>>(obj_id, key);
        value
    }

    /// Returns true if `object` contains an field for `key`, include normal field and object field
    public fun contains_field<T: key, Name: copy + drop + store>(obj: &Object<T>, name: Name): bool {
        contains_field_internal<Name>(obj.id, name)
    }

    public(friend) fun contains_field_internal<Name: copy + drop + store>(obj_id: ObjectID, name: Name): bool {
        let key = derive_field_key(name);
        native_contains_field(obj_id, key)
    }

    /// Returns true if `object` contains an field for `key` and the value type is `Value`. only for normal field
    public fun contains_field_with_type<T: key, Name: copy + drop + store, Value: store>(obj: &Object<T>, name: Name): bool {
        let key = derive_field_key(name);
        native_contains_field_with_value_type<DynamicField<Name, Value>>(obj.id, key)
    }

    /// Returns the size of the object fields, the number of key-value pairs
    public fun field_size<T: key>(obj: &Object<T>): u64 {
        native_object_size(obj.id)
    }

    /// List all field names of the object
    public(friend) fun list_field_keys<T: key, Name: copy + drop + store>(obj: &Object<T>, name: Option<Name>, limit: u64): vector<address> {
        let cursor = if (option::is_some(&name)) {
            option::some(derive_field_key(option::extract(&mut name)))
        } else {
            option::none()
        };
        native_list_field_keys(obj.id, cursor, limit)
    }


    // ====== Utility functions ======

    fun derive_object_id(): ObjectID {
        address_to_object_id(tx_context::fresh_address())
    }

    /// Derive a field key from the `key` and the type name of `Name`
    /// hash(key || key_type_tag)
    /// We just need to make sure the field key is unique in the parent object
    fun derive_field_key<Name: store + copy + drop>(key: Name): address {
        let bytes = bcs::to_bytes(&key);
        vector::append(&mut bytes, std::string::into_bytes(type_info::type_name<Name>()));
        let hash = hash::sha3_256(bytes);
        address::from_bytes(hash)
    }

    /// Derive a object key from the `id` and the type name of `T`
    /// hash(id || T_type_tag)
    /// We need to make sure the key is unique in the global object storage
    /// So we append the type name of `T`
    /// The developer should ensure the uniqueness of the `id`
    /// If two different `T` use same `id`, the object key will be different.
    fun derive_object_key<ID: store + copy + drop, T>(id: ID): address {
        let bytes = bcs::to_bytes(&id);
        vector::append(&mut bytes, std::string::into_bytes(type_info::type_name<T>()));
        let hash = hash::sha3_256(bytes);
        address::from_bytes(hash)
    }

    // ===== ObjectMeta native functions ======

    native fun native_object_owner(object_id: ObjectID): address;
    // Object fields size
    native fun native_object_size(object_id: ObjectID): u64;
    native fun native_object_flag(object_id: ObjectID): u8;
    native fun native_object_created_at(object_id: ObjectID): u64;
    native fun native_object_updated_at(object_id: ObjectID): u64;

    // ===== Object<T> native functions ====

    native fun native_transfer_object<T: key>(obj: Object<T>, new_owner: address);

    native fun native_to_shared_object<T: key>(obj: Object<T>);

    native fun native_to_frozen_object<T: key>(obj: Object<T>);

    native fun native_borrow_object<T: key>(object_id: ObjectID): &Object<T>;

    native fun native_borrow_mut_object<T: key>(object_id: ObjectID): &mut Object<T>;

    native fun native_take_object<T: key>(object_id: ObjectID): Object<T>;

    // ==== object field native functions ====

    native fun native_add_field<V>(obj_id: ObjectID, key: address, val: V): Object<V>;

    native fun native_borrow_field<V>(obj_id: ObjectID, key: address): &V;

    native fun native_borrow_mut_field<V>(obj_id: ObjectID, key: address): &mut V;

    native fun native_contains_field(obj_id: ObjectID, key: address): bool;

    /// If the Object contains a field for `key` with value type `V`.
    native fun native_contains_field_with_value_type<V>(obj_id: ObjectID, key: address): bool;

    native fun native_remove_field<V>(obj_id: ObjectID, key: address): V;

    native fun native_list_field_keys(obj_id: ObjectID, cursor: Option<address>, limit: u64): vector<address>;

    // ===== Public Object metadata access functions =====

    /// Get the creation timestamp of an object
    public fun created_at(object_id: ObjectID): u64 {
        native_object_created_at(object_id)
    }

    /// Get the last update timestamp of an object
    public fun updated_at(object_id: ObjectID): u64 {
        native_object_updated_at(object_id)
    }

    #[test_only]
    public fun new_object_id_for_test(path: vector<address>): ObjectID {
        ObjectID { path }
    }

    #[test_only]
    public fun derive_object_id_for_test():ObjectID{
        address_to_object_id(tx_context::fresh_address())
    }

    #[test_only]
    /// Testing only: allows to drop a Object even if it's fields is not empty.
    public fun drop_unchecked<T: key>(self: Object<T>): T {
        remove_unchecked(self)
    }

    #[test_only]
    struct TestParent has key {
    }

    #[test_only]
    struct TestStruct has key {
        count: u64,
    }

    #[test_only]
    struct TestStruct2 has key {
        count: u64,
    }

    #[test(sender = @0x42)]
    fun test_object(sender: signer) {
        let sender_addr = std::signer::address_of(&sender);
        let init_count = 12;
        let test_struct = TestStruct {
            count: init_count,
        };
        let obj = new<TestStruct>(test_struct);
        let obj_id = obj.id;
        assert!(exists_object(obj_id), 1000);
        transfer_extend(obj, sender_addr);
        {
            let obj_ref = borrow_object<TestStruct>(obj_id);
            assert!(owner(obj_ref) == sender_addr, 1001);
        };
        {
            let obj_ref = borrow_mut_object_extend<TestStruct>(obj_id);
            let test_struct_mut = borrow_mut(obj_ref);
            test_struct_mut.count = test_struct_mut.count + 1;
        };
        {
            let obj_ref = borrow_object<TestStruct>(obj_id);
            let test_struct_ref = borrow(obj_ref);
            assert!(test_struct_ref.count == init_count + 1, 1002);
        };
        {
            let obj = take_object_extend<TestStruct>(obj_id);
            transfer_extend(obj, @moveos_std);
            let obj_ref = borrow_object<TestStruct>(obj_id);
            assert!(owner(obj_ref) != sender_addr, 1003);
        };
        let obj = take_object_extend<TestStruct>(obj_id);
        let test_struct = remove(obj);
        let TestStruct { count: _count } = test_struct;
    }

    #[test]
    fun test_shared() {
        let obj = new(TestStruct { count: 1 });
        let obj_id = obj.id;
        assert!(!is_shared(&obj), 1000);
        assert!(!is_frozen(&obj), 1001);
        to_shared(obj);
        let obj_ref = borrow_object<TestStruct>(obj_id);
        assert!(is_shared(obj_ref), 1002);
        assert!(!is_frozen(obj_ref), 1003);
    }

    #[test]
    fun test_frozen() {
        let obj = new(TestStruct { count: 1 });
        let obj_id = obj.id;
        assert!(!is_shared(&obj), 1000);
        assert!(!is_frozen(&obj), 1001);
        to_frozen(obj);
        let obj_ref = borrow_object<TestStruct>(obj_id);
        assert!(!is_shared(obj_ref), 1002);
        assert!(is_frozen(obj_ref), 1003);
    }

    #[test]
    #[expected_failure(abort_code = ErrorNotFound, location = moveos_std::object)]
    fun test_borrow_not_exist_failure() {
        let obj = new(TestStruct { count: 1 });
        let object_id = obj.id;
        let TestStruct { count : _ } = remove(obj);
        let _obj_ref = borrow_object<TestStruct>(object_id);
    }

    #[test]
    #[expected_failure(abort_code = ErrorNotFound, location = moveos_std::object)]
    fun test_remove_then_borrow_failure() {
        let obj = new(TestStruct { count: 1 });
        let obj_id = id(&obj);
        let TestStruct { count : _ } = remove(obj);
        let _obj_ref = borrow_object<TestStruct>(obj_id);
    }

    #[test]
    #[expected_failure(abort_code = ErrorTypeMismatch, location = moveos_std::object)]
    fun test_type_mismatch() {
        let object_id = derive_object_id();
        let obj = new_with_object_id(object_id, TestStruct { count: 1 });
        transfer_extend(obj, @moveos_std);
        {
            let _test_struct2_object = borrow_object<TestStruct2>(object_id);
        };
    }

    #[test_only]
    struct TestStructID has store, copy, drop {
        id: u64,
    }

    #[test]
    fun test_custom_object_id() {
        let id = TestStructID { id: 1 };
        let object_id = custom_object_id<TestStructID, TestStruct>(id);
        //ensure the object_id is the same as the object_id generated by the object.rs
        assert!(object_id.path == vector[@0x6c62fde28fadbe652ba0eec95f5f096c900c94191a2debca96276b2de4b6ee3a], 1);
    }

    #[test]
    fun test_remove_object_success_with_dynamic_fields() {
        let obj = new(TestStruct { count: 1 });
        add_field(&mut obj, 1u64, 1u64);
        let _v: u64 = remove_field(&mut obj, 1u64);
        let s = remove(obj);
        let TestStruct { count : _ } = s;
    }

    #[test]
    #[expected_failure(abort_code = ErrorFieldsNotEmpty, location = Self)]
    fun test_remove_object_faild_with_dynamic_fields() {
        let obj = new(TestStruct { count: 1 });
        add_field(&mut obj, 1u64, 1u64);
        let s = remove(obj);
        let TestStruct { count : _ } = s;
    }

    #[test]
    fun test_new() {
        let obj1 = new(TestStruct { count: 1 });
        let obj2 = new(TestStruct { count: 2 });
        assert!(obj1.id != obj2.id, 1);
        let TestStruct { count: _ } = drop_unchecked(obj1);
        let TestStruct { count: _ } = drop_unchecked(obj2);
    }

    #[test]
    fun test_object_mut() {
        let obj = new(TestStruct { count: 1 });
        {
            let obj_value = borrow_mut(&mut obj);
            obj_value.count = 2;
        };
        {
            let obj_value = borrow(&obj);
            assert!(obj_value.count == 2, 1000);
        };
        let TestStruct { count: _ } = remove(obj);
    }

    #[test(alice = @0x42)]
    fun test_borrow_object(alice: signer) {
        let alice_addr = signer::address_of(&alice);

        let obj = new(TestStruct { count: 1 });
        let object_id = id(&obj);
        transfer_extend(obj, alice_addr);

        //test borrow_object by id
        {
            let _obj = borrow_object<TestStruct>(object_id);
        };
    }

    #[test(alice = @0x42, bob = @0x43)]
    #[expected_failure(abort_code = 4, location = Self)]
    fun test_borrow_mut_object(alice: &signer, bob: &signer) {
        let alice_addr = signer::address_of(alice);
        let obj = new(TestStruct { count: 1 });
        let object_id = id(&obj);
        transfer_extend(obj, alice_addr);

        //test borrow_mut_object by owner
        {
            let _obj = borrow_mut_object<TestStruct>(alice, object_id);
        };

        // borrow_mut_object by non-owner failed
        {
            let _obj = borrow_mut_object<TestStruct>(bob, object_id);
        };
    }

    #[test]
    fun test_shared_object() {
        let obj = new(TestStruct { count: 1 });
        let object_id = id(&obj);

        to_shared(obj);
        // any one can borrow_mut the shared object
        {
            let obj = borrow_mut_object_shared<TestStruct>(object_id);
            assert!(is_shared(obj), 1000);
        };
    }


    #[test]
    #[expected_failure(abort_code = ErrorObjectFrozen, location = Self)]
    fun test_frozen_object_by_extend() {
        let obj = new(TestStruct { count: 1 });
        let object_id = id(&obj);
        to_frozen(obj);
        //test borrow_object
        {
            let _obj = borrow_object<TestStruct>(object_id);
        };

        // none one can borrow_mut from the frozen object
        {
            let _obj = borrow_mut_object_extend<TestStruct>(object_id);
        };
    }

    #[test]
    fun test_new_with_parent() {
        let parent = new(TestStruct { count: 1 });
        let parent_id = id(&parent);
        to_shared(parent);
        let parent = borrow_mut_object_shared<TestStruct>(parent_id);
        let child = new_with_parent(parent, TestStruct { count: 2 });
        let TestStruct { count: _ } = remove(child);
    }

    #[test]
    fun test_child_field(){
        let parent = new(TestStruct { count: 1 });
        let parent_id = id(&parent);
        to_shared(parent);
        let parent = borrow_mut_object_shared<TestStruct>(parent_id);
        let child = new_with_parent(parent, TestStruct { count: 2 });
        add_field(&mut child, b"key", 1u64);
        {
            let v = borrow_mut_field(&mut child, b"key");
            *v = 2u64;
        };
        assert!(*borrow_field(&child, b"key") == 2u64, 1000);
        let _v:u64 = remove_field(&mut child, b"key");
        let TestStruct { count: _ } = remove(child);
    }

    #[test]
    fun test_parent_not_shared(){
        let parent = new(TestStruct { count: 1 });
        //let parent_id = id(&parent);
        let child = new_with_parent(&mut parent, TestStruct { count: 2 });
        let TestStruct { count: _ } = remove(child);
        let TestStruct{ count: _} = remove(parent);
    }

    #[test]
    #[expected_failure(abort_code = ErrorChildObjectTooDeep, location = moveos_std::object)]
    fun test_child_too_deep(){
        let parent = new(TestStruct { count: 1 });
        let parent_id = id(&parent);
        to_shared(parent);
        let parent = borrow_mut_object_shared<TestStruct>(parent_id);
        let child = new_with_parent(parent, TestStruct { count: 2 });
        let child_id = id(&child);
        to_shared(child);
        let child = borrow_mut_object_shared<TestStruct>(child_id);
        let grand_child = new_with_parent(child, TestStruct { count: 3 });
        let TestStruct { count: _ } = remove(grand_child);
    }

    #[test]
    fun test_child_field_upsert(){
        let parent = new(TestStruct { count: 1 });
        let parent_id = id(&parent);
        to_shared(parent);
        let parent = borrow_mut_object_shared<TestStruct>(parent_id);
        let child = new_with_parent(parent, TestStruct { count: 2 });
        upsert_field(&mut child, b"key", 1u64);
        assert!(*borrow_field(&child, b"key") == 1u64, 1000);
        upsert_field(&mut child, b"key", 2u64);
        assert!(*borrow_field(&child, b"key") == 2u64, 1000);
        let _v:u64 = remove_field(&mut child, b"key");
        let TestStruct { count: _ } = remove(child);
    }

    #[test]
    #[expected_failure(abort_code = ErrorObjectAlreadyBorrowed, location = Self)]
    fun test_borrow_two_reference(){
        let obj = new(TestStruct { count: 1 });
        let obj_id = id(&obj);
        to_shared(obj);
        let obj_ref1 = borrow_object<TestStruct>(obj_id);
        let obj_ref2 = borrow_object<TestStruct>(obj_id);
        assert!(obj_ref1.id == obj_ref2.id, 1000);
    }

    #[test]
    #[expected_failure(abort_code = ErrorObjectAlreadyBorrowed, location = Self)]
    fun test_borrow_two_mut_reference(){
        let obj = new(TestStruct { count: 1 });
        let obj_id = id(&obj);
        to_shared(obj);
        let obj_ref1 = borrow_mut_object_shared<TestStruct>(obj_id);
        let obj_ref2 = borrow_mut_object_shared<TestStruct>(obj_id);
        assert!(obj_ref1.id == obj_ref2.id, 1000);
    }

    #[test]
    fun test_take_shared_object_and_remove(){
        let obj = new(TestStruct { count: 1 });
        let obj_id = id(&obj);
        to_shared(obj);
        let obj = take_object_extend<TestStruct>(obj_id);
        let TestStruct{count:_} = remove(obj);
    }

    #[test_only]
    struct TestContainer has key {
        inner_obj: Object<TestStruct>,
    }

    #[test]
    fun test_embed_object_unpack_and_transfer(){
        let obj = new(TestStruct { count: 1 });
        let container = TestContainer {
            inner_obj: obj,
        };
        let container_obj = new(container);
        let container_obj_id = id(&container_obj);
        transfer_extend(container_obj, @moveos_std);
        let container_obj = take_object_extend<TestContainer>(container_obj_id);
        let TestContainer{inner_obj} = remove(container_obj);
        transfer_extend(inner_obj, @moveos_std);
    }

    #[test]
    #[expected_failure(abort_code = ErrorObjectAlreadyTakenOutOrEmbeded, location = Self)]
    fun test_borrow_embed_object_failed(){
        let obj = new(TestStruct { count: 1 });
        let id = id(&obj);
        let container = TestContainer {
            inner_obj: obj,
        };
        transfer_extend(new(container), @moveos_std);
        let _obj_ref = borrow_object<TestStruct>(id);
    }

    #[test]
    fun test_remove_field_and_add_again_with_same_type(){
        let obj = new(TestStruct { count: 1 });
        //let obj_id = id(&obj);
        add_field(&mut obj, b"key", 1u64);
        let _v:u64 = remove_field(&mut obj, b"key");
        add_field(&mut obj, b"key", 2u64);
        let _v:u64 = remove_field(&mut obj, b"key");
        let TestStruct{ count: _} = remove(obj);
    }

    #[test]
    fun test_remove_field_and_add_again_with_different_type(){
        let obj = new(TestStruct { count: 1 });
        //let obj_id = id(&obj);
        add_field(&mut obj, b"key", 1u64);
        let _v:u64 = remove_field(&mut obj, b"key");
        add_field(&mut obj, b"key", 2u128);
        let _v:u128 = remove_field(&mut obj, b"key");
        let TestStruct{ count: _} = remove(obj);
    }

    #[test]
    fun test_child_object_with_same_id_but_different_type(){
        let parent = new(TestParent {});
        let parent_id = id(&parent);
        to_shared(parent);
        let parent_ref = borrow_mut_object_shared<TestParent>(parent_id);
        let id = 1u64;
        let child1 = new_with_parent_and_id(parent_ref, id, TestStruct { count: 1 });
        let child_id1 = id(&child1);
        let child2 = new_with_parent_and_id(parent_ref, id, TestStruct2 { count: 2 });
        let child_id2 = id(&child2);
        assert!(child_id1 != child_id2, 1000);
        let TestStruct { count: _ } = remove(child1);
        let TestStruct2 { count: _ } = remove(child2);
    }

    #[test]
    fun test_child_object_with_same_id_remove_and_add_again(){
        let parent = new(TestParent {});
        let parent_id = id(&parent);
        to_shared(parent);
        let parent_ref = borrow_mut_object_shared<TestParent>(parent_id);
        let id = 1u64;
        let child1 = new_with_parent_and_id(parent_ref, id, TestStruct { count: 1 });
        let child_id1 = id(&child1);
        let TestStruct { count: _ } = remove(child1);
        let child2 = new_with_parent_and_id(parent_ref, id, TestStruct { count: 2 });
        let child_id2 = id(&child2);
        assert!(child_id1 == child_id2, 1000);
        let TestStruct { count: _ } = remove(child2);
    }

    #[test_only]
    fun field_key_derive_test<Name: store + copy + drop>(name: Name, expect_result: address){
        let key = derive_field_key(name);
        assert!(key == expect_result, 1000);
    }

    #[test]
    fun test_field_key_derive_cases(){
        //test vector
        field_key_derive_test(b"1", @0x7301c6d045ed0df28fa129f5a825b210c8300eb0f44bb302e8a54b5eebeae13f);
        //test string
        field_key_derive_test(std::string::utf8(b"1"), @0x5c01fed5cc173458597a3d55ec9942f1a385d5aa66f15e3615378d8a773e4d58);
        //test u8
        field_key_derive_test(1u8, @0x988ba0cd547556c2014c5e718b15fce912b95aa39db882de598b6ea841cde194);
        //test u64
        field_key_derive_test(1u64, @0x7eb4036673c8611e43c3eff1202446612f22a4b3bac92b7e14c0562ade5f1a3f);
        //test address
        field_key_derive_test(@0x1, @0x07d29b5cffb95d39f98baed1a973e676891bc9d379022aba6f4a2e4912a5e552);
    }

    #[test]
    fun test_list_fields(){
        use std::option;
        let obj = new(TestStruct { count: 1 });
        add_field(&mut obj, b"key1", 1u64);
        add_field(&mut obj, b"key2", 2u64);

        assert!(field_size(&obj) == 2, 1000);

        let field_keys = list_field_keys<TestStruct, vector<u8>>(&obj, option::none(), 10);
        std::debug::print(&field_keys);

        assert!(!vector::is_empty(&field_keys), 1001);
        assert!(vector::length(&field_keys) == 2, 1002);

        let field_key1 = *vector::borrow(&field_keys, 0);
        std::debug::print(&field_key1);

        let field1 = native_borrow_field<DynamicField<vector<u8>, u64>>(obj.id, field_key1);
        assert!(field1.name == b"key1", 1003);
        assert!(field1.value == 1u64, 1004);

        let field_key2 = *vector::borrow(&field_keys, 1);
        std::debug::print(&field_key2);

        let field2 = native_borrow_field<DynamicField<vector<u8>, u64>>(obj.id, field_key2);
        assert!(field2.name == b"key2", 1005);
        assert!(field2.value == 2u64, 1006);

        let TestStruct{ count: _} = drop_unchecked(obj);
    }

    #[test]
    fun test_object_id_to_string() {
        let path = vector::empty<address>();
        vector::push_back(&mut path, @0x1);
        vector::push_back(&mut path, @0xa7afe75c4f3a7631191905601f4396b25dde044539807de65ed4fc7358dbd98e);

        let id = ObjectID { path };
        let str = to_string(&id);
        // Expected: "0x0000000000000000000000000000000000000000000000000000000000000001a7afe75c4f3a7631191905601f4396b25dde044539807de65ed4fc7358dbd98e"
        assert!(str == string::utf8(b"0x0000000000000000000000000000000000000000000000000000000000000001a7afe75c4f3a7631191905601f4396b25dde044539807de65ed4fc7358dbd98e"), 1000);
        let from_id = from_string(&str);
        assert!(id == from_id, 1001);

        let id2 = ObjectID{path: vector[@0x1234]};
        let str2 = to_string(&id2);
        let from_id2 = from_string(&str2);
        assert!(id2 == from_id2, 1002);
    }

    #[test]
    fun test_object_id_from_string() {
        // test empty string (root)
        let root = from_string(&string::utf8(b""));
        assert!(vector::is_empty(&root.path), 1010);

        // test with "0x" prefix
        // let id1 = from_string(&string::utf8(b"0x1234"));
        let id1 = from_string(&string::utf8(b"0x0000000000000000000000000000000000000000000000000000000000001234"));
        assert!(vector::length(&id1.path) == 1, 1011);

        // test without prefix
        let id2 = from_string(&string::utf8(b"1234"));
        assert!(vector::length(&id2.path) == 1, 1012);
    }

    #[test]
    fun test_remove_field_and_contains(){
        let obj = new(TestStruct { count: 1 });
        add_field(&mut obj, b"key", 1u64);
        assert!(contains_field(&obj, b"key"), 1000);
        let _v:u64 = remove_field(&mut obj, b"key");
        assert!(!contains_field(&obj, b"key"), 1001);
        let TestStruct{ count: _} = remove(obj);
    }
}

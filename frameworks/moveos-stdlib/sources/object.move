// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// Move Object
/// For more details, please refer to https://rooch.network/docs/developer-guides/object
module moveos_std::object {
    use std::hash;
    use std::vector;
    use moveos_std::signer;
    use moveos_std::tx_context;
    use moveos_std::bcs;
    use moveos_std::type_info;
    use moveos_std::address;

    friend moveos_std::account;
    friend moveos_std::module_store;
    friend moveos_std::event;
    friend moveos_std::table;
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

    const SYSTEM_OWNER_ADDRESS: address = @0x0;

    const SHARED_OBJECT_FLAG_MASK: u8 = 1;
    const FROZEN_OBJECT_FLAG_MASK: u8 = 1 << 1;
    const BOUND_OBJECT_FLAG_MASK: u8 = 1 << 2;

    const SPARSE_MERKLE_PLACEHOLDER_HASH: address = @0x5350415253455f4d45524b4c455f504c414345484f4c4445525f484153480000;

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

    public fun custom_object_id<ID: drop, T>(id: ID): ObjectID {
        custom_child_object_id<ID, T,>(root_object_id(), id)
    }

    public fun custom_child_object_id<ID: drop, T>(parent_id: ObjectID, id: ID): ObjectID {
        let bytes = bcs::to_bytes(&id);
        vector::append(&mut bytes, *std::string::bytes(&type_info::type_name<T>()));
        let hash = hash::sha3_256(bytes);
        let child = address::from_bytes(hash);
        let path = parent_id.path;
        vector::push_back(&mut path, child);
        ObjectID { path } 
    }

    struct Root has key {
        // Move VM will auto add a bool field to the empty struct
        // So we manually add a bool field to the struct
        _placeholder: bool,
    }

    /// ObjectEntity<T> is a box of the value of T
    /// It does not have any ability, so it can not be `drop`, `copy`, or `store`, and can only be handled by storage API after creation.
    struct ObjectEntity<T> {
        // The object id
        id: ObjectID,
        // The owner of the object
        owner: address,
        /// A flag to indicate whether the object is shared or frozen
        flag: u8,
        // Fields SMT root
        state_root: address,
        // Fields size, number of items
        size: u64,
        // The object created timestamp on chain
        created_at: u64,
        // The object updated timestamp on chain
        updated_at: u64,

        // The value of the object
        // The value must be the last field
        value: T,
    }

    /// Object<T> is a pointer to the ObjectEntity<T>, It has `key` and `store` ability. 
    /// It has the same lifetime as the ObjectEntity<T>
    /// Developers only need to use Object<T> related APIs and do not need to know the ObjectEntity<T>.
    struct Object<phantom T> has key, store {
        id: ObjectID,
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
    public fun new_with_id<ID: drop, T: key>(id: ID, value: T): Object<T> {
        let id = custom_object_id<ID, T>(id);
        new_with_object_id(id, value)
    }


    fun derive_object_id(): ObjectID {
        address_to_object_id(tx_context::fresh_address())
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

    fun derive_child_object_id(parent: &ObjectID): ObjectID {
        let path = parent.path;
        vector::push_back(&mut path, tx_context::fresh_address());
        ObjectID { path }
    }

    public(friend) fun new_with_object_id<T: key>(id: ObjectID, value: T): Object<T> {
        add_object_field_internal<Root, T>(root_object_id(), id, value)
    }

    fun new_internal<T: key>(id: ObjectID, value: T): ObjectEntity<T> {
        let owner = SYSTEM_OWNER_ADDRESS;

        ObjectEntity<T> {
            id,
            owner,
            flag: 0u8,
            state_root: SPARSE_MERKLE_PLACEHOLDER_HASH,
            size: 0,
            created_at: 0,
            updated_at: 0,
            value,
        }
    }

    /// Borrow the object value
    public fun borrow<T: key>(self: &Object<T>): &T {
        let obj_enitty = borrow_from_global<T>(self.id);
        &obj_enitty.value
    }

    /// Borrow the object mutable value
    public fun borrow_mut<T: key>(self: &mut Object<T>): &mut T {
        let obj_entity = borrow_mut_from_global<T>(self.id);
        &mut obj_entity.value
    }

    /// Check if the object with `object_id` exists in the global object storage
    public fun exists_object(object_id: ObjectID): bool {
        contains_field_internal(parent_id(&object_id), object_id)
    }

    /// Check if the object exists in the global object storage and the type of the object is `T`
    public fun exists_object_with_type<T: key>(object_id: ObjectID): bool {
        contains_object_field_internal<T>(parent_id(&object_id), object_id)
    }

    /// Borrow Object from object store by object_id
    /// Any one can borrow an `&Object<T>` from the global object storage
    public fun borrow_object<T: key>(object_id: ObjectID): &Object<T> {
        let object_entity = borrow_from_global<T>(object_id);
        as_ref(object_entity)
    }

    /// Borrow mut Object by `owner` and `object_id`
    public fun borrow_mut_object<T: key>(owner: &signer, object_id: ObjectID): &mut Object<T> {
        let owner_address = signer::address_of(owner);
        let obj = borrow_mut_object_internal<T>(object_id);
        assert!(owner(obj) == owner_address, ErrorObjectOwnerNotMatch);
        obj
    }

    #[private_generics(T)]
    /// Borrow mut Object by `object_id`
    public fun borrow_mut_object_extend<T: key>(object_id: ObjectID): &mut Object<T> {
        let obj = borrow_mut_object_internal<T>(object_id);
        obj
    }

    fun borrow_mut_object_internal<T: key>(object_id: ObjectID): &mut Object<T> {
        let object_entity = borrow_mut_from_global<T>(object_id);
        let obj = as_mut_ref(object_entity);
        obj
    }

    /// Take out the UserOwnedObject by `owner` and `object_id`
    /// The `T` must have `key + store` ability.
    /// Note: When the Object is taken out, the Object will auto become `SystemOwned` Object.
    public fun take_object<T: key + store>(owner: &signer, object_id: ObjectID): Object<T> {
        let owner_address = signer::address_of(owner);
        let object_entity = borrow_mut_from_global<T>(object_id);
        assert!(owner_internal(object_entity) == owner_address, ErrorObjectOwnerNotMatch);
        assert!(!is_bound_internal(object_entity), ErrorObjectIsBound);
        to_system_owned_internal(object_entity);
        mut_entity_as_object(object_entity)
    }

    #[private_generics(T)]
    /// Take out the UserOwnedObject by `object_id`, return the owner and Object
    /// This function is for developer to extend, Only the module of `T` can take out the `UserOwnedObject` with object_id.
    public fun take_object_extend<T: key>(object_id: ObjectID): (address, Object<T>) {
        let object_entity = borrow_mut_from_global<T>(object_id);
        assert!(is_user_owned_internal(object_entity), ErrorObjectOwnerNotMatch);
        assert!(!is_bound_internal(object_entity), ErrorObjectIsBound);
        let owner = owner_internal(object_entity);
        to_system_owned_internal(object_entity);
        (owner, mut_entity_as_object(object_entity))
    }

    /// Borrow mut Shared Object by object_id
    public fun borrow_mut_object_shared<T: key>(object_id: ObjectID): &mut Object<T> {
        let obj = borrow_mut_object_internal<T>(object_id);
        assert!(is_shared(obj), ErrorObjectNotShared);
        obj
    }


    #[private_generics(T)]
    /// Remove the object from the global storage, and return the object value
    /// This function is only can be called by the module of `T`.
    /// The caller must ensure that the dynamic fields are empty before delete the Object
    public fun remove<T: key>(self: Object<T>): T {
        let Object{id} = self;
        // Currently, we only support to remove the object from the root object
        // If we want to remove the child object, we need to call the `remove_object_field` function
        remove_object_field_internal<Root, T>(root_object_id(), id, true)
    }

    /// Remove the object from the global storage, and return the object value
    /// Do not check if the dynamic fields are empty 
    public(friend) fun remove_unchecked<T: key>(self: Object<T>): T {
        let Object{id} = self;
        remove_object_field_internal<Root, T>(root_object_id(), id, false)
    }

    /// Directly drop the Object
    fun drop<T: key>(self: Object<T>) {
        let Object { id: _ } = self;
    }

    fun drop_entity<T: key>(entity: ObjectEntity<T>): T {
        let ObjectEntity { id: _, owner: _, flag: _, state_root: _, size: _, created_at: _, updated_at: _, value } = entity;
        value
    }

    /// Make the Object shared, Any one can get the &mut Object<T> from shared object
    /// The shared object also can be removed from the object storage.
    public fun to_shared<T: key>(self: Object<T>) {
        let obj_entity = borrow_mut_from_global<T>(self.id);
        to_shared_internal(obj_entity);
        drop(self);
    }

    fun to_shared_internal<T: key>(self: &mut ObjectEntity<T>) {
        self.flag = self.flag | SHARED_OBJECT_FLAG_MASK;
        to_system_owned_internal(self);
    }

    public fun is_shared<T: key>(self: &Object<T>): bool {
        let obj_enitty = borrow_from_global<T>(self.id);
        is_shared_internal(obj_enitty)
    }

    fun is_shared_internal<T>(self: &ObjectEntity<T>): bool {
        self.flag & SHARED_OBJECT_FLAG_MASK == SHARED_OBJECT_FLAG_MASK
    }

    /// Make the Object frozen, Any one can not get the &mut Object<T> from frozen object
    public fun to_frozen<T: key>(self: Object<T>) {
        let obj_entity = borrow_mut_from_global<T>(self.id);
        to_frozen_internal(obj_entity);
        drop(self);
    }

    fun to_frozen_internal<T: key>(self: &mut ObjectEntity<T>) {
        self.flag = self.flag | FROZEN_OBJECT_FLAG_MASK;
        to_system_owned_internal(self);
    }

    public fun is_frozen<T: key>(self: &Object<T>): bool {
        let obj_enitty = borrow_from_global<T>(self.id);
        is_frozen_internal(obj_enitty)
    }

    fun is_frozen_internal<T>(self: &ObjectEntity<T>): bool {
        self.flag & FROZEN_OBJECT_FLAG_MASK == FROZEN_OBJECT_FLAG_MASK
    }

    //TODO how to provide public bound object API

    fun to_bound_internal<T>(self: &mut ObjectEntity<T>) {
        self.flag = self.flag | BOUND_OBJECT_FLAG_MASK;
    }

    public fun is_bound<T: key>(self: &Object<T>): bool {
        let obj_enitty = borrow_from_global<T>(self.id);
        is_bound_internal(obj_enitty)
    }

    public(friend) fun is_bound_internal<T>(self: &ObjectEntity<T>): bool {
        self.flag & BOUND_OBJECT_FLAG_MASK == BOUND_OBJECT_FLAG_MASK
    }

    public(friend) fun to_user_owned<T: key>(self: &mut Object<T>, new_owner: address) {
        assert!(new_owner != SYSTEM_OWNER_ADDRESS, ErrorInvalidOwnerAddress);
        let obj_entity = borrow_mut_from_global<T>(self.id);
        obj_entity.owner = new_owner;
    }

    public(friend) fun to_system_owned<T: key>(self: &mut Object<T>) {
        let obj_entity = borrow_mut_from_global<T>(self.id);
        to_system_owned_internal(obj_entity);
    }

    public(friend) fun to_system_owned_internal<T>(self: &mut ObjectEntity<T>) {
        self.owner = SYSTEM_OWNER_ADDRESS;
    }

    /// Transfer the object to the new owner
    /// Only the `T` with `store` can be directly transferred.
    public fun transfer<T: key + store>(self: Object<T>, new_owner: address) {
        to_user_owned(&mut self, new_owner);
        drop(self);
    }

    #[private_generics(T)]
    /// Transfer the object to the new owner
    /// This function is for the module of `T` to extend the `transfer` function.
    public fun transfer_extend<T: key>(self: Object<T>, new_owner: address) {
        to_user_owned(&mut self, new_owner);
        drop(self);
    }

    public fun id<T>(self: &Object<T>): ObjectID {
        self.id
    }

    public fun owner<T: key>(self: &Object<T>): address {
        let obj_enitty = borrow_from_global<T>(self.id);
        obj_enitty.owner
    }

    public(friend) fun owner_internal<T: key>(self: &ObjectEntity<T>): address {
        self.owner
    }

    public fun is_system_owned<T: key>(self: &Object<T>): bool {
        owner(self) == SYSTEM_OWNER_ADDRESS
    }

    public(friend) fun is_user_owned_internal<T: key>(self: &ObjectEntity<T>): bool {
        owner_internal(self) != SYSTEM_OWNER_ADDRESS
    }

    public fun is_user_owned<T: key>(self: &Object<T>): bool {
        owner(self) != SYSTEM_OWNER_ADDRESS
    }

    // === Object Ref ===

    public(friend) fun as_ref<T: key>(object_entity: &ObjectEntity<T>): &Object<T> {
        as_ref_inner<Object<T>>(object_entity.id)
    }

    public(friend) fun as_mut_ref<T: key>(object_entity: &mut ObjectEntity<T>): &mut Object<T> {
        as_mut_ref_inner<Object<T>>(object_entity.id)
    }

    public(friend) fun mut_entity_as_object<T: key>(object_entity: &mut ObjectEntity<T>): Object<T> {
        Object { id: object_entity.id }
    }

    /// Convert the ObjectID to &T or &mut T
    /// The caller must ensure the T only has one `ObjectID` field, such as `Object<T>`.
    native fun as_ref_inner<T>(object_id: ObjectID): &T;

    native fun as_mut_ref_inner<T>(object_id: ObjectID): &mut T;

    // === Object Storage ===


    /// The global root object id is `[]`
    fun root_object_id(): ObjectID {
        ObjectID { path: vector::empty() }
    }

    fun borrow_from_global<T: key>(object_id: ObjectID): &ObjectEntity<T> {
        borrow_object_field_internal<T>(parent_id(&object_id), object_id)
    }

    fun borrow_mut_from_global<T: key>(object_id: ObjectID): &mut ObjectEntity<T> {
        borrow_mut_object_field_internal<T>(parent_id(&object_id), object_id)
    }

    // === Object Raw Dynamic Fields ===

    #[private_generics(T)]
    /// Add a dynamic field to the object. Aborts if an field for this
    /// key already exists. The field itself is not stored in the
    /// object, and cannot be discovered from it.
    public fun add_field<T: key, K: copy + drop, V: store>(obj: &mut Object<T>, key: K, val: V) {
        add_field_internal<T, K, V>(obj.id, key, val);
    }

    // Add field and wrap the value to FieldValue
    public(friend) fun add_field_internal<T: key, K: copy + drop, V>(obj_id: ObjectID, key: K, val: V) {
        native_add_field<K, FieldValue<V>>(obj_id, key, FieldValue{val});
        increment_size<T>(obj_id);
        update_timestamp<T>(obj_id);
    }

    #[private_generics(T, V)]
    /// Add a object field to the object. return the child object
    /// The parent object must be a shared object
    public fun add_object_field<T: key, V: key>(obj: &mut Object<T>, v: V): Object<V> {
        // Only shared object can add child object
        assert!(is_shared(obj), ErrorObjectNotShared);
        // Currently, the child object level is limited to 2
        assert!(vector::length(&obj.id.path) < 2, ErrorChildObjectTooDeep);
        let child_id = derive_child_object_id(&obj.id);
        add_object_field_internal<T, V>(obj.id, child_id, v)
    }

    #[private_generics(T, V)]
    /// Add a object field to the object with custom ID. return the child object
    /// The child ObjectID can be generated via the `custom_child_object_id` function
    public fun add_object_field_with_id<T: key, ID:drop, V: key>(obj: &mut Object<T>, id: ID, v: V): Object<V> {
        // Only shared object can add child object
        assert!(is_shared(obj), ErrorObjectNotShared);
        // Currently, the child object level is limited to 2
        assert!(vector::length(&obj.id.path) < 2, ErrorChildObjectTooDeep);
        let child_id = custom_child_object_id<ID, V>(obj.id, id);
        add_object_field_internal<T, V>(obj.id, child_id, v)
    }


    fun add_object_field_internal<T: key, V: key>(parent_id: ObjectID, child_id: ObjectID, v: V): Object<V> {
        let child_entity = new_internal(child_id, v);
        init_timestamp(&mut child_entity);
        native_add_field<ObjectID, ObjectEntity<V>>(parent_id, child_id, child_entity);
        increment_size<T>(parent_id);
        // init_timestamp(&mut child_entity);
        update_timestamp<T>(parent_id);
        Object { id: child_id }
    }

    /// Acquire an immutable reference to the value which `key` maps to.
    /// Aborts if there is no field for `key`.
    public fun borrow_field<T: key, K: copy + drop, V: store>(obj: &Object<T>, key: K): &V {
        borrow_field_internal<K, V>(obj.id, key)
    }

    /// Borrow the child object by `key`
    public fun borrow_object_field<T: key, V: key>(obj: &Object<T>, key: ObjectID): &Object<V> {
        let object_entity = borrow_object_field_internal<V>(obj.id, key);
        as_ref(object_entity)
    }

    fun borrow_object_field_internal<V: key>(parent_id: ObjectID, key: ObjectID): &ObjectEntity<V> {
        assert!(is_parent(&parent_id, &key), ErrorParentNotMatch);
        native_borrow_field<ObjectID, ObjectEntity<V>>(parent_id, key)
    }

    /// Borrow FieldValue and return the val of FieldValue
    public(friend) fun borrow_field_internal<K: copy + drop, V>(obj_id: ObjectID, key: K): &V {
        &native_borrow_field<K, FieldValue<V>>(obj_id, key).val
    }

    /// Acquire an immutable reference to the value which `key` maps to.
    /// Returns specified default value if there is no field for `key`.
    public fun borrow_field_with_default<T: key, K: copy + drop, V: store>(obj: &Object<T>, key: K, default: &V): &V {
        borrow_field_with_default_internal<K, V>(obj.id, key, default)
    }

    /// Acquire an immutable reference to the value which `key` maps to.
    /// Returns specified default value if there is no field for `key`.
    fun borrow_field_with_default_internal<K: copy + drop, V>(obj_id: ObjectID, key: K, default: &V): &V {
        if (!contains_field_internal<K>(obj_id, key)) {
            default
        } else {
            borrow_field_internal(obj_id, key)
        }
    }

    #[private_generics(T)]
    /// Acquire a mutable reference to the value which `key` maps to.
    /// Aborts if there is no field for `key`.
    public fun borrow_mut_field<T: key, K: copy + drop, V: store>(obj: &mut Object<T>, key: K): &mut V {
        borrow_mut_field_internal<K, V>(obj.id, key)
    }

    /// Acquire a mutable reference to the value which `key` maps to.
    /// Aborts if there is no field for `key`.
    public(friend) fun borrow_mut_field_internal<K: copy + drop, V>(obj_id: ObjectID, key: K): &mut V {
        &mut native_borrow_mut_field<K, FieldValue<V>>(obj_id, key).val
    }

    #[private_generics(T)]
    /// Acquire a mutable reference to the value which `key` maps to.
    /// Insert the pair (`key`, `default`) first if there is no field for `key`.
    public fun borrow_mut_field_with_default<T: key, K: copy + drop, V: store + drop>(
        obj: &mut Object<T>,
        key: K,
        default: V
    ): &mut V {
        borrow_mut_field_with_default_internal<T, K, V>(obj.id, key, default)
    }

    fun borrow_mut_field_with_default_internal<T: key, K: copy + drop, V: drop>(
        obj_id: ObjectID,
        key: K,
        default: V
    ): &mut V {
        if (!contains_field_internal<K>(obj_id, copy key)) {
            add_field_internal<T, K, V>(obj_id, key, default)
        };
        borrow_mut_field_internal(obj_id, key)
    }

    /// Borrow the child object by `key`
    /// Because the parent object must be a shared object, so we do not require the #[private_generics(T)] here
    public fun borrow_mut_object_field<T: key, V: key>(obj: &mut Object<T>, key: ObjectID): &mut Object<V> {
        let object_entity = borrow_mut_object_field_internal<V>(obj.id, key);
        as_mut_ref(object_entity)
    }

    fun borrow_mut_object_field_internal<V: key>(parent_id: ObjectID, key: ObjectID): &mut ObjectEntity<V> {
        assert!(is_parent(&parent_id, &key), ErrorParentNotMatch);
        let object_entity = native_borrow_mut_field<ObjectID, ObjectEntity<V>>(parent_id, key);
        assert!(!is_frozen_internal(object_entity), ErrorObjectFrozen);
        object_entity
    }

    #[private_generics(T)]
    /// Insert the pair (`key`, `value`) if there is no field for `key`.
    /// update the value of the field for `key` to `value` otherwise
    public fun upsert_field<T: key, K: copy + drop, V: store + drop>(obj: &mut Object<T>, key: K, value: V) {
        upsert_field_internal<T, K, V>(obj.id, key, value)
    }

    fun upsert_field_internal<T: key, K: copy + drop, V: drop>(obj_id: ObjectID, key: K, value: V) {
        if (!contains_field_internal<K>(obj_id, copy key)) {
            add_field_internal<T, K, V>(obj_id, key, value)
        } else {
            let ref = borrow_mut_field_internal(obj_id, key);
            *ref = value;
            update_timestamp<T>(obj_id);
        };
    }

    #[private_generics(T)]
    /// Remove from `object` and return the value which `key` maps to.
    /// Aborts if there is no field for `key`.
    public fun remove_field<T: key, K: copy + drop, V: store>(obj: &mut Object<T>, key: K): V {
        remove_field_internal<T, K, V>(obj.id, key)
    }

    public(friend) fun remove_field_internal<T: key, K: copy + drop, V>(obj_id: ObjectID, key: K): V {
        let FieldValue { val } = native_remove_field<K, FieldValue<V>>(obj_id, key);
        decreases_size<T>(obj_id);
        update_timestamp<T>(obj_id);
        val
    }

    fun increment_size<T: key>(obj_id: ObjectID) {
        if(has_parent(&obj_id)) {
            let object_entity = borrow_mut_from_global<T>(obj_id);
            object_entity.size = object_entity.size + 1;
        }else{
            let root = native_borrow_root();
            root.size = root.size + 1;
        }
    }

    fun decreases_size<T: key>(obj_id: ObjectID) {
        if(has_parent(&obj_id)) {
            let object_entity = borrow_mut_from_global<T>(obj_id);
            object_entity.size = object_entity.size - 1;
        }else{
            let root = native_borrow_root();
            root.size = root.size - 1;
        }
    }

    fun init_timestamp<T: key>(entity: &mut ObjectEntity<T>) {
        let now_milliseconds = now_milliseconds();
        entity.created_at = now_milliseconds;
        entity.updated_at = now_milliseconds;
    }

    fun update_timestamp<T: key>(obj_id: ObjectID) {
        let now_milliseconds = now_milliseconds();
        if(has_parent(&obj_id)) {
            let object_entity = borrow_mut_from_global<T>(obj_id);
            object_entity.updated_at = now_milliseconds;
        }else{
            let root = native_borrow_root();
            root.updated_at = now_milliseconds;
        }
    }

    #[private_generics(T)]
    public fun remove_object_field<T: key, V: key>(obj: &mut Object<T>, child: Object<V>): V {
        let Object { id: child_id } = child;
        remove_object_field_internal<T, V>(obj.id, child_id, true)
    }

    fun remove_object_field_internal<T: key, V: key>(parent_id: ObjectID, child_id: ObjectID, check_size: bool): V {
        assert!(is_parent(&parent_id, &child_id), ErrorParentNotMatch);
        let object_entity = native_remove_field<ObjectID, ObjectEntity<V>>(parent_id, child_id);
        let ObjectEntity { id: _, owner: _, flag: _, value, state_root: _, size: size, created_at: _, updated_at: _ } = object_entity;
        if (check_size) {
            // Need to ensure that the Fields is empty before delete the Object
            assert!(size == 0, ErrorFieldsNotEmpty);
        };
        decreases_size<T>(parent_id);
        update_timestamp<T>(parent_id);
        value
    }

   
    /// Returns true if `object` contains an field for `key`, include normal field and object field
    public fun contains_field<T: key, K: copy + drop>(obj: &Object<T>, key: K): bool {
        contains_field_internal<K>(obj.id, key)
    }

    public(friend) fun contains_field_internal<K: copy + drop>(obj_id: ObjectID, key: K): bool {
        native_contains_field<K>(obj_id, key)
    }

    /// Returns true if `object` contains an field for `key` and the value type is `V`. only for normal field
    public fun contains_field_with_type<T: key, K: copy + drop, V: store>(obj: &Object<T>, key: K): bool {
        contains_field_with_value_type_internal<K, V>(obj.id, key)
    }

    fun contains_field_with_value_type_internal<K: copy + drop, V>(obj_id: ObjectID, key: K): bool {
        native_contains_field_with_value_type<K, FieldValue<V>>(obj_id, key)
    }

    /// Returns true if `object` contains an Object field for `key` and the value type is `V`.
    public fun contains_object_field<T: key, V: key>(obj: &Object<T>, key: ObjectID): bool {
        contains_object_field_internal<V>(obj.id, key)
    }

    fun contains_object_field_internal<V: key>(parent: ObjectID, key: ObjectID): bool {
        if (is_parent(&parent, &key)) {
            native_contains_field_with_value_type<ObjectID, ObjectEntity<V>>(parent, key)
        }else {
            false
        }
    }

    /// Returns the size of the object fields, the number of key-value pairs
    public fun field_size<T: key>(obj: &Object<T>): u64 {
        field_size_internal<T>(obj.id)
    }

    fun field_size_internal<T: key>(object_id: ObjectID): u64 {
        let object_entity = borrow_from_global<T>(object_id);
        object_entity.size
    }
    // ======================================================================================================
    // Internal API

    /// Wrapper for file values. Required for making values appear as struct in the implementation.
    /// Because the GlobalValue in MoveVM must be a struct.
    struct FieldValue<V> has key, drop, store {
        val: V
    }

    // === Timestamp store ===
    // Limited by Move's circular dependency restrictions,
    // Timestamp Struct definition and store are placed in Object Module

    /// A object holding the current Unix time in milliseconds
    struct Timestamp has key {
        milliseconds: u64,
    }

    /// Conversion factor between seconds and milliseconds
    const MILLI_CONVERSION_FACTOR: u64 = 1000;

    /// An invalid timestamp was provided
    const ErrorInvalidTimestamp: u64 = 21;
    const ErrorNotGenesisAddress: u64 = 22;

    public(friend) fun genesis_init(_genesis_account: &signer, initial_time_milliseconds: u64) {
        let timestamp_id = named_object_id<Timestamp>();
        // The Timestamp object will initialize before the genesis.
        if (!exists_object(timestamp_id)) {
            let timestamp = Timestamp { milliseconds: initial_time_milliseconds };
            let obj = new_named_object(timestamp);
            transfer_extend(obj, @moveos_std);
        } else {
            update_global_time(initial_time_milliseconds)
        }
    }

    /// Updates the global clock time, if the new time is smaller than the current time, aborts.
    public(friend) fun update_global_time(timestamp_milliseconds: u64) {
        let current_timestamp = timestamp_mut();
        let now = current_timestamp.milliseconds;
        assert!(now <= timestamp_milliseconds, ErrorInvalidTimestamp);
        current_timestamp.milliseconds = timestamp_milliseconds;
    }

    public(friend) fun try_update_global_time_internal(timestamp_milliseconds: u64) : bool {
        let current_timestamp = timestamp_mut();
        let now = current_timestamp.milliseconds;
        if(now <= timestamp_milliseconds) {
            current_timestamp.milliseconds = timestamp_milliseconds;
            true
        }else{
            false
        }
    }

    fun timestamp_mut(): &mut Timestamp {
        let object_id = named_object_id<Timestamp>();
        let obj = borrow_mut_object_extend<Timestamp>(object_id);
        borrow_mut(obj)
    }

    public(friend) fun timestamp(): &Timestamp {
        let object_id = named_object_id<Timestamp>();
        let obj = borrow_object<Timestamp>(object_id);
        borrow(obj)
    }

    public(friend) fun milliseconds(self: &Timestamp): u64 {
        self.milliseconds
    }

    public(friend) fun seconds(self: &Timestamp): u64 {
        self.milliseconds / MILLI_CONVERSION_FACTOR
    }

    /// Gets the current time in milliseconds.
    public(friend) fun now_milliseconds(): u64 {
        milliseconds(timestamp())
    }

    /// Gets the current time in seconds.
    public(friend) fun now_seconds(): u64 {
        now_milliseconds() / MILLI_CONVERSION_FACTOR
    }

    native fun native_borrow_root(): &mut ObjectEntity<Root>;

    native fun native_add_field<K: copy + drop, V>(obj_id: ObjectID, key: K, val: V);

    native fun native_borrow_field<K: copy + drop, V>(obj_id: ObjectID, key: K): &V;

    native fun native_borrow_mut_field<K: copy + drop, V>(obj_id: ObjectID, key: K): &mut V;

    native fun native_contains_field<K: copy + drop>(obj_id: ObjectID, key: K): bool;

    /// If the Object contains a field for `key` with value type `V`.
    native fun native_contains_field_with_value_type<K: copy + drop, V>(obj_id: ObjectID, key: K): bool;

    native fun native_remove_field<K: copy + drop, V>(obj_id: ObjectID, key: K): V;

    #[test_only]
    /// Testing only: allows to drop a Object even if it's fields is not empty.
    public fun drop_unchecked<T: key>(self: Object<T>): T {
        remove_unchecked(self)
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
        assert!(exists_object(obj.id), 1000);
        {
            to_user_owned(&mut obj, sender_addr);
            assert!(owner(&obj) == sender_addr, 1001);
        };
        {
            let test_struct_mut = borrow_mut(&mut obj);
            test_struct_mut.count = test_struct_mut.count + 1;
        };
        {
            let test_struct_ref = borrow(&obj);
            assert!(test_struct_ref.count == init_count + 1, 1002);
        };
        {
            to_user_owned(&mut obj, @0x10);
            assert!(owner(&obj) != sender_addr, 1003);
        };

        let test_obj = remove(obj);
        let TestStruct { count: _count } = test_obj;
    }

    #[test]
    fun test_shared() {
        let object_id = derive_object_id();
        let obj_enitty = new_internal(object_id, TestStruct { count: 1 });
        assert!(!is_shared_internal(&obj_enitty), 1000);
        assert!(!is_frozen_internal(&obj_enitty), 1001);
        to_shared_internal(&mut obj_enitty);
        assert!(is_shared_internal(&obj_enitty), 1002);
        assert!(!is_frozen_internal(&obj_enitty), 1003);
        let TestStruct { count: _ } = drop_entity(obj_enitty);
    }

    #[test]
    fun test_frozen() {
        let object_id = derive_object_id();
        let obj_enitty = new_internal(object_id, TestStruct { count: 1 });
        assert!(!is_shared_internal(&obj_enitty), 1000);
        assert!(!is_frozen_internal(&obj_enitty), 1001);
        to_frozen_internal(&mut obj_enitty);
        assert!(!is_shared_internal(&obj_enitty), 1002);
        assert!(is_frozen_internal(&obj_enitty), 1003);
        let TestStruct { count: _ } = drop_entity(obj_enitty);
    }

    // An object can not be shared and frozen at the same time
    // This test just ensure the flag can be set at the same time
    #[test]
    fun test_all_flag() {
        let object_id = derive_object_id();
        let obj_enitty = new_internal(object_id, TestStruct { count: 1 });
        assert!(!is_shared_internal(&obj_enitty), 1000);
        assert!(!is_frozen_internal(&obj_enitty), 1001);
        to_shared_internal(&mut obj_enitty);
        to_frozen_internal(&mut obj_enitty);
        assert!(is_shared_internal(&obj_enitty), 1002);
        assert!(is_frozen_internal(&obj_enitty), 1003);
        let TestStruct { count: _ } = drop_entity(obj_enitty);
    }

    #[test]
    #[expected_failure(abort_code = ErrorNotFound, location = moveos_std::object)]
    fun test_borrow_not_exist_failure() {
        let obj = new(TestStruct { count: 1 });
        let object_id = obj.id;
        let TestStruct { count : _ } = remove(obj);
        let _obj_ref = borrow_from_global<TestStruct>(object_id);
    }

    #[test]
    #[expected_failure(abort_code = ErrorNotFound, location = moveos_std::object)]
    fun test_double_remove_failure() {
        let obj = new(TestStruct { count: 1 });
        let obj_id = id(&obj);

        let TestStruct { count : _ } = remove_object_field_internal<Root, TestStruct>(root_object_id(), obj_id, true);
        let TestStruct { count : _ } = remove(obj);
    }

    #[test]
    #[expected_failure(abort_code = ErrorTypeMismatch, location = moveos_std::object)]
    fun test_type_mismatch() {
        let object_id = derive_object_id();
        let obj = new_with_object_id(object_id, TestStruct { count: 1 });
        {
            let test_struct_ref = borrow(&obj);
            assert!(test_struct_ref.count == 1, 1001);
        };
        {
            let test_struct2_object_entity = borrow_from_global<TestStruct2>(object_id);
            assert!(test_struct2_object_entity.value.count == 1, 1002);
        };
        drop(obj);
    }

    struct TestStructID has store, copy, drop {
        id: u64,
    }

    #[test]
    fun test_custom_object_id() {
        let id = TestStructID { id: 1 };
        let object_id = custom_object_id<TestStructID, TestStruct>(id);
        //ensure the object_id is the same as the object_id generated by the object.rs
        assert!(object_id.path == vector[@0xaa825038ae811f5c94d20175699d808eae4c624fa85c81faad45de1145284e06], 1);
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
        let child = add_object_field(parent, TestStruct { count: 2 });
        let TestStruct { count: _ } = remove_object_field(parent, child);
    }

    #[test]
    fun test_child_field(){
        let parent = new(TestStruct { count: 1 });
        let parent_id = id(&parent);
        to_shared(parent);
        let parent = borrow_mut_object_shared<TestStruct>(parent_id);
        let child = add_object_field(parent, TestStruct { count: 2 });
        add_field(&mut child, b"key", 1u64);
        {
            let v = borrow_mut_field(&mut child, b"key");
            *v = 2u64;
        };
        assert!(*borrow_field(&child, b"key") == 2u64, 1000);
        let _v:u64 = remove_field(&mut child, b"key");
        let TestStruct { count: _ } = remove_object_field(parent, child);
    }

    #[test]
    #[expected_failure(abort_code = ErrorObjectNotShared, location = moveos_std::object)]
    fun test_parent_not_shared(){
        let parent = new(TestStruct { count: 1 });
        //let parent_id = id(&parent);
        let child = add_object_field(&mut parent, TestStruct { count: 2 });
        let TestStruct { count: _ } = remove_object_field(&mut parent, child);
        let TestStruct{ count: _} = remove(parent);
    }

    #[test]
    #[expected_failure(abort_code = ErrorChildObjectTooDeep, location = moveos_std::object)]
    fun test_child_too_deep(){
        let parent = new(TestStruct { count: 1 });
        let parent_id = id(&parent);
        to_shared(parent);
        let parent = borrow_mut_object_shared<TestStruct>(parent_id);
        let child = add_object_field(parent, TestStruct { count: 2 });
        let child_id = id(&child);
        to_shared(child);
        let child = borrow_mut_object_shared<TestStruct>(child_id);
        let grand_child = add_object_field(child, TestStruct { count: 3 });
        let TestStruct { count: _ } = remove_object_field(child, grand_child);
    }

    #[test]
    fun test_child_field_upsert(){
        let parent = new(TestStruct { count: 1 });
        let parent_id = id(&parent);
        to_shared(parent);
        let parent = borrow_mut_object_shared<TestStruct>(parent_id);
        let child = add_object_field(parent, TestStruct { count: 2 });
        upsert_field(&mut child, b"key", 1u64);
        assert!(*borrow_field(&child, b"key") == 1u64, 1000);
        upsert_field(&mut child, b"key", 2u64);
        assert!(*borrow_field(&child, b"key") == 2u64, 1000);
        let _v:u64 = remove_field(&mut child, b"key");
        let TestStruct { count: _ } = remove_object_field(parent, child);
    }
}

// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// Move Object
/// For more details, please refer to https://rooch.network/docs/developer-guides/object
module moveos_std::object {
    use moveos_std::signer;
    use moveos_std::object_id;
    use moveos_std::object_id::{ObjectID, TypedUID, address_to_object_id};
    use moveos_std::raw_table::TableInfo;
    use moveos_std::raw_table;
    #[test_only]
    use moveos_std::object_id::{custom_object_id, new_uid, UID};

    friend moveos_std::context;
    friend moveos_std::account;
    friend moveos_std::move_module;
    friend moveos_std::storage_context;
    friend moveos_std::event;
    friend moveos_std::table;
    friend moveos_std::type_table;
    friend moveos_std::object_table;
    friend moveos_std::object_dynamic_field;

    const ErrorObjectAlreadyExist: u64 = 1;
    const ErrorObjectFrozen: u64 = 2;
    const ErrorInvalidOwnerAddress:u64 = 3;

    const ErrorObjectOwnerNotMatch: u64 = 4;
    const ErrorObjectNotShared: u64 = 5;
    ///Can not take out the object which is bound to the account
    const ErrorObjectIsBound: u64 = 6;

    const SYSTEM_OWNER_ADDRESS: address = @0x0;
    
    const SHARED_OBJECT_FLAG_MASK: u8 = 1;
    const FROZEN_OBJECT_FLAG_MASK: u8 = 1 << 1;
    const BOUND_OBJECT_FLAG_MASK: u8 = 1 << 2;

    const SPARSE_MERKLE_PLACEHOLDER_HASH_VALUE: vector<u8> = b"SPARSE_MERKLE_PLACEHOLDER_HASH";

    /// ObjectEntity<T> is a box of the value of T
    /// It does not have any ability, so it can not be `drop`, `copy`, or `store`, and can only be handled by storage API after creation.
    struct ObjectEntity<T> {
        // The object id
        id: ObjectID,
        // The owner of the object
        owner: address,
        /// A flag to indicate whether the object is shared or frozen
        flag: u8,
        // Table SMT root
        state_root: address,
        // Table size, number of items
        size: u64,

        // The value of the object
        // The value must be the last field
        value: T,
    }

    struct TablePlaceholder has key,store {}

    /// Object<T> is a pointer to the ObjectEntity<T>, It has `key` and `store` ability. 
    /// It has the same lifetime as the ObjectEntity<T>
    /// Developers only need to use Object<T> related APIs and do not need to know the ObjectEntity<T>.
    struct Object<phantom T> has key, store {
        id: ObjectID,
    }

    #[private_generics(T)]
    /// Create a new Object, Add the Object to the global object storage and return the Object
    /// Note: the default owner is the SystemOwned Object, the caller should explicitly transfer the Object to the owner.
    public fun new<T: key>(id: TypedUID<T>, value: T): Object<T> {
        new_with_id(object_id::typed_uid_id(&id), value)
    }

    public(friend) fun new_with_id<T: key>(id: ObjectID, value: T): Object<T> {
        let obj_entity = new_internal(id, value);
        add_to_global(obj_entity);
        Object{id}
    }

    /// New pure table object
    public(friend) fun new_table_with_id(id: ObjectID): Object<TablePlaceholder> {
        let obj_entity = new_internal(id, TablePlaceholder{});
        add_to_global(obj_entity);
        Object{id}
    }

    fun new_internal<T: key>(id: ObjectID, value: T): ObjectEntity<T> {
        assert!(!contains_global(id), ErrorObjectAlreadyExist);
        let owner = SYSTEM_OWNER_ADDRESS;

        let table_info = new_table(id);
        ObjectEntity<T>{
            id,
            owner,
            flag: 0u8,
            state_root: raw_table::state_root(&table_info),
            size: raw_table::size(&table_info),
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
        // assert!(!object::is_shared(obj), ErrorObjectIsShared);
        let obj_entity = borrow_mut_from_global<T>(self.id);
        &mut obj_entity.value
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

    // // #[private_generics(T)]
    // TODO Need to tighter restrictions ?
    /// Borrow mut Shared Object by object_id
    public fun borrow_mut_object_shared<T: key>(object_id: ObjectID): &mut Object<T> {
        let obj = borrow_mut_object_internal<T>(object_id);
        assert!(is_shared(obj), ErrorObjectNotShared);
        obj
    }


    #[private_generics(T)]
    /// Remove the object from the global storage, and return the object value
    /// This function is only can be called by the module of `T`.
    public fun remove<T: key>(self: Object<T>) : T {
        let Object{id} = self;
        // Need to ensure that the Table is empty before delete the Object
        destroy_empty_table(id);
        let object_entity = remove_from_global<T>(id);
        let ObjectEntity{id:_, owner:_, flag:_, value, state_root:_, size:_} = object_entity;
        value
    }

    /// Directly drop the Object
    fun drop<T: key>(self: Object<T>) {
        let Object{id:_} = self;
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

    public fun is_shared<T: key>(self: &Object<T>) : bool {
        let obj_enitty = borrow_from_global<T>(self.id);
        is_shared_internal(obj_enitty)
    }

    fun is_shared_internal<T>(self: &ObjectEntity<T>) : bool {
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

    public fun is_frozen<T:key>(self: &Object<T>) : bool {
        let obj_enitty = borrow_from_global<T>(self.id);
        is_frozen_internal(obj_enitty)
    }

    fun is_frozen_internal<T>(self: &ObjectEntity<T>) : bool {
        self.flag & FROZEN_OBJECT_FLAG_MASK == FROZEN_OBJECT_FLAG_MASK
    }

    //TODO how to provide public bound object API

    fun to_bound_internal<T>(self: &mut ObjectEntity<T>) {
        self.flag = self.flag | BOUND_OBJECT_FLAG_MASK;
    }

    public fun is_bound<T: key>(self: &Object<T>) : bool {
        let obj_enitty = borrow_from_global<T>(self.id);
        is_bound_internal(obj_enitty)
    }
    
    public(friend) fun is_bound_internal<T>(self: &ObjectEntity<T>) : bool {
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

    public(friend) fun to_system_owned_internal<T>(self: &mut ObjectEntity<T>){
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

    public fun is_system_owned<T: key>(self: &Object<T>) : bool {
        owner(self) == SYSTEM_OWNER_ADDRESS
    } 
    
    public(friend) fun is_user_owned_internal<T: key>(self: &ObjectEntity<T>) : bool {
        owner_internal(self) != SYSTEM_OWNER_ADDRESS
    }

    public fun is_user_owned<T: key>(self: &Object<T>) : bool {
        owner(self) != SYSTEM_OWNER_ADDRESS
    }

    // === Object Ref ===

    public(friend) fun as_ref<T: key>(object_entity: &ObjectEntity<T>) : &Object<T>{
        as_ref_inner<Object<T>>(object_entity.id)
    }
    public(friend) fun as_mut_ref<T: key>(object_entity: &mut ObjectEntity<T>) : &mut Object<T>{
        as_mut_ref_inner<Object<T>>(object_entity.id)
    }
    public(friend) fun mut_entity_as_object<T: key>(object_entity: &mut ObjectEntity<T>) : Object<T> {
        Object{id: object_entity.id}
    }

    /// Convert the ObjectID to &T or &mut T
    /// The caller must ensure the T only has one `ObjectID` field, such as `Object<T>` or `Table<K,V>`, or `TypeTable`.
    native fun as_ref_inner<T>(object_id: ObjectID): &T;
    native fun as_mut_ref_inner<T>(object_id: ObjectID): &mut T;

    // === Object Storage ===

    const GlobalObjectStorageHandleID: address = @0x0;

    /// The global object storage's table handle should be `0x0`
    public(friend) fun global_object_storage_handle(): ObjectID {
        // raw_table::new_table_handle(GlobalObjectStorageHandle)
        address_to_object_id(GlobalObjectStorageHandleID)
    }

    public(friend) fun add_to_global<T: key>(obj: ObjectEntity<T>) {
        add_field<ObjectID, ObjectEntity<T>>(global_object_storage_handle(), obj.id, obj);
    }

    public(friend) fun borrow_from_global<T: key>(object_id: ObjectID): &ObjectEntity<T> {
        borrow_field<ObjectID, ObjectEntity<T>>(global_object_storage_handle(), object_id)
    }

    public(friend) fun borrow_mut_from_global<T: key>(object_id: ObjectID): &mut ObjectEntity<T> {
        let object_entity = borrow_mut_field<ObjectID, ObjectEntity<T>>(global_object_storage_handle(), object_id);
        assert!(!is_frozen_internal(object_entity), ErrorObjectFrozen);
        object_entity
    }

    public(friend) fun remove_from_global<T: key>(object_id: ObjectID): ObjectEntity<T> {
        remove_field<ObjectID, ObjectEntity<T>>(global_object_storage_handle(), object_id)
    }

    public(friend) fun contains_global(object_id: ObjectID): bool {
        contains_field<ObjectID>(global_object_storage_handle(), object_id)
    }


    // === Object Raw Dynamic Table ===

    /// New a table. Aborts if the table exists.
    public(friend) fun new_table(table_handle: ObjectID): TableInfo {
        raw_table::new_table(table_handle)
    }

    /// Add a new entry to the table. Aborts if an entry for this
    /// key already exists. The entry itself is not stored in the
    /// table, and cannot be discovered from it.
    public(friend) fun add_field<K: copy + drop, V>(table_handle: ObjectID, key: K, val: V) {
        raw_table::add<K,V>(table_handle, key, val)
    }

    /// Acquire an immutable reference to the value which `key` maps to.
    /// Aborts if there is no entry for `key`.
    public(friend) fun borrow_field<K: copy + drop, V>(table_handle: ObjectID, key: K): &V {
        raw_table::borrow<K, V>(table_handle, key)
    }

    /// Acquire an immutable reference to the value which `key` maps to.
    /// Returns specified default value if there is no entry for `key`.
    public(friend) fun borrow_field_with_default<K: copy + drop, V>(table_handle: ObjectID, key: K, default: &V): &V {
        raw_table::borrow_with_default<K, V>(table_handle, key, default)
    }

    /// Acquire a mutable reference to the value which `key` maps to.
    /// Aborts if there is no entry for `key`.
    public(friend) fun borrow_mut_field<K: copy + drop, V>(table_handle: ObjectID, key: K): &mut V {
        raw_table::borrow_mut<K, V>(table_handle, key)
    }

    /// Acquire a mutable reference to the value which `key` maps to.
    /// Insert the pair (`key`, `default`) first if there is no entry for `key`.
    public(friend) fun borrow_mut_field_with_default<K: copy + drop, V: drop>(table_handle: ObjectID, key: K, default: V): &mut V {
        raw_table::borrow_mut_with_default<K, V>(table_handle, key, default)
    }

    /// Insert the pair (`key`, `value`) if there is no entry for `key`.
    /// update the value of the entry for `key` to `value` otherwise
    public(friend) fun upsert_field<K: copy + drop, V: drop>(table_handle: ObjectID, key: K, value: V) {
        raw_table::upsert<K, V>(table_handle, key, value)
    }

    /// Remove from `table` and return the value which `key` maps to.
    /// Aborts if there is no entry for `key`.
    public(friend) fun remove_field<K: copy + drop, V>(table_handle: ObjectID, key: K): V {
        raw_table::remove<K, V>(table_handle, key)
    }

    /// Returns true if `table` contains an entry for `key`.
    public(friend) fun contains_field<K: copy + drop>(table_handle: ObjectID, key: K): bool {
        raw_table::contains<K>(table_handle, key)
    }

    /// Returns the size of the table, the number of key-value pairs
    public(friend) fun table_length(table_handle: ObjectID): u64 {
        // native_box_length(table_handle)
        raw_table::length(table_handle)
    }

    /// Returns true if the table is empty (if `length` returns `0`)
    public(friend) fun is_empty_table(table_handle: ObjectID): bool {
        raw_table::is_empty(table_handle)
    }

    /// Drop a table even if it is not empty.
    public(friend) fun drop_unchecked_table(table_handle: ObjectID) {
        raw_table::drop_unchecked(table_handle)
    }

    /// Destroy a table. Aborts if the table is not empty
    public(friend) fun destroy_empty_table(table_handle: ObjectID) {
        raw_table::destroy_empty(table_handle)
    }


    #[test_only]
    public fun new_uid_for_test(tx_context: &mut moveos_std::tx_context::TxContext) : UID {
        let object_id = address_to_object_id(moveos_std::tx_context::fresh_address(tx_context));
        new_uid(object_id)
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
        let tx_context = moveos_std::tx_context::new_test_context(sender_addr);
        let init_count = 12;
        let test_struct = TestStruct {
            count: init_count,
        };
        let object_id = address_to_object_id(moveos_std::tx_context::fresh_address(&mut tx_context));
        let obj = new_with_id<TestStruct>(object_id, test_struct);
        assert!(contains_global(object_id), 1000);
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
        let TestStruct{count: _count} = test_obj;
        moveos_std::tx_context::drop(tx_context);
    }

    #[test(sender = @0x42)]
    fun test_shared(sender: address){
        let tx_context = moveos_std::tx_context::new_test_context(sender);
        let object_id = address_to_object_id(moveos_std::tx_context::fresh_address(&mut tx_context));
        let obj_enitty = new_internal(object_id, TestStruct { count: 1 });
        assert!(!is_shared_internal(&obj_enitty), 1000);
        assert!(!is_frozen_internal(&obj_enitty), 1001);
        to_shared_internal(&mut obj_enitty);
        assert!(is_shared_internal(&obj_enitty), 1002);
        assert!(!is_frozen_internal(&obj_enitty), 1003);
        add_to_global(obj_enitty);
        moveos_std::tx_context::drop(tx_context);
    }

    #[test(sender = @0x42)]
    fun test_frozen(sender: address){
        let tx_context = moveos_std::tx_context::new_test_context(sender);
        let object_id = address_to_object_id(moveos_std::tx_context::fresh_address(&mut tx_context));
        let obj_enitty = new_internal(object_id, TestStruct { count: 1 });
        assert!(!is_shared_internal(&obj_enitty), 1000);
        assert!(!is_frozen_internal(&obj_enitty), 1001);
        to_frozen_internal(&mut obj_enitty);
        assert!(!is_shared_internal(&obj_enitty), 1002);
        assert!(is_frozen_internal(&obj_enitty), 1003);
        add_to_global(obj_enitty);
        moveos_std::tx_context::drop(tx_context);
    }

    // An object can not be shared and frozen at the same time
    // This test just ensure the flag can be set at the same time
    #[test(sender = @0x42)]
    fun test_all_flag(sender: address){
        let tx_context = moveos_std::tx_context::new_test_context(sender);
        let object_id = address_to_object_id(moveos_std::tx_context::fresh_address(&mut tx_context));
        let obj_enitty = new_internal(object_id, TestStruct { count: 1 });
        assert!(!is_shared_internal(&obj_enitty), 1000);
        assert!(!is_frozen_internal(&obj_enitty), 1001);
        to_shared_internal(&mut obj_enitty);
        to_frozen_internal(&mut obj_enitty);
        assert!(is_shared_internal(&obj_enitty), 1002);
        assert!(is_frozen_internal(&obj_enitty), 1003);
        add_to_global(obj_enitty);
        moveos_std::tx_context::drop(tx_context);
    }

    #[test(sender = @0x42)]
    #[expected_failure(abort_code = 2, location = moveos_std::raw_table)]
    fun test_borrow_not_exist_failure(sender: signer) {
        let sender_addr = std::signer::address_of(&sender);
        let tx_context = moveos_std::tx_context::new_test_context(sender_addr);
        let object_id = address_to_object_id(moveos_std::tx_context::fresh_address(&mut tx_context));
        let obj = new_with_id(object_id, TestStruct { count: 1 });
        let TestStruct { count : _ } = remove(obj); 
        let _obj_ref = borrow_from_global<TestStruct>(object_id);
        moveos_std::tx_context::drop(tx_context);
    }

    #[test(sender = @0x42)]
    #[expected_failure(abort_code = 2, location = moveos_std::raw_table)]
    fun test_double_remove_failure(sender: signer) {
        let sender_addr = std::signer::address_of(&sender);
        let tx_context = moveos_std::tx_context::new_test_context(sender_addr);
        let object_id = address_to_object_id(moveos_std::tx_context::fresh_address(&mut tx_context));
        let object = new_with_id(object_id, TestStruct { count: 1 });
        
        let ObjectEntity{ id:_,owner:_,flag:_, value:test_struct1, state_root:_, size:_} = remove_from_global<TestStruct>(object_id);
        let test_struct2 = remove(object);
        let TestStruct { count : _ } = test_struct1;
        let TestStruct { count : _ } = test_struct2;
        moveos_std::tx_context::drop(tx_context);
    }

    #[test(sender = @0x42)]
    #[expected_failure(abort_code = 2, location = moveos_std::raw_table)]
    fun test_type_mismatch(sender: signer) {
        let sender_addr = std::signer::address_of(&sender);
        let tx_context = moveos_std::tx_context::new_test_context(sender_addr);
        let object_id = address_to_object_id(moveos_std::tx_context::fresh_address(&mut tx_context));
        let obj = new_with_id(object_id, TestStruct { count: 1 });
        {
            let test_struct_ref = borrow(&obj);
            assert!(test_struct_ref.count == 1, 1001);
        };
        {
            let test_struct2_object_entity = borrow_from_global<TestStruct2>(object_id);
            assert!(test_struct2_object_entity.value.count == 1, 1002);
        };
        drop(obj);
        moveos_std::tx_context::drop(tx_context);
    }

    struct TestStructID has store, copy, drop{
        id: u64,
    }

    #[test]
    fun test_custom_object_id(){
        let id = TestStructID{id: 1};
        let object_id = custom_object_id<TestStructID, TestStruct>(id);
        //ensure the object_id is the same as the object_id generated by the object.rs
        assert!(object_id::value(&object_id) == &std::bcs::to_bytes(&@0xaa825038ae811f5c94d20175699d808eae4c624fa85c81faad45de1145284e06), 1);
        let bytes = std::bcs::to_bytes(&object_id);
        //std::debug::print(&bytes);
        assert!(bytes == x"20aa825038ae811f5c94d20175699d808eae4c624fa85c81faad45de1145284e06", 2);
    }
}

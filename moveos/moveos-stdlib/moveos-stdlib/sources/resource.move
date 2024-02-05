// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// AccountStorage is part of the StorageAbstraction
/// It is used to store the account's resources
module moveos_std::resource {

    use std::ascii::String;
    use moveos_std::object_id::ObjectID;
    use moveos_std::object_id;
    use moveos_std::type_table::{key};
    use moveos_std::object::{Self, Object};
    #[test_only]
    use moveos_std::object::{borrow_object, borrow_mut_object, take_object};
    #[test_only]
    use moveos_std::signer;

    friend moveos_std::context;

    /// The resource with the given type already exists
    const ErrorResourceAlreadyExists: u64 = 1;
    /// The resource with the given type not exists 
    const ErrorResourceNotExists: u64 = 2;

    // const NamedTableResource: u64 = 0;
    // const NamedTableModule: u64 = 1;

    struct Resource has key {
        // resources: TypeTable,
        // modules: Table<String, MoveModule>,
    }

    // //Ensure the NamedTableID generate use same method with Rust code
    // public fun named_table_id(account: address, table_type: u64): ObjectID{
    //     object_id::address_to_object_id(tx_context::derive_id(bcs::to_bytes(&account), table_type))
    // }

    public fun resource_object_id(account: address): ObjectID {
        object_id::address_to_object_id(account)
    }

    /// Create a new resource object space
    public(friend) fun create_resource_object(account: address) {
        // let resource_object = AccountStorage {
        //     resources: type_table::new_with_id(named_table_id(account, NamedTableResource)),
        //     // modules: table::new_with_id(named_table_id(account, NamedTableModule)),
        // };
        // resource_object

        let object_id = object_id::address_to_object_id(account);
        let obj = object::new_with_id(object_id, Resource {});
        object::transfer(obj, account)
        // resource_object
    }

    // /// Borrow a resource from the AccountStorage
    // fun borrow_resource_from_resource_object<T: key>(self: &Object<AccountStorage>): &T {
    //     // type_table::borrow<T>(&self.resources)
    //     object::borrow_field<String, T>(object::id(self), key<T>())
    // }
    //
    // /// Borrow a mut resource from the AccountStorage
    // // fun borrow_mut_resource_from_resource_object<T: key>(self: &mut AccountStorage): &mut T {
    // fun borrow_mut_resource_from_resource_object<T: key>(self: &mut Object<AccountStorage>): &mut T {
    //     // type_table::borrow_mut<T>(&mut self.resources)
    //
    //     // object::borrow_mut_field<K, V>(object::id(obj), key)
    //     object::borrow_mut_field<String, T>(object::id(self), key<T>())
    // }
    // // public(friend) fun borrow_mut_field<K: copy + drop, V>(table_handle: ObjectID, key: K): &mut V {
    // //     raw_table::borrow_mut<K, V>(table_handle, key)
    // // }
    //
    // /// Add a resource to the resource object
    // fun add_resource_to_resource_object<T: key>(self: &mut Object<AccountStorage>, resource: T){
    //     // assert!(!type_table::contains<T>(&self.resources), ErrorResourceAlreadyExists);
    //     // type_table::add(&mut self.resources, resource);
    //
    //     assert!(!object::contains_field<String>(object::id(self), key<T>()), ErrorResourceAlreadyExists);
    //     object::add_field<String, T>(object::id(self), key<T>(), resource)
    // }
    //
    // /// Remove a resource from the resource object
    // fun remove_resource_from_resource_object<T: key>(self: &mut Object<AccountStorage>): T {
    //     // assert!(type_table::contains<T>(&self.resources), ErrorResourceNotExists);
    //     // type_table::remove<T>(&mut self.resources)
    //
    //     assert!(object::contains_field<String>(object::id(self), key<T>()), ErrorResourceNotExists);
    //     object::remove_field<String, T>(object::id(self), key<T>())
    // }
    //
    // fun exists_resource_at_resource_object<T: key>(self: &Object<AccountStorage>) : bool {
    //     // type_table::contains<T>(&self.resources)
    //     object::contains_field<String>(object::id(self), key<T>())
    // }

    // fun exists_module_at_resource_object(self: &AccountStorage, name: String) : bool {
    //     table::contains(&self.modules, name)
    // }

    // === Account Storage Functions

    public fun borrow_resource<T: key>(self: &Object<Resource>): &T {
        // borrow_resource_from_resource_object<T>(self)

        object::borrow_field<String, T>(object::id(self), key<T>())
    }

    public fun borrow_mut_resource<T: key>(self: &mut Object<Resource>): &mut T {
        // borrow_mut_resource_from_resource_object<T>(self)

        object::borrow_mut_field<String, T>(object::id(self), key<T>())
    }

    public fun move_resource_to<T: key>(self: &mut Object<Resource>, resource: T){
        // add_resource_to_resource_object(self, resource);

        assert!(!object::contains_field<String>(object::id(self), key<T>()), ErrorResourceAlreadyExists);
        object::add_field<String, T>(object::id(self), key<T>(), resource)
    }

    public fun move_resource_from<T: key>(self: &mut Object<Resource>): T {
        // remove_resource_from_resource_object(self)

        assert!(object::contains_field<String>(object::id(self), key<T>()), ErrorResourceNotExists);
        object::remove_field<String, T>(object::id(self), key<T>())
    }

    public fun exists_resource<T: key>(self: &Object<Resource>) : bool {
        // exists_resource_at_resource_object<T>(self)

        object::contains_field<String>(object::id(self), key<T>())
    }

    public(friend) fun transfer(obj: Object<Resource>, account: address) {
        object::transfer_extend(obj, account);
    }

    // // ==== Module functions ====
    //
    // /// Check if the account has a module with the given name
    // public fun exists_module(self: &AccountStorage, name: String): bool {
    //     exists_module_at_resource_object(self, name)
    // }
    //
    // /// Publish modules to the account's storage
    // /// Return true if the modules are upgraded
    // public(friend) fun publish_modules(self: &mut AccountStorage, account_address: address, modules: vector<MoveModule>) : bool {
    //     let i = 0;
    //     let len = vector::length(&modules);
    //     let (module_names, module_names_with_init_fn, indices) = move_module::sort_and_verify_modules(&modules, account_address);
    //
    //     let upgrade_flag = false;
    //     while (i < len) {
    //         let name = vector::pop_back(&mut module_names);
    //         let index = vector::pop_back(&mut indices);
    //         let m = vector::borrow(&modules, index);
    //
    //         // The module already exists, which means we are upgrading the module
    //         if (table::contains(&self.modules, name)) {
    //             let old_m = table::remove(&mut self.modules, name);
    //             move_module::check_comatibility(m, &old_m);
    //             upgrade_flag = true;
    //         } else {
    //             // request init function invoking
    //             if (vector::contains(&module_names_with_init_fn, &name)) {
    //                 move_module::request_init_functions(vector::singleton(copy name), account_address);
    //             }
    //         };
    //         table::add(&mut self.modules, name, *m);
    //         i = i + 1;
    //     };
    //     upgrade_flag
    // }

    #[test_only]
    fun drop_resource_object(self: Object<Resource>) {
        // let AccountStorage {
        //     resources,
        //     modules
        // } = self;
        // type_table::drop_unchecked(resources);
        // table::drop_unchecked(modules);

        // let AccountStorage {
        // } = self;
        object::drop_unchecked_table(object::id(&self));
        let obj = object::remove(self);
        let Resource {} = obj;
    }
    
    
    // #[test]
    // fun test_named_table_id() {
    //     assert!(named_table_id(@0xae43e34e51db9c833ab50dd9aa8b27106519e5bbfd533737306e7b69ef253647, NamedTableResource) == object_id::address_to_object_id(@0x04d8b5ccef4d5b55fa9371d1a9c344fcd4bd40dd9f32dd1d94696775fe3f3013), 1000);
    //     assert!(named_table_id(@0xae43e34e51db9c833ab50dd9aa8b27106519e5bbfd533737306e7b69ef253647, NamedTableModule) == object_id::address_to_object_id(@0xead64c5e724c9d52b0eb792b350d56001f1fe0dc2dec0e2e713420daba18109a), 1001);
    // }

    #[test_only]
    struct Test has key{
        addr: address,
        version: u64
    }

    #[test(sender=@0x42)]
    fun test_resource_object(sender: signer){
        let sender_addr = signer::address_of(&sender);
        create_resource_object(sender_addr);
        let obj_mut = borrow_mut_object<Resource>(&sender, resource_object_id(sender_addr));
        move_resource_to(obj_mut, Test{
            addr: sender_addr,
            version: 1,
        });
        let obj = take_object<Resource>(&sender, resource_object_id(sender_addr));
        Self::drop_resource_object(obj);
    }

    #[test(sender=@0x42)]
    fun test_move_to_resource_object(sender: signer){
        let sender_addr = signer::address_of(&sender);
        create_resource_object(sender_addr);
        let obj_mut = borrow_mut_object<Resource>(&sender, resource_object_id(sender_addr));
        move_resource_to(obj_mut, Test{
            addr: sender_addr,
            version: 1,
        });
        let obj = take_object<Resource>(&sender, resource_object_id(sender_addr));
        Self::drop_resource_object(obj);
    }

    #[test(sender=@0x42)]
    fun test_move_from_resource_object(sender: signer){
        let sender_addr = signer::address_of(&sender);
        create_resource_object(sender_addr);
        let obj_mut = borrow_mut_object<Resource>(&sender, resource_object_id(sender_addr));
        move_resource_to(obj_mut, Test{
            addr: sender_addr,
            version: 1,
        });
        let Test {
            addr,
            version
        } = move_resource_from<Test>(obj_mut);
        assert!(addr == sender_addr, 0x10);
        assert!(version == 1, 0x11);
        let obj = take_object<Resource>(&sender, resource_object_id(sender_addr));
        Self::drop_resource_object(obj);
    } 

    #[test(sender=@0x42)]
    #[expected_failure(abort_code = ErrorResourceAlreadyExists, location = Self)]
    fun test_failure_repeatedly_move_to_resource_object(sender: signer){
        let sender_addr = signer::address_of(&sender);
        create_resource_object(sender_addr);
        let obj_mut = borrow_mut_object<Resource>(&sender, resource_object_id(sender_addr));
        move_resource_to(obj_mut, Test{
            addr: sender_addr,
            version: 1,
        });
        move_resource_to(obj_mut, Test{
            addr: sender_addr,
            version: 1,
        });
        let obj = take_object<Resource>(&sender, resource_object_id(sender_addr));
        Self::drop_resource_object(obj);
    }

    #[test(sender=@0x42)]
    #[expected_failure(abort_code = ErrorResourceNotExists, location = Self)]
    fun test_failure_repeatedly_move_from_resource_object(sender: signer){
        let sender_addr = signer::address_of(&sender);
        create_resource_object(sender_addr);
        let obj_mut = borrow_mut_object<Resource>(&sender, resource_object_id(sender_addr));
        move_resource_to(obj_mut, Test{
            addr: sender_addr,
            version: 1,
        });
        let Test {
            addr: _,
            version: _
        } = move_resource_from<Test>(obj_mut);
        let Test {
            addr: _,
            version: _
        } = move_resource_from<Test>(obj_mut);
        let obj = take_object<Resource>(&sender, resource_object_id(sender_addr));
        Self::drop_resource_object(obj);
    }

    #[test(sender=@0x42)]
    fun test_borrow_resource(sender: signer){
        let sender_addr = signer::address_of(&sender);
        create_resource_object(sender_addr);
        let obj_mut = borrow_mut_object<Resource>(&sender, resource_object_id(sender_addr));
        move_resource_to(obj_mut, Test{
            addr: sender_addr,
            version: 1,
        });

        let ref_test = borrow_resource<Test>(obj_mut);
        assert!( ref_test.version == 1, 1);
        assert!( ref_test.addr == sender_addr, 2);
        let obj = take_object<Resource>(&sender, resource_object_id(sender_addr));
        Self::drop_resource_object(obj);
    }

    #[test(sender=@0x42)]
    fun test_borrow_mut_resource(sender: signer){
        let sender_addr = signer::address_of(&sender);
        create_resource_object(sender_addr);
        let obj_mut = borrow_mut_object<Resource>(&sender, resource_object_id(sender_addr));
        move_resource_to(obj_mut, Test{
            addr: sender_addr,
            version: 1,
        });
        {
            let ref_test = borrow_mut_resource<Test>(obj_mut);
            assert!( ref_test.version == 1, 1);
            assert!( ref_test.addr == sender_addr, 2);
            ref_test.version = 2;
        };
        {
            let ref_test = borrow_resource<Test>(obj_mut);
            assert!( ref_test.version == 2, 3);
        };
        let obj = take_object<Resource>(&sender, resource_object_id(sender_addr));
        Self::drop_resource_object(obj);
    }

    #[test(sender=@0x42)]
    #[expected_failure(abort_code = 2, location = moveos_std::raw_table)]
    fun test_failure_borrow_resource_no_exists(sender: signer){
        let sender_addr = signer::address_of(&sender);
        create_resource_object(sender_addr);
        let obj_ref = borrow_object<Resource>(resource_object_id(sender_addr));
        borrow_resource<Test>(obj_ref);
        let obj = take_object<Resource>(&sender, resource_object_id(sender_addr));
        Self::drop_resource_object(obj);
    }

    #[test(sender=@0x42)]
    #[expected_failure(abort_code = 2, location = moveos_std::raw_table)]
    fun test_failure_borrow_mut_resource_no_exists(sender: signer){
        let sender_addr = signer::address_of(&sender);
        create_resource_object(sender_addr);
        let obj_mut = borrow_mut_object<Resource>(&sender, resource_object_id(sender_addr));
        borrow_mut_resource<Test>(obj_mut);
        let obj = take_object<Resource>(&sender, resource_object_id(sender_addr));
        Self::drop_resource_object(obj);
    }
   
    #[test(sender=@0x42)]
    fun test_ensure_move_from_and_exists(sender: signer){
        let sender_addr = signer::address_of(&sender);
        create_resource_object(sender_addr);
        let obj_mut = borrow_mut_object<Resource>(&sender, resource_object_id(sender_addr));
        let test_exists = exists_resource<Test>(obj_mut);
        assert!(!test_exists, 1);
        move_resource_to(obj_mut, Test{
            addr: sender_addr,
            version: 1,
        });
        let test_exists = exists_resource<Test>(obj_mut);
        assert!(test_exists, 2);
        let test = move_resource_from<Test>(obj_mut);
        let test_exists = exists_resource<Test>(obj_mut);
        assert!(!test_exists, 3);
        let Test{
            addr: _,
            version: _
        } = test;
        let obj = take_object<Resource>(&sender, resource_object_id(sender_addr));
        Self::drop_resource_object(obj);
    }

    // #[test(sender=@0x42)]
    // fun test_publish_modules(sender: signer) {
    //     let sender_addr = signer::address_of(&sender);
    //     create_resource_object(sender_addr);
    //     let obj_mut = borrow_mut_object<AccountStorage>(&sender, resource_object_id(sender_addr));
    //     // The following is the bytes and hex of the compiled module: example/counter/sources/counter.move
    //     // with account 0x42
    //     let module_bytes: vector<u8> = x"a11ceb0b060000000b010004020408030c26043206053832076a7308dd0140069d02220abf02050cc402560d9a03020000010100020c00010300000004000100000500010000060201000007030400010807080108010909010108010a0a0b0108040605060606010708010002070801060c0106080101030107080001080002070801050107090003070801060c090002060801050106090007636f756e74657207636f6e7465787407436f756e74657207436f6e7465787408696e63726561736509696e6372656173655f04696e69740576616c756513626f72726f775f6d75745f7265736f75726365106d6f76655f7265736f757263655f746f0f626f72726f775f7265736f75726365000000000000000000000000000000000000000000000000000000000000004200000000000000000000000000000000000000000000000000000000000000020520000000000000000000000000000000000000000000000000000000000000004200020107030001040001030b0011010201010000050d0b00070038000c010a01100014060100000000000000160b010f0015020200000001060b000b0106000000000000000012003801020301000001060b000700380210001402000000";
    //     let m: MoveModule = move_module::new(module_bytes);
    //     Self::publish_modules(obj_mut, sender, vector::singleton(m));
    //     let obj = take_object<AccountStorage>(&sender, resource_object_id(sender_addr));
    //     Self::drop_resource_object(obj);
    // }
}
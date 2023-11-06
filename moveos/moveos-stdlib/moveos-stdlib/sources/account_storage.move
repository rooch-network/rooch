// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// AccountStorage is part of the StorageAbstraction
/// It is used to store the account's resources and modules
module moveos_std::account_storage {

    use std::string::String;
    use std::vector;
    use std::error;
    use moveos_std::bcs;
    use moveos_std::type_table::{Self, TypeTable};
    use moveos_std::table::{Self, Table};
    use moveos_std::object::{Self, ObjectID, Object};
    use moveos_std::tx_context;
    use moveos_std::move_module::{Self, MoveModule};

    friend moveos_std::context;

    /// The resource with the given type already exists
    const ErrorResourceAlreadyExists: u64 = 1;
    /// The resource with the given type not exists 
    const ErrorResourceNotExists: u64 = 2;

    const NamedTableResource: u64 = 0;
    const NamedTableModule: u64 = 1;

    struct AccountStorage has key {
        resources: TypeTable,
        modules: Table<String, MoveModule>,
    }

    //Ensure the NamedTableID generate use same method with Rust code
    public fun named_table_id(account: address, table_type: u64): ObjectID{
        object::address_to_object_id(tx_context::derive_id(bcs::to_bytes(&account), table_type))
    }

    /// Create a new account storage space
    public(friend) fun create_account_storage(account: address) : AccountStorage {
        let account_storage = AccountStorage {
            resources: type_table::new_with_id(named_table_id(account, NamedTableResource)),
            modules: table::new_with_id(named_table_id(account, NamedTableModule)),
        };
        account_storage
    }

    /// Borrow a resource from the AccountStorage
    fun borrow_resource_from_account_storage<T: key>(self: &AccountStorage): &T {
        type_table::borrow<T>(&self.resources)
    }

    /// Borrow a mut resource from the AccountStorage
    fun borrow_mut_resource_from_account_storage<T: key>(self: &mut AccountStorage): &mut T {
        type_table::borrow_mut<T>(&mut self.resources)
    }

    /// Add a resource to the account storage
    fun add_resource_to_account_storage<T: key>(self: &mut AccountStorage, resource: T){
        assert!(!type_table::contains<T>(&self.resources), error::invalid_argument(ErrorResourceAlreadyExists));
        type_table::add(&mut self.resources, resource);
    }

    /// Remove a resource from the account storage
    fun remove_resource_from_account_storage<T: key>(self: &mut AccountStorage): T {
        assert!(type_table::contains<T>(&self.resources), error::invalid_argument(ErrorResourceNotExists));
        type_table::remove<T>(&mut self.resources)
    }

    fun exists_resource_at_account_storage<T: key>(self: &AccountStorage) : bool {
        type_table::contains<T>(&self.resources)
    }

    fun exists_module_at_account_storage(self: &AccountStorage, name: String) : bool {
        table::contains(&self.modules, name)
    }

    // === Account Storage Functions

    public fun borrow_resource<T: key>(self: &AccountStorage): &T {
        borrow_resource_from_account_storage<T>(self)
    }

    public fun borrow_mut_resource<T: key>(self: &mut AccountStorage): &mut T {
        borrow_mut_resource_from_account_storage<T>(self)
    }

    public fun move_resource_to<T: key>(self: &mut AccountStorage, resource: T){
        add_resource_to_account_storage(self, resource);
    }

    public fun move_resource_from<T: key>(self: &mut AccountStorage): T {
        remove_resource_from_account_storage(self)
    }

    public fun exists_resource<T: key>(self: &AccountStorage) : bool {
        exists_resource_at_account_storage<T>(self)
    }

    public(friend) fun transfer(obj: Object<AccountStorage>, account: address) {
        object::transfer_extend(obj, account);
    }

    // ==== Module functions ====

    /// Check if the account has a module with the given name
    public fun exists_module(self: &AccountStorage, name: String): bool {
        exists_module_at_account_storage(self, name) 
    }

    fun pop_module_by_name(modules: &mut vector<MoveModule>, name: String): MoveModule {
        let i = 0;
        let len = vector::length(modules);
        while (i < len) {
            let m = vector::borrow(modules, i);
            if (move_module::module_name(m) == name) {
                return vector::remove(modules, i)
            };
            i = i + 1;
        };
        abort(0x0) // unreachable.
    }

    /// Publish modules to the account's storage
    /// Return true if the modules are upgraded
    public(friend) fun publish_modules(self: &mut AccountStorage, account_address: address, modules: vector<MoveModule>) : bool {        
        let i = 0;
        let len = vector::length(&modules);
        let (module_names, module_names_with_init_fn) = move_module::sort_and_verify_modules(&modules, account_address);
        
        let upgrade_flag = false;
        while (i < len) {
            let name = vector::pop_back(&mut module_names);
            let m = pop_module_by_name(&mut modules, name);   

            // The module already exists, which means we are upgrading the module
            if (table::contains(&self.modules, name)) {
                let old_m = table::remove(&mut self.modules, name);
                move_module::check_comatibility(&m, &old_m);
                upgrade_flag = true;
            } else {
                // request init function invoking
                if (vector::contains(&module_names_with_init_fn, &name)) {
                    move_module::request_init_functions(vector::singleton(copy name), account_address);
                }
            };
            table::add(&mut self.modules, name, m);
            i = i + 1;
        };
        upgrade_flag 
    }

    #[test_only]
    fun drop_account_storage(self: AccountStorage) {
        let AccountStorage {
            resources,
            modules
        } = self;
        type_table::drop_unchecked(resources);
        table::drop_unchecked(modules);
    }
    
    
    #[test]
    fun test_named_table_id() {
        assert!(named_table_id(@0xae43e34e51db9c833ab50dd9aa8b27106519e5bbfd533737306e7b69ef253647, NamedTableResource) == object::address_to_object_id(@0x04d8b5ccef4d5b55fa9371d1a9c344fcd4bd40dd9f32dd1d94696775fe3f3013), 1000);
        assert!(named_table_id(@0xae43e34e51db9c833ab50dd9aa8b27106519e5bbfd533737306e7b69ef253647, NamedTableModule) == object::address_to_object_id(@0xead64c5e724c9d52b0eb792b350d56001f1fe0dc2dec0e2e713420daba18109a), 1001);
    }

    #[test_only]
    struct Test has key{
        addr: address,
        version: u64
    }

    #[test(sender=@0x42)]
    fun test_account_storage(sender: address){
        let account_storage = create_account_storage(sender);
        move_resource_to(&mut account_storage, Test{
            addr: sender,
            version: 1,
        });
        Self::drop_account_storage(account_storage);
    }

    #[test(sender=@0x42)]
    fun test_move_to_account_storage(sender: address){
        let account_storage = create_account_storage(sender);
        move_resource_to(&mut account_storage, Test{
            addr: sender,
            version: 1,
        });
        Self::drop_account_storage(account_storage);
    }

    #[test(sender=@0x42)]
    fun test_move_from_account_storage(sender: address){
        let account_storage = create_account_storage(sender);
        move_resource_to(&mut account_storage, Test{
            addr: sender,
            version: 1,
        });
        let Test {
            addr,
            version
        } = move_resource_from<Test>(&mut account_storage);
        assert!(addr == sender, 0x10);
        assert!(version == 1, 0x11);
        Self::drop_account_storage(account_storage);
    } 

    #[test(sender=@0x42)]
    #[expected_failure(abort_code = 65537, location = Self)]
    fun test_failure_repeatedly_move_to_account_storage(sender: address){
        let account_storage = create_account_storage(sender);
        move_resource_to(&mut account_storage, Test{
            addr: sender,
            version: 1,
        });
        move_resource_to(&mut account_storage, Test{
            addr: sender,
            version: 1,
        });
        Self::drop_account_storage(account_storage);
    }

    #[test(sender=@0x42)]
    #[expected_failure(abort_code = 65538, location = Self)]
    fun test_failure_repeatedly_move_from_account_storage(sender: address){
        let account_storage = create_account_storage(sender);
        move_resource_to(&mut account_storage, Test{
            addr: sender,
            version: 1,
        });
        let Test {
            addr: _,
            version: _
        } = move_resource_from<Test>(&mut account_storage);
        let Test {
            addr: _,
            version: _
        } = move_resource_from<Test>(&mut account_storage);
        Self::drop_account_storage(account_storage);
    }

    #[test(sender=@0x42)]
    fun test_borrow_resource(sender: address){
        let account_storage = create_account_storage(sender);
        move_resource_to(&mut account_storage, Test{
            addr: sender,
            version: 1,
        });

        let ref_test = borrow_resource<Test>(&account_storage);
        assert!( ref_test.version == 1, 1);
        assert!( ref_test.addr == sender, 2);
        Self::drop_account_storage(account_storage);
    }

    #[test(sender=@0x42)]
    fun test_borrow_mut_resource(sender: address){
        let account_storage = create_account_storage(sender);
        move_resource_to(&mut account_storage, Test{
            addr: sender,
            version: 1,
        });
        {
            let ref_test = borrow_mut_resource<Test>(&mut account_storage);
            assert!( ref_test.version == 1, 1);
            assert!( ref_test.addr == sender, 2);
            ref_test.version = 2;
        };
        {
            let ref_test = borrow_resource<Test>(&account_storage);
            assert!( ref_test.version == 2, 3);
        };
        Self::drop_account_storage(account_storage);
    }

    #[test(sender=@0x42)]
    #[expected_failure(abort_code = 393218, location = moveos_std::raw_table)]
    fun test_failure_borrow_resource_no_exists(sender: address){
        let account_storage = create_account_storage(sender);
        borrow_resource<Test>(&account_storage);
        Self::drop_account_storage(account_storage);
    }

    #[test(sender=@0x42)]
    #[expected_failure(abort_code = 393218, location = moveos_std::raw_table)]
    fun test_failure_borrow_mut_resource_no_exists(sender: address){
        let account_storage = create_account_storage(sender);
        borrow_mut_resource<Test>(&mut account_storage);
        Self::drop_account_storage(account_storage);
    }
   
    #[test(sender=@0x42)]
    fun test_ensure_move_from_and_exists(sender: address){
        let account_storage = create_account_storage(sender);
        let test_exists = exists_resource<Test>(&account_storage);
        assert!(!test_exists, 1);
        move_resource_to(&mut account_storage, Test{
            addr: sender,
            version: 1,
        });
        let test_exists = exists_resource<Test>(&account_storage);
        assert!(test_exists, 2);
        let test = move_resource_from<Test>(&mut account_storage); 
        let test_exists = exists_resource<Test>(&account_storage);
        assert!(!test_exists, 3);
        let Test{
            addr: _,
            version: _
        } = test;
        Self::drop_account_storage(account_storage);
    }

    #[test(sender=@0x42)]
    fun test_publish_modules(sender: address) {
        let account_storage = create_account_storage(sender);
        // The following is the bytes and hex of the compiled module: example/counter/sources/counter.move
        // with account 0x42       
        let module_bytes: vector<u8> = x"a11ceb0b060000000b010004020408030c26043206053832076a7308dd0140069d02220abf02050cc402560d9a03020000010100020c00010300000004000100000500010000060201000007030400010807080108010909010108010a0a0b0108040605060606010708010002070801060c0106080101030107080001080002070801050107090003070801060c090002060801050106090007636f756e74657207636f6e7465787407436f756e74657207436f6e7465787408696e63726561736509696e6372656173655f04696e69740576616c756513626f72726f775f6d75745f7265736f75726365106d6f76655f7265736f757263655f746f0f626f72726f775f7265736f75726365000000000000000000000000000000000000000000000000000000000000004200000000000000000000000000000000000000000000000000000000000000020520000000000000000000000000000000000000000000000000000000000000004200020107030001040001030b0011010201010000050d0b00070038000c010a01100014060100000000000000160b010f0015020200000001060b000b0106000000000000000012003801020301000001060b000700380210001402000000";
        let m: MoveModule = move_module::new(module_bytes);
        Self::publish_modules(&mut account_storage, sender, vector::singleton(m));
        Self::drop_account_storage(account_storage);
    }
}
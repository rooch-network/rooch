// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// ResourceObject is part of the StorageAbstraction
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

    struct Resource has key, store {}

    public fun resource_object_id(account: address): ObjectID {
        object_id::address_to_object_id(account)
    }

    /// Create a new resource object space
    public(friend) fun create_resource_object(account: address) {
        let object_id = object_id::address_to_object_id(account);
        let obj = object::new_with_id(object_id, Resource {});
        object::transfer(obj, account)
    }

    // === Resource Object Functions

    public fun borrow_resource<T: key>(self: &Object<Resource>): &T {
        object::borrow_field<String, T>(object::id(self), key<T>())
    }

    public fun borrow_mut_resource<T: key>(self: &mut Object<Resource>): &mut T {
        object::borrow_mut_field<String, T>(object::id(self), key<T>())
    }

    public fun move_resource_to<T: key>(self: &mut Object<Resource>, resource: T){
        assert!(!object::contains_field<String>(object::id(self), key<T>()), ErrorResourceAlreadyExists);
        object::add_field<String, T>(object::id(self), key<T>(), resource)
    }

    public fun move_resource_from<T: key>(self: &mut Object<Resource>): T {
        assert!(object::contains_field<String>(object::id(self), key<T>()), ErrorResourceNotExists);
        object::remove_field<String, T>(object::id(self), key<T>())
    }

    public fun exists_resource<T: key>(self: &Object<Resource>) : bool {
        object::contains_field<String>(object::id(self), key<T>())
    }

    public(friend) fun transfer(obj: Object<Resource>, account: address) {
        object::transfer_extend(obj, account);
    }


    #[test_only]
    fun drop_resource_object(self: Object<Resource>) {
        object::drop_unchecked_table(object::id(&self));
        let obj = object::remove(self);
        let Resource {} = obj;
    }
    
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
}
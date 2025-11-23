// Test data generator module for Rooch pruner testing
// Provides functions to create and update objects to generate prunable old versions

module test_data_generator::test_object {
    // use std::string::{Self};
    use moveos_std::object::{Self};
    use moveos_std::timestamp;

    /// Test object that will be repeatedly updated
    struct TestObject has key, store {
        value: u64,
        update_count: u64,
        last_update_time: u64,
        data: vector<u8>,
    }

    /// Create a new test object with custom ID
    /// This generates new data (will be kept)
    public entry fun create_object(
        account: &signer,
        initial_value: u64,
    ) {
        let obj = TestObject {
            value: initial_value,
            update_count: 0,
            last_update_time: timestamp::now_milliseconds(),
            data: b"test_data_for_storage_testing",
        };
        
        // Create object with custom ID (using initial_value as ID)
        // This allows creating multiple objects per account
        let account_addr = std::signer::address_of(account);
        let obj_wrapper = object::new_with_id(initial_value, obj);
        object::transfer(obj_wrapper, account_addr);
    }

    /// Create a new account-named test object (one per account)
    /// This generates new data (will be kept)
    public entry fun create_account_named_object(
        account: &signer,
        initial_value: u64,
    ) {
        let obj = TestObject {
            value: initial_value,
            update_count: 0,
            last_update_time: timestamp::now_milliseconds(),
            data: b"test_data_for_storage_testing",
        };
        
        // Create account-named object (one per account, can be updated repeatedly)
        let account_addr = std::signer::address_of(account);
        let obj_wrapper = object::new_account_named_object(account_addr, obj);
        object::transfer(obj_wrapper, account_addr);
    }

    /// Update an account-named object
    /// This overwrites old data, generating prunable old versions
    public entry fun update_account_object(
        account: &signer,
        account_addr: address,
        new_value: u64,
    ) {
        // Get account-named object ID and borrow mutably
        let id = object::account_named_object_id<TestObject>(account_addr);
        let obj_ref = object::borrow_mut_object<TestObject>(account, id);
        let test_obj = object::borrow_mut(obj_ref);
        
        // Update fields - this creates a new version, old version becomes prunable
        // Each update modifies the object, causing SMT node versioning
        test_obj.value = new_value;
        test_obj.update_count = test_obj.update_count + 1;
        test_obj.last_update_time = timestamp::now_milliseconds();
        
        // Note: Updating value, update_count, and last_update_time is sufficient
        // to generate new SMT versions that make old versions prunable
    }

    /// Update object by custom ID (for objects created with new_with_id)
    public entry fun update_object_by_id(
        account: &signer,
        object_id_value: u64,
        new_value: u64,
    ) {
        // Get object ID using the same ID value and type used in create_object
        let id = object::custom_object_id<u64, TestObject>(object_id_value);
        let obj_ref = object::borrow_mut_object<TestObject>(account, id);
        let test_obj = object::borrow_mut(obj_ref);
        
        // Update fields
        test_obj.value = new_value;
        test_obj.update_count = test_obj.update_count + 1;
        test_obj.last_update_time = timestamp::now_milliseconds();
    }

    /// Batch create multiple objects
    /// Useful for initializing the object pool
    public entry fun batch_create_objects(
        account: &signer,
        count: u64,
    ) {
        let i = 0;
        while (i < count) {
            create_object(account, i);
            i = i + 1;
        };
    }

    /// Get account-named object info for testing
    public fun get_account_object_value(account_addr: address): (u64, u64, u64) {
        let id = object::account_named_object_id<TestObject>(account_addr);
        let obj_ref = object::borrow_object<TestObject>(id);
        let test_obj = object::borrow(obj_ref);
        (test_obj.value, test_obj.update_count, test_obj.last_update_time)
    }

    /// Get object info by custom ID for testing
    public fun get_object_value(object_id_value: u64): (u64, u64, u64) {
        let id = object::custom_object_id<u64, TestObject>(object_id_value);
        let obj_ref = object::borrow_object<TestObject>(id);
        let test_obj = object::borrow(obj_ref);
        (test_obj.value, test_obj.update_count, test_obj.last_update_time)
    }
}


// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_nursery::cosmwasm_vm {
    use std::vector;
    use std::string::{Self, String};
    use std::option::{Self, Option};
    
    use moveos_std::features;
    use moveos_std::table::{Self, Table};
    use moveos_std::result::{Self, Result, ok};

    use rooch_nursery::cosmwasm_std::{Response, Error, Env, MessageInfo, 
        new_error, new_error_result, serialize_env, serialize_message_info, deserialize_response};

    struct Instance {
        code_checksum: vector<u8>,
        store: table::Table<String, vector<u8>>
    }

    public fun code_checksum(instance: &Instance): vector<u8> {
        instance.code_checksum
    }

    public fun store(instance: &Instance): &table::Table<String, vector<u8>> {
        &instance.store
    }

    public fun from_code(code: vector<u8>): Result<Instance, Error> {
        features::ensure_wasm_enabled();

        let store = table::new<String, vector<u8>>();

        let (checksum, error_code) = native_create_instance(code, &mut store);
        if (error_code == 0) {
            ok(Instance { 
                code_checksum: checksum,
                store: store,
            })
        } else {
            table::drop(store);
            new_error_result(error_code, string::utf8(b"native_create_instance_error"))
        }
    }
    
    public fun call_instantiate(instance: &mut Instance, env: &Env, info: &MessageInfo, msg: vector<u8>): Result<Response, Error> {
        let (raw_response, error_code) = native_call_instantiate_raw(instance.code_checksum, &mut instance.store, serialize_env(env), serialize_message_info(info), msg);
        if (error_code == 0) {
            ok(deserialize_response(raw_response))
        } else {
            new_error_result(error_code, string::utf8(b"native_call_instantiate_raw_error"))
        }
    }

    public fun call_execute(instance: &mut Instance, env: &Env, info: &MessageInfo, msg: vector<u8>): Result<Response, Error> {
        let (raw_response, error_code) = native_call_execute_raw(instance.code_checksum, &mut instance.store, serialize_env(env), serialize_message_info(info), msg);
        if (error_code == 0) {
            ok(deserialize_response(raw_response))
        } else {
            new_error_result(error_code, string::utf8(b"native_call_execute_raw_error"))
        }
    }

    public fun call_query(instance: &Instance, env: &Env, msg: vector<u8>): Result<Response, Error> {
        let (raw_response, error_code) = native_call_query_raw(instance.code_checksum, &instance.store, serialize_env(env), msg);
        if (error_code == 0) {
            ok(deserialize_response(raw_response))
        } else {
            new_error_result(error_code, string::utf8(b"native_call_query_raw_error"))
        }
    }

    public fun call_migrate(instance: &mut Instance, env: &Env, msg: vector<u8>): Result<Response, Error> {
        let (raw_response, error_code) = native_call_migrate_raw(instance.code_checksum, &mut instance.store, serialize_env(env), msg);
        if (error_code == 0) {
            ok(deserialize_response(raw_response))
        } else {
            new_error_result(error_code, string::utf8(b"native_call_migrate_raw_error"))
        }
    }

    public fun call_reply(instance: &mut Instance, env: &Env, msg: vector<u8>): Result<Response, Error> {
        let (raw_response, error_code) = native_call_reply_raw(instance.code_checksum, &mut instance.store, serialize_env(env), msg);
        if (error_code == 0) {
            ok(deserialize_response(raw_response))
        } else {
            new_error_result(error_code, string::utf8(b"native_call_reply_raw_error"))
        }
    }

    public fun call_sudo(instance: &mut Instance, env: &Env, msg: vector<u8>): Result<Response, Error> {
        let (raw_response, error_code) = native_call_sudo_raw(instance.code_checksum, &mut instance.store, serialize_env(env), msg);
        if (error_code == 0) {
            ok(deserialize_response(raw_response))
        } else {
            new_error_result(error_code, string::utf8(b"native_call_sudo_raw_error"))
        }
    }

    /// Destroys an Instance and releases associated resources.
    public fun destroy_instance(instance: Instance): Option<Error> {
        let Instance { code_checksum, store } = instance;
        table::drop(store);

        let error_code = native_destroy_instance(code_checksum);
        if (error_code == 0) {
            option::none()
        } else {
            option::some(new_error(error_code, string::utf8(b"native_destroy_instance_error")))
        }
    }

    /// Deserialize a slice of bytes into the given type T.
    /// This function mimics the behavior of cosmwasm_vm::from_slice.
    public fun from_slice<T>(_data: vector<u8>): Result<T, Error> {
        new_error_result(1, string::utf8(b"native_destroy_instance_error"))
    }

    /// Serialize the given data to a vector of bytes.
    /// This function mimics the behavior of cosmwasm_vm::to_vec.
    public fun to_vec<T>(_data: &T): Result<vector<u8>, Error> {
        new_error_result(1, string::utf8(b"native_destroy_instance_error"))
    }
 

    // Native function declarations
    native fun native_create_instance(code: vector<u8>, store: &mut Table<String, vector<u8>>): (vector<u8>, u32);
    native fun native_destroy_instance(code_checksum: vector<u8>): u32;
    native fun native_call_instantiate_raw(code_checksum: vector<u8>, store: &mut Table<String, vector<u8>>, env: vector<u8>, info: vector<u8>, msg: vector<u8>): (vector<u8>, u32);
    native fun native_call_execute_raw(code_checksum: vector<u8>, store: &mut Table<String, vector<u8>>, env: vector<u8>, info: vector<u8>, msg: vector<u8>): (vector<u8>, u32);
    native fun native_call_query_raw(code_checksum: vector<u8>, store: &Table<String, vector<u8>>, env: vector<u8>, msg: vector<u8>): (vector<u8>, u32);
    native fun native_call_migrate_raw(code_checksum: vector<u8>, store: &mut Table<String, vector<u8>>, env: vector<u8>, msg: vector<u8>): (vector<u8>, u32);
    native fun native_call_reply_raw(code_checksum: vector<u8>, store: &mut Table<String, vector<u8>>, env: vector<u8>, msg: vector<u8>): (vector<u8>, u32);
    native fun native_call_sudo_raw(code_checksum: vector<u8>, store: &mut Table<String, vector<u8>>, env: vector<u8>, msg: vector<u8>):(vector<u8>, u32);

    #[test]
    fun test_from_code() {
        // Enable all features for testing
        features::init_and_enable_all_features_for_test();

        // Create a simple CosmWasm contract bytecode in WAT format
        let wasm_code_wat: vector<u8> = b"(module)"; // Example WAT code

        // Call from_code function to create an instance
        let instance_result = from_code(wasm_code_wat);

        // Verify that the result is successful
        let instance = result::assert_ok(instance_result, 1); // Use assert_ok here

        // Verify some properties of the instance
        // Note: The specific verification method may need to be adjusted based on the actual definition of the Instance structure
        assert!(vector::length(&code_checksum(&instance)) > 0, 2);

        // Destroy the instance
        let destroy_result = destroy_instance(instance);
        assert!(option::is_none(&destroy_result), 3);
    }

}
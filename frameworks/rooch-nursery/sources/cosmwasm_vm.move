// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_nursery::cosmwasm_vm {
    use std::string::{Self, String};
    use std::option::{Self, Option};
    
    use moveos_std::features;
    use moveos_std::json;
    use moveos_std::object::{ObjectID};
    use moveos_std::table::{Self, Table};
    use moveos_std::result::{Result, ok};

    use rooch_nursery::cosmwasm_std::{Response, Error, Env, MessageInfo, Reply, StdResult,
        new_error, new_error_result, serialize_env, serialize_message_info, serialize_message, deserialize_response, deserialize_stdresult};

    struct Instance has key, store {
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
        let store_handle = table::handle(&store);

        let (checksum, error_code) = native_create_instance(code, store_handle);
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
    
    #[data_struct(T)]
    public fun call_instantiate<T: drop>(instance: &mut Instance, env: &Env, info: &MessageInfo, msg: &T): Result<Response, Error> {
        let store_handle = table::handle(&mut instance.store);
        let env_bytes = serialize_env(env);
        let info_bytes = serialize_message_info(info);
        let msg_bytes = serialize_message(msg);

        let (std_result, error_code) = native_call_instantiate_raw(instance.code_checksum, store_handle, env_bytes, info_bytes, msg_bytes);
        if (error_code == 0) {
            deserialize_stdresult(std_result)
        } else {
            new_error_result(error_code, string::utf8(b"native_call_instantiate_raw_error"))
        }
    }

    #[data_struct(T)]
    public fun call_execute<T: drop>(instance: &mut Instance, env: &Env, info: &MessageInfo, msg: &T): Result<Response, Error> {
        let store_handle = table::handle(&mut instance.store);
        let env_bytes = serialize_env(env);
        let info_bytes = serialize_message_info(info);
        let msg_bytes = serialize_message(msg);

        let (std_result, error_code) = native_call_execute_raw(instance.code_checksum, store_handle, env_bytes, info_bytes, msg_bytes);
        if (error_code == 0) {
            deserialize_stdresult(std_result)
        } else {
            new_error_result(error_code, string::utf8(b"native_call_execute_raw_error"))
        }
    }

    #[data_struct(T)]
    public fun call_query<T: drop>(instance: &Instance, env: &Env, msg: &T): Result<Response, Error> {
        let store_handle = table::handle(&instance.store);
        let env_bytes = serialize_env(env);
        let msg_bytes = serialize_message(msg);

        let (std_result, error_code) = native_call_query_raw(instance.code_checksum, store_handle, env_bytes, msg_bytes);
        if (error_code == 0) {
            deserialize_stdresult(std_result)
        } else {
            new_error_result(error_code, string::utf8(b"native_call_query_raw_error"))
        }
    }

    #[data_struct(T)]
    public fun call_migrate<T: drop>(instance: &mut Instance, env: &Env, msg: &T): Result<Response, Error> {
        let store_handle = table::handle(&instance.store);
        let env_bytes = serialize_env(env);
        let msg_bytes = serialize_message(msg);

        let (std_result, error_code) = native_call_migrate_raw(instance.code_checksum, store_handle, env_bytes, msg_bytes);
        if (error_code == 0) {
            deserialize_stdresult(std_result)
        } else {
            new_error_result(error_code, string::utf8(b"native_call_migrate_raw_error"))
        }
    }

    public fun call_reply(instance: &mut Instance, env: &Env, reply: &Reply): Result<Response, Error> {
        let store_handle = table::handle(&mut instance.store);
        let env_bytes = serialize_env(env);
        let msg_bytes = serialize_message(reply);

        let (std_result, error_code) = native_call_reply_raw(instance.code_checksum, store_handle, env_bytes, msg_bytes);
        if (error_code == 0) {
            deserialize_stdresult(std_result)
        } else {
            new_error_result(error_code, string::utf8(b"native_call_reply_raw_error"))
        }
    }

    #[data_struct(T)]
    public fun call_sudo<T: drop>(instance: &mut Instance, env: &Env, msg: &T): Result<Response, Error> {
        let store_handle = table::handle(&mut instance.store);
        let env_bytes = serialize_env(env);
        let msg_bytes = serialize_message(msg);

        let (std_result, error_code) = native_call_sudo_raw(instance.code_checksum, store_handle, env_bytes, msg_bytes);
        if (error_code == 0) {
            deserialize_stdresult(std_result)
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

    // Native function declarations
    native fun native_create_instance(code: vector<u8>, store_handle: ObjectID): (vector<u8>, u32);
    native fun native_destroy_instance(code_checksum: vector<u8>): u32;
    native fun native_call_instantiate_raw(code_checksum: vector<u8>, store_handle: ObjectID, env: vector<u8>, info: vector<u8>, msg: vector<u8>): (vector<u8>, u32);
    native fun native_call_execute_raw(code_checksum: vector<u8>, store_handle: ObjectID, env: vector<u8>, info: vector<u8>, msg: vector<u8>): (vector<u8>, u32);
    native fun native_call_query_raw(code_checksum: vector<u8>, store_handle: ObjectID, env: vector<u8>, msg: vector<u8>): (vector<u8>, u32);
    native fun native_call_migrate_raw(code_checksum: vector<u8>, store_handle: ObjectID, env: vector<u8>, msg: vector<u8>): (vector<u8>, u32);
    native fun native_call_reply_raw(code_checksum: vector<u8>, store_handle: ObjectID, env: vector<u8>, msg: vector<u8>): (vector<u8>, u32);
    native fun native_call_sudo_raw(code_checksum: vector<u8>, store_handle: ObjectID, env: vector<u8>, msg: vector<u8>):(vector<u8>, u32);
}
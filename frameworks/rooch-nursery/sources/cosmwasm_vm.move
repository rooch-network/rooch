// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_nursery::cosmwasm_vm {
    use std::string;
    use std::option::{Self, Option};
    
    use moveos_std::features;
    use moveos_std::result::{Result, ok};

    use rooch_nursery::cosmwasm_std::{Response, Error, Env, MessageInfo, 
        new_error, new_error_result, serialize_env, serialize_message_info, deserialize_response};

    struct Instance {
        id: u64
    }

    public fun from_code(code: vector<u8>): Result<Instance, Error> {
        features::ensure_wasm_enabled();

        let (instance_id, error_code) = native_create_instance(code);
        if (error_code == 0) {
            ok(Instance { id: instance_id })
        } else {
            new_error_result(error_code, string::utf8(b"native_create_instance_error"))
        }
    }
    
   
    public fun call_instantiate(instance: &Instance, env: &Env, info: &MessageInfo, msg: vector<u8>): Result<Response, Error> {
        let (raw_response, error_code) = native_call_instantiate_raw(instance.id, serialize_env(env), serialize_message_info(info), msg);
        if (error_code == 0) {
            ok(deserialize_response(raw_response))
        } else {
            new_error_result(error_code, string::utf8(b"native_call_instantiate_raw_error"))
        }
    }

    public fun call_execute(instance: &Instance, env: &Env, info: &MessageInfo, msg: vector<u8>): Result<Response, Error> {
        let (raw_response, error_code) = native_call_execute_raw(instance.id, serialize_env(env), serialize_message_info(info), msg);
        if (error_code == 0) {
            ok(deserialize_response(raw_response))
        } else {
            new_error_result(error_code, string::utf8(b"native_call_execute_raw_error"))
        }
    }

    public fun call_query(instance: &Instance, env: &Env, msg: vector<u8>): Result<Response, Error> {
        let (raw_response, error_code) = native_call_query_raw(instance.id, serialize_env(env), msg);
        if (error_code == 0) {
            ok(deserialize_response(raw_response))
        } else {
            new_error_result(error_code, string::utf8(b"native_call_query_raw_error"))
        }
    }

    public fun call_migrate(instance: &Instance, env: &Env, msg: vector<u8>): Result<Response, Error> {
        let (raw_response, error_code) = native_call_migrate_raw(instance.id, serialize_env(env), msg);
        if (error_code == 0) {
            ok(deserialize_response(raw_response))
        } else {
            new_error_result(error_code, string::utf8(b"native_call_migrate_raw_error"))
        }
    }

    public fun call_reply(instance: &Instance, env: &Env, msg: vector<u8>): Result<Response, Error> {
        let (raw_response, error_code) = native_call_reply_raw(instance.id, serialize_env(env), msg);
        if (error_code == 0) {
            ok(deserialize_response(raw_response))
        } else {
            new_error_result(error_code, string::utf8(b"native_call_reply_raw_error"))
        }
    }

    public fun call_sudo(instance: &Instance, env: &Env, msg: vector<u8>): Result<Response, Error> {
        let (raw_response, error_code) = native_call_sudo_raw(instance.id, serialize_env(env), msg);
        if (error_code == 0) {
            ok(deserialize_response(raw_response))
        } else {
            new_error_result(error_code, string::utf8(b"native_call_sudo_raw_error"))
        }
    }

    /// Destroys an Instance and releases associated resources.
    public fun destroy_instance(instance: Instance): Option<Error> {
        let Instance { id } = instance;
        let error_code = native_destroy_instance(id);
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
    native fun native_create_instance(code: vector<u8>): (u64, u32);
    native fun native_destroy_instance(instance_id: u64): u32;
    native fun native_call_instantiate_raw(instance_id: u64, env: vector<u8>, info: vector<u8>, msg: vector<u8>): (vector<u8>, u32);
    native fun native_call_execute_raw(instance_id: u64, env: vector<u8>, info: vector<u8>, msg: vector<u8>): (vector<u8>, u32);
    native fun native_call_query_raw(instance_id: u64, env: vector<u8>, msg: vector<u8>): (vector<u8>, u32);
    native fun native_call_migrate_raw(instance_id: u64, env: vector<u8>, msg: vector<u8>): (vector<u8>, u32);
    native fun native_call_reply_raw(instance_id: u64, env: vector<u8>, msg: vector<u8>): (vector<u8>, u32);
    native fun native_call_sudo_raw(instance_id: u64, env: vector<u8>, msg: vector<u8>):(vector<u8>, u32);
}
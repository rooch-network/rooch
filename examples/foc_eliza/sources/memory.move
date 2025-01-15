// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module foc_eliza::memory {

    use std::string::{Self, String};
    use std::option::{Self, Option};
    
    use moveos_std::json;
    use moveos_std::object::{Self, Object};

    use foc_eliza::types::{Content};
    use foc_eliza::agent_cap::{Self, MemoryCap};

    #[data_struct]
    struct Memory has store, copy, drop {
        id: String,
        userId: String,
        agentId: String,
        createdAt: Option<u64>,
        content: Content,
        character: String,
        embedding: vector<u128>,
        roomId: String,
        unique: bool,
        similarity: Option<u128>,
    }

    struct MemoryStore has key {
        agent_account: address,
    }

    public fun new_memory(
        id: String, 
        userId: String, 
        agentId: String, 
        createdAt: Option<u64>, 
        content: Content, 
        character: String, 
        embedding: vector<u128>, 
        roomId: String, 
        unique: bool, 
        similarity: Option<u128>
    ) : Memory {
        Memory {
            id,
            userId,
            agentId,
            createdAt,
            content,
            character,
            embedding,
            roomId,
            unique,
            similarity,
        }
    }

    fun init_memory_store(agent_account: address){
        let memory_store = MemoryStore {
            agent_account,
        };
        let memory_store_obj = object::new_account_named_object(agent_account, memory_store);
        object::transfer_extend(memory_store_obj, agent_account);
    }

    fun init_or_borrow_memory_store(agent_account: address) : &mut Object<MemoryStore> {
        let memory_store_obj_id = object::account_named_object_id<MemoryStore>(agent_account);
        let memory_store_obj = if (object::exists_object(memory_store_obj_id)) {
            object::borrow_mut_object_extend<MemoryStore>(memory_store_obj_id)
        } else {
            init_memory_store(agent_account);
            object::borrow_mut_object_extend<MemoryStore>(memory_store_obj_id)
        };
        memory_store_obj
    }

    public fun create_memory(cap: &mut Object<MemoryCap>, memory: Memory) {
        let agent_account = agent_cap::check_memory_create_cap(cap);
        let memory_store_obj = init_or_borrow_memory_store(agent_account);
        object::add_field(memory_store_obj, memory.id, memory);
    }

    public entry fun create_memory_entry(cap: &mut Object<MemoryCap>, memory_json: String) {
        let memory = json::from_json<Memory>(string::into_bytes(memory_json));
        create_memory(cap, memory);
    }

    public fun remove_memory(cap: &mut Object<MemoryCap>, memory_id: String) {
        let agent_account = agent_cap::check_memory_remove_cap(cap);
        let memory_store_obj = init_or_borrow_memory_store(agent_account);
        let _memory: Memory = object::remove_field(memory_store_obj, memory_id);
    }

    public entry fun remove_memory_entry(cap: &mut Object<MemoryCap>, memory_id: String) {
        remove_memory(cap, memory_id);
    }

    public fun get_memory_by_id(agent_account: address, memory_id: String) : Option<Memory> {
        let memory_store_obj_id = object::account_named_object_id<MemoryStore>(agent_account);
        if (!object::exists_object(memory_store_obj_id)) {
            return option::none()
        };
        let memory_store_obj = object::borrow_object<MemoryStore>(memory_store_obj_id);
        if (!object::contains_field(memory_store_obj, memory_id)) {
            return option::none()
        };
        let memory = object::borrow_field(memory_store_obj, memory_id);
        option::some(*memory)
    }

    #[test]
    fun test_memory_store(){
        let agent_account = @0x42;
    
        let memory_id = string::utf8(b"67561f62-d8f9-4f2b-a2c8-61ce15a73749");
        let user_id = string::utf8(b"362b9cda-fd51-44c2-bbb2-83af3baf793b");
        let agent_id = string::utf8(b"67561f62-d8f9-4f2b-a2c8-61ce15a73749");
        let created_at = 1710384000;
        let content = foc_eliza::types::new_content(string::utf8(b"Hello, world!"), option::none(), option::none(), option::none(), option::none(), vector[]);
        let character = string::utf8(b"Eliza");
        let embedding = vector[1, 2, 3];
        let room_id = string::utf8(b"67561f62-d8f9-4f2b-a2c8-61ce15a73749");
        let unique = false;
        let similarity = option::none();

        let memory = new_memory(memory_id, user_id, agent_id, option::some(created_at), content, character, embedding, room_id, unique, similarity);
        let cap = agent_cap::issue_memory_cap_for_test(agent_account, true, true, true);
        create_memory(&mut cap, memory);
        let memory_opt = get_memory_by_id(agent_account, memory_id);
        assert!(option::is_some(&memory_opt), 1001);
        let memory = option::destroy_some(memory_opt);
        assert!(memory.id == memory_id, 1002);
        remove_memory(&mut cap, memory_id);
        let memory_opt = get_memory_by_id(agent_account, memory_id);
        assert!(option::is_none(&memory_opt), 1003);

        agent_cap::destroy_memory_cap(cap);
    }
}
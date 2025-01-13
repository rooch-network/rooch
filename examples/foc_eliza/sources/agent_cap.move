// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module foc_eliza::agent_cap {

    use moveos_std::object::{Self, Object};
    use moveos_std::event;
    
    const ErrorAgentCapNotFound: u64 = 1;
    const ErrorCallerHasNoMemoryCap: u64 = 2;
    const ErrorCallerHasNoMemoryCreateCap: u64 = 3;
    const ErrorCallerHasNoMemoryDeleteCap: u64 = 4;
    const ErrorCallerHasNoMemoryUpdateCap: u64 = 5;

    friend foc_eliza::character;

    struct AgentCap has store, key {
        agent_account: address,
    }

    /// A cap for managing the memory of an agent.
    struct MemoryCap has store, key {
        agent_account: address,
        create: bool,
        remove: bool,
        update: bool,
    }

    struct AgentCapDestroyedEvent has copy, drop, store {
        agent_account: address,
    }

    struct MemoryCapDestroyedEvent has copy, drop, store {
        agent_account: address,
        create: bool,
        remove: bool,
        update: bool,
    }

    public(friend) fun new_agent_cap(agent_account: address) : Object<AgentCap> {
        let cap = AgentCap {
            agent_account,
        };
        // every agent account only has one cap
        object::new_account_named_object(agent_account, cap)
    }

    public(friend) fun new_memory_cap(agent_account: address, create: bool, remove: bool, update: bool) : Object<MemoryCap> {
        let cap = MemoryCap {
            agent_account,
            create,
            remove,
            update,
        };
        object::new(cap)
    }

    public fun destroy_agent_cap(cap: Object<AgentCap>) {
        let agent_cap = object::remove(cap);
        let AgentCap { agent_account } = agent_cap;
        event::emit(AgentCapDestroyedEvent { agent_account });
    }

    public fun destroy_memory_cap(cap: Object<MemoryCap>) {
        let memory_cap = object::remove(cap);
        let MemoryCap { agent_account, create, remove, update } = memory_cap;
        event::emit(MemoryCapDestroyedEvent { agent_account, create, remove, update });
    }

    public fun borrow_mut_agent_cap(caller: &signer, agent_account: address) : &mut Object<AgentCap> {
        let cap_obj_id = object::account_named_object_id<AgentCap>(agent_account);
        assert!(object::exists_object(cap_obj_id), ErrorAgentCapNotFound);
        object::borrow_mut_object<AgentCap>(caller, cap_obj_id)
    }

    public fun check_agent_cap(cap: &mut Object<AgentCap>) : address {
        let cap = object::borrow(cap);
        cap.agent_account
    }

    public fun check_memory_create_cap(cap: &mut Object<MemoryCap>) : address {
        let cap = object::borrow(cap);
        assert!(cap.create, ErrorCallerHasNoMemoryCreateCap);
        cap.agent_account
    }

    public fun check_memory_remove_cap(cap: &mut Object<MemoryCap>) : address {
        let cap = object::borrow(cap);
        assert!(cap.remove, ErrorCallerHasNoMemoryDeleteCap);
        cap.agent_account
    }

    public fun check_memory_update_cap(cap: &mut Object<MemoryCap>) : address {
        let cap = object::borrow(cap);
        assert!(cap.update, ErrorCallerHasNoMemoryUpdateCap);
        cap.agent_account
    }


    #[test_only]
    public fun issue_agent_cap_for_test(agent_account: address) : Object<AgentCap> {
        new_agent_cap(agent_account)
    }

    #[test_only]
    public fun issue_memory_cap_for_test(agent_account: address, create: bool, remove: bool, update: bool) : Object<MemoryCap> {
        new_memory_cap(agent_account, create, remove, update)
    }

}

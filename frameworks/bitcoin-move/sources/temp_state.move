// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// This module is used to store temporary states for UTXO and Inscription.
module bitcoin_move::temp_state {

    use std::string::String;
    use std::vector;
    use moveos_std::type_info;
    use moveos_std::bag::{Self, Bag};

    friend bitcoin_move::utxo;
    friend bitcoin_move::ord;
    friend bitcoin_move::bbn;

    const MAX_TEMP_STATES: u64 = 20;

    const ErrorMaxTempStateExceeded: u64 = 1;
    const ErrorTempStateNotFound: u64 = 2;

    struct TempState has store {
        names: vector<String>,
        states: Bag,
    }

    public(friend) fun new(): TempState {
        TempState{
            names: vector::empty(),
            states: bag::new_dropable(),
        }
    }

    public(friend) fun add_state<T: drop + store>(self: &mut TempState, value: T) {
        assert!(vector::length(&self.names) < MAX_TEMP_STATES, ErrorMaxTempStateExceeded);
        let key = type_info::type_name<T>();
        bag::add_dropable(&mut self.states, key, value);
        vector::push_back(&mut self.names, key);
    }

    public(friend) fun borrow_state<T: drop + store>(self: &TempState): &T {
        assert!(contains_state<T>(self), ErrorTempStateNotFound);
        let key = type_info::type_name<T>();
        bag::borrow(&self.states, key)
    }

    public(friend) fun borrow_mut_state<T: drop + store>(self: &mut TempState): &mut T {
        assert!(contains_state<T>(self), ErrorTempStateNotFound);
        let key = type_info::type_name<T>();
        bag::borrow_mut(&mut self.states, key)
    }

    public(friend) fun remove_state<T: drop + store>(self: &mut TempState): T {
        assert!(contains_state<T>(self), ErrorTempStateNotFound);
        let key = type_info::type_name<T>();
        self.names = vector::remove_value(&mut self.names, &key);
        bag::remove(&mut self.states, key)
    }

    public(friend) fun contains_state<T: drop + store>(self: &TempState): bool {
        let key = type_info::type_name<T>();
        bag::contains(&self.states, key)
    }

    public(friend) fun remove(self: TempState) : vector<String> {
        let TempState{names, states} = self;
        bag::drop(states);
        names
    }

}
// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// This module is a empty module that does nothing
/// It is used to test or demo some use cases
module rooch_framework::empty{
    use moveos_std::object;

    struct Empty has key{}

    fun init(){
        let obj = object::new_named_object(Empty{});
        object::to_shared(obj);        
    }
    
    /// This empty function does nothing
    public entry fun empty(){
        // Just do nothing
    }

    public entry fun empty_with_signer(_sender: &signer) {
        // Just do nothing
    }
}

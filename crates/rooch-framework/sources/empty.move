// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// This module is a empty module that does nothing
/// It is used to test or demo some use cases
module rooch_framework::empty{
    use moveos_std::context::{Self, Context};
    use moveos_std::object;

    struct Empty has key{}

    fun init(ctx: &mut Context){
        let obj = context::new_named_object(ctx, Empty{});
        object::to_shared(obj);        
    }
    
    /// This empty function does nothing
    public entry fun empty(){
        //Just do nothing
    }

}

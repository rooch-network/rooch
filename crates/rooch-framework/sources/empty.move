// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// This module is a empty module that does nothing
/// It is used to test or demo some use cases
module rooch_framework::empty{
    use moveos_std::context::{Self, Context};

    struct Empty has key{}

    fun init(ctx: &mut Context){
        context::new_singleton(ctx, Empty{});        
    }
    
    /// This empty function does nothing
    public entry fun empty(){
        //Just do nothing
    }

}

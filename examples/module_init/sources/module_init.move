// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_examples::module_init {
    use std::debug;
    use std::string::{Self, String};
    use moveos_std::context::Context;
    use moveos_std::account_storage;
    use moveos_std::signer;
    
    struct InitConfig has key{
        is_init: bool,
    }

    /// This init function will be called when the module is first deployed.
    fun init(ctx: &mut Context) {
        let module_signer = signer::module_signer<InitConfig>();
        account_storage::global_move_to(ctx, &module_signer, InitConfig{is_init: true});
        debug::print<String>(&string::utf8(b"module init finish"));
    }

    public fun is_init(ctx: &Context): bool{
        account_storage::global_borrow<InitConfig>(ctx, @rooch_examples).is_init
    }
}

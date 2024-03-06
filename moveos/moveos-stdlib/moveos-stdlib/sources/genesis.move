// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module moveos_std::genesis {
    use moveos_std::context::Context;
    use moveos_std::move_module;

    fun init(_ctx: &mut Context){
        std::debug::print(&b"moveos_std genesis");
        move_module::create_module_store();
    }
}
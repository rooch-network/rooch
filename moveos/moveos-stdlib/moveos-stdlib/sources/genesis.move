// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module moveos_std::genesis {
    use moveos_std::move_module;

    fun init(){
        move_module::init_module_store();
    }
}
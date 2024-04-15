// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module moveos_std::genesis {
    use moveos_std::move_module;
    use moveos_std::features;

    fun init(){
        move_module::init_module_store();
        features::init_feature_store();
    }

    #[test_only]
    /// init the genesis context for test
    public fun init_for_test(){
        init()
    }
}
// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

#[test_only]
module rooch_framework::chain_id_test{
    use rooch_framework::chain_id;
    
    #[test]
    fun test_get_chain_id(){
        let ctx = rooch_framework::genesis::init_for_test();
        let _id = chain_id::chain_id(&ctx);
        moveos_std::context::drop_test_context(ctx);
    }
}
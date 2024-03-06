// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_examples::wasm_execution {
   use moveos_std::context::{Context};
   use moveos_std::account;
   use moveos_std::wasm;
   use std::debug;

   public fun run(ctx: &Context) {
      let wasm_bytes: vector<u8> = x"a11ceb";
      let wasm_instance_id = wasm::create_wasm_instance(ctx, wasm_bytes);
      debug::print(&33333333333);
      debug::print(&wasm_instance_id);
   }
}

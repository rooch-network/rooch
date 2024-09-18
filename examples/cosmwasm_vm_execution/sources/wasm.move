// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_examples::cosmwasm_vm_execution {
   use std::debug;
   use std::string;
   use std::vector;
   use std::option;

   use moveos_std::result;
   use rooch_nursery::cosmwasm_vm;

   entry public fun run_cosmwasm_bitseed_generator(_account: &signer, wasm_bytes: vector<u8>) {
      debug::print(&string::utf8(b"run_cosmwasm_bitseed_generator start"));

      // 1. create wasm VM instance (required step)
      let instance_result = cosmwasm_vm::from_code(wasm_bytes);

      debug::print(&string::utf8(b"run_cosmwasm_bitseed_generator debug 1"));

      // Verify that the result is successful
      let instance = result::assert_ok(instance_result, 1); // Use assert_ok here

      debug::print(&string::utf8(b"run_cosmwasm_bitseed_generator debug 2"));

      // 2. Verify some properties of the instance
      // Note: The specific verification method may need to be adjusted based on the actual definition of the Instance structure
      assert!(vector::length(&cosmwasm_vm::code_checksum(&instance)) > 0, 2);

      debug::print(&string::utf8(b"run_cosmwasm_bitseed_generator debug 3"));

      // 3. Destroy the instance
      let destroy_result = cosmwasm_vm::destroy_instance(instance);
      assert!(option::is_none(&destroy_result), 3);
   }
}

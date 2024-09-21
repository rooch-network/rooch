// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_examples::cosmwasm_vm_execution {
   use std::debug;
   use std::string;
   use std::vector;
   use std::option;

   use moveos_std::result;
   use rooch_nursery::cosmwasm_std;
   use rooch_nursery::cosmwasm_vm;

   #[data_struct]
   struct InstantiateMsg has store, copy, drop {}

   #[data_struct]
   struct Add has store, copy, drop {
      value: u64
   }

   #[data_struct]
   struct GetValue has store, copy, drop {}

   #[data_struct]
   struct ExecuteMsg has store, copy, drop {
      add: Add
   }

   #[data_struct]
   struct QueryMsg has store, copy, drop {
      get_value: GetValue
   }

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

      let env = cosmwasm_std::current_env();
      let info = cosmwasm_std::current_message_info();

      // 3. call instantiate of the instance
      let msg = InstantiateMsg{};
      let instantiate_result = cosmwasm_vm::call_instantiate(&mut instance, &env, &info, &msg);
      let instantiate_resp = result::assert_ok(instantiate_result, 3); // Use assert_ok here
      debug::print(&string::utf8(b"instantiate_resp:"));
      debug::print(&instantiate_resp);

      // 4. call execute of the instance
      let msg = ExecuteMsg{
         add: Add {
            value: 1
         }
      };

      let execute_result = cosmwasm_vm::call_execute(&mut instance, &env, &info, &msg);
      let execute_resp = result::assert_ok(execute_result, 4); // Use assert_ok here
      debug::print(&string::utf8(b"execute_resp:"));
      debug::print(&execute_resp);

      // 5. call query of the instance
      let msg = QueryMsg{
         get_value: GetValue {}
      };

      let query_result = cosmwasm_vm::call_query(&instance, &env, &msg);
      let query_resp = result::assert_ok(query_result, 4); // Use assert_ok here
      debug::print(&string::utf8(b"query_resp:"));
      debug::print(&query_resp);

      // 6. Destroy the instance
      let destroy_result = cosmwasm_vm::destroy_instance(instance);
      assert!(option::is_none(&destroy_result), 4);
   }
}

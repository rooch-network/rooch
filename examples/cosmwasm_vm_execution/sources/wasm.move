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
   struct InstantiateMsg has store, copy, drop {
      initial_value: u64
   }

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

   #[data_struct]
   struct MigreateMsg has store, copy, drop {
      new_value: u64
   }

   #[data_struct]
   struct UpdateValue has store, copy, drop {
      value: u64 
   }

   #[data_struct]
   struct SudoMsg has store, copy, drop {
      update_value: UpdateValue
   }

   entry public fun run_cosmwasm_bitseed_generator(_account: &signer, wasm_bytes: vector<u8>) {
      debug::print(&string::utf8(b"run_cosmwasm_bitseed_generator start"));

      // 1. create wasm VM instance (required step)
      let instance_result = cosmwasm_vm::from_code(wasm_bytes);
      debug::print(&string::utf8(b"instance_result:"));
      debug::print(&instance_result);

      // Verify that the result is successful
      let instance = result::assert_ok(instance_result, 1); // Use assert_ok here

      // 2. Verify some properties of the instance
      // Note: The specific verification method may need to be adjusted based on the actual definition of the Instance structure
      assert!(vector::length(&cosmwasm_vm::code_checksum(&instance)) > 0, 2);

      let env = cosmwasm_std::current_env();
      let info = cosmwasm_std::current_message_info();

      // 3. call instantiate of the instance
      let msg = InstantiateMsg{
         initial_value: 1
      };
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
      let query_resp = result::assert_ok(query_result, 5); // Use assert_ok here
      debug::print(&string::utf8(b"query_resp:"));
      debug::print(&query_resp);

      // 6. call migrate of the instance
      let msg = MigreateMsg{
         new_value: 2
      };

      let migrate_result = cosmwasm_vm::call_migrate(&mut instance, &env, &msg);
      let migrate_resp = result::assert_ok(migrate_result, 6); // Use assert_ok here
      debug::print(&string::utf8(b"migrate_resp:"));
      debug::print(&migrate_resp);

      // 7. call reply of the instance
      let resp = cosmwasm_std::new_sub_msg_response();
      let msg = cosmwasm_std::new_reply(1, cosmwasm_std::new_binary(b"hello"), 10, resp);

      let reply_result = cosmwasm_vm::call_reply(&mut instance, &env, &msg);
      let reply_resp = result::assert_ok(reply_result, 7); // Use assert_ok here
      debug::print(&string::utf8(b"reply_resp:"));
      debug::print(&reply_resp);

      // 8. call sudo of the instance
      let msg = SudoMsg{
         update_value: UpdateValue {
            value: 2
         }
      };

      let sudo_result = cosmwasm_vm::call_sudo(&mut instance, &env, &msg);
      let sudo_resp = result::assert_ok(sudo_result, 8); // Use assert_ok here
      debug::print(&string::utf8(b"sudo_resp:"));
      debug::print(&sudo_resp);

      // 9. Destroy the instance
      let destroy_result = cosmwasm_vm::destroy_instance(instance);
      assert!(option::is_none(&destroy_result), 9);
   }
}

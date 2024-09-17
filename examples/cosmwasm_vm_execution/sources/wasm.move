// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_examples::cosmwasm_vm_execution {
   use rooch_nursery::wasm;
   use std::debug;
   use std::string;
   use std::vector;
   use std::option;

   use moveos_std::cbor;
   
   entry public fun run_generator_cosmwasm(wasm_bytes: vector<u8>) {
      debug::print(&string::utf8(b"run_generator start"));

      // 1. create wasm VM instance (required step)
      let wasm_instance = wasm::create_wasm_instance(wasm_bytes);
      debug::print(&string::utf8(b"wasm_instance:"));
      debug::print(&wasm_instance);

      // 2. inscribe_verify
      let function_name = b"inscribe_verify";
      let deploy_args = x"81a166686569676874a264747970656572616e67656464617461a2636d696e01636d61781903e8";
      let seed = x"3330376539626235323861613230393034366530623033333631616233346138396663306331323332376439396436323966633639663438323266363837";
      let user_input = x"";
      let attributes_output = x"a26668656967687418c56269646d68656c6c6f5f62697473656564";

      let buffer = pack_inscribe_generate_args(deploy_args, seed, user_input);
      std::debug::print(&string::utf8(b"buffer:"));
      std::debug::print(&buffer);

      let arg_with_length = wasm::add_length_with_data(buffer);

      let arg_list = vector::empty<vector<u8>>();
      vector::push_back(&mut arg_list, arg_with_length);
      vector::push_back(&mut arg_list, attributes_output);
      let memory_args_list = wasm::create_memory_wasm_args(&mut wasm_instance, function_name, arg_list);
      std::debug::print(&string::utf8(b"memory_args_list:"));
      std::debug::print(&memory_args_list);

      let ret_val_option = wasm::execute_wasm_function_option(&mut wasm_instance, function_name, memory_args_list);
      assert!(option::is_some(&ret_val_option), 1);

      let ret_val = option::destroy_some(ret_val_option);
      std::debug::print(&string::utf8(b"ret_val:"));
      debug::print(&ret_val);

      // 3. release the wasm VM instance (required step)
      wasm::release_wasm_instance(wasm_instance);
   }

   #[data_struct]
   struct InscribeGenerateArgs has copy, drop, store {
      attrs: vector<u16>,
      seed: std::string::String,
      user_input: std::string::String,
   }

   fun pack_inscribe_generate_args(deploy_args: vector<u8>, seed: vector<u8>, user_input: vector<u8>): vector<u8>{
      let attrs = vector::empty();

      let i=0;
      let len = vector::length(&deploy_args);
      while (i < len) {
         vector::push_back(&mut attrs, (*vector::borrow(&deploy_args, i) as u16));
         i = i + 1;
      };

      let args = InscribeGenerateArgs{
         attrs: attrs,
         seed: string::utf8(seed),
         user_input: string::utf8(user_input)
      };

      cbor::to_cbor(&args)
   }
}

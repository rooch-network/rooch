// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_examples::bitseed_runner {
   use std::string;
   use std::vector;
   use moveos_std::string_utils::parse_u64;
   use moveos_std::simple_map;
   use moveos_std::json;
   use bitcoin_move::bitseed;
   use bitcoin_move::bitseed::{MintOp, DeployOp};
   use bitcoin_move::ord::{InscriptionID, Inscription};
   use bitcoin_move::ord;
   use moveos_std::object;

   struct BitseedRunnerStore has key,store,drop {
      index: u64
   }

   fun init() {
      let store = BitseedRunnerStore {
         index: 0
      };
      let store_obj = object::new_named_object(store);
      object::to_shared(store_obj);
   }

   fun hash_string_to_u32(str_bytes: vector<u8>): u32 {
      let hash: u32 = 0x811c9dc5;
      let prime: u32 = 0x1000193;

      vector::for_each(str_bytes, |u8_byte| {
         hash = hash ^ (u8_byte as u32);
         hash = hash * prime;
      });

      hash
   }

   fun get_generator_bytes(inscription_index: u64): vector<u8> {
      let ret_bytes = vector::empty<u8>();

      let inscription_id = ord::get_inscription_id_by_index(inscription_index);
      let object_id = object::custom_object_id<InscriptionID, Inscription>(*inscription_id);
      let inscription_obj = object::borrow_object<Inscription>(object_id);
      let inscrption = object::borrow(inscription_obj);
      let generator_json_map = ord::json_body(inscrption);

      let tick = *simple_map::borrow(&generator_json_map, &string::utf8(b"tick"));
      let op = *simple_map::borrow(&generator_json_map, &string::utf8(b"op"));
      let amount = *simple_map::borrow(&generator_json_map, &string::utf8(b"amount"));
      let amount = parse_u64(&amount);
      let inner_connten = *simple_map::borrow(&generator_json_map, &string::utf8(b"content"));
      let inner_content_map = json::to_map(string::into_bytes(inner_connten));
      let content_type = *simple_map::borrow(&inner_content_map, &string::utf8(b"content_type"));
      let content_body = *simple_map::borrow(&inner_content_map, &string::utf8(b"body"));
      if ((tick == string::utf8(b"generator"))
         && (op == string::utf8(b"mint"))
         && (amount == 1)
         && (content_type == string::utf8(b"application/wasm"))) {

         ret_bytes = string::into_bytes(content_body);
      };

      simple_map::drop(generator_json_map);
      simple_map::drop(inner_content_map);

      ret_bytes
   }

   entry public fun run(seed: vector<u8>) {
      let object_id = object::named_object_id<BitseedRunnerStore>();
      let bitseed_runner_store = object::borrow_mut_object_shared<BitseedRunnerStore>(object_id);
      let runner = object::borrow_mut(bitseed_runner_store);
      let index = runner.index;

      // get a Inscription by InscriptionId
      let inscription_id = ord::get_inscription_id_by_index(index);
      // let id = object::account_named_object_id<Inscription>(to_address);
      let object_id = object::custom_object_id<InscriptionID, Inscription>(*inscription_id);
      let inscription_obj = object::borrow_object<Inscription>(object_id);

      let bitseed_mint_key = bitseed::bitseed_mint_key();
      if (object::contains_field(inscription_obj, bitseed_mint_key)) {
         // get MintOp from the child fields of the Inscription
         let bitseed_mint_op = object::borrow_field<Inscription, vector<u8>, MintOp>(inscription_obj, bitseed_mint_key);
         if (bitseed::mint_op_is_valid(bitseed_mint_op) == 0) {
            let minted_attributes = bitseed::mint_op_attributes(bitseed_mint_op);
            let minted_attributes_map = json::to_map(minted_attributes);
            let inner_attributes_string = *simple_map::borrow(&minted_attributes_map, &string::utf8(b"attributes"));
            let inner_attributes_map = json::to_map(string::into_bytes(inner_attributes_string));
            let user_input_id_string = *simple_map::borrow(&inner_attributes_map, &string::utf8(b"id"));


            // Find the deploy_args and the generator bytes that matched the current MintOp
            let generator_bytes = vector::empty<u8>();
            let deploy_args = vector::empty<u8>();
            let bitseed_deploy_key = bitseed::bitseed_deploy_key();
            if (object::contains_field(inscription_obj, bitseed_mint_key)) {
               let bitseed_deploy_op = object::borrow_field<Inscription, vector<u8>, DeployOp>(inscription_obj, bitseed_deploy_key);
               let generator_id_string = bitseed::deploy_op_generator(bitseed_deploy_op);
               let generator_id = parse_u64(&generator_id_string);
               generator_bytes = get_generator_bytes(generator_id);
               deploy_args = bitseed::deploy_op_args(bitseed_deploy_op);
            };

            // execute the verify_generate function
            if (vector::length(&generator_bytes)>0 && vector::length(&deploy_args)>0) {
               let verify_result = bitseed::inscribe_verify(generator_bytes, deploy_args, seed,
                  string::into_bytes(user_input_id_string), minted_attributes);
               if (verify_result) {
                  // Update the valid status of MintOp or save the MintOp to another store.
               }
            };

            simple_map::drop(minted_attributes_map);
            simple_map::drop(inner_attributes_map);

            index = index + 1;
            let store = BitseedRunnerStore {
               index
            };
            let store_obj = object::new_named_object(store);
            object::to_shared(store_obj);
         }
      }
   }
}

// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_examples::bitseed_runner {

   use moveos_std::object;

   use bitcoin_move::ord::{InscriptionID, Inscription};
   use bitcoin_move::ord;
   use rooch_nursery::bitseed;

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

   entry public fun run() {
      let object_id = object::named_object_id<BitseedRunnerStore>();
      let bitseed_runner_store = object::borrow_mut_object_shared<BitseedRunnerStore>(object_id);
      let runner = object::borrow_mut(bitseed_runner_store);
      let index = runner.index;

      // get a Inscription by InscriptionId
      let inscription_id = ord::get_inscription_id_by_index(index);
      let object_id = object::custom_object_id<InscriptionID, Inscription>(*inscription_id);
      let inscription_obj = object::borrow_object<Inscription>(object_id);
      let inscription = object::borrow(inscription_obj);

      bitseed::process_inscription(inscription);

      index = index + 1;
      let store = BitseedRunnerStore {
         index
      };
      let store_obj = object::new_named_object(store);
      object::to_shared(store_obj);
   }
}

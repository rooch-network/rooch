// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_examples::bitseed_runner {
   use moveos_std::object;
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

   public entry fun run() {
      let object_id = object::named_object_id<BitseedRunnerStore>();
      let bitseed_runner_store = object::borrow_mut_object_shared<BitseedRunnerStore>(object_id);
      let runner = object::borrow_mut(bitseed_runner_store);
     
      let latest_height = ord::inscription_latest_height();
      let current_index = runner.index;

      if (current_index < latest_height) {
         // get a Inscription by InscriptionId
         let inscription_id = ord::get_inscription_id_by_index(current_index);
         let inscription = ord::borrow_inscription_by_id(*inscription_id);

         bitseed::process_inscription(inscription);

         runner.index = current_index + 1;
      }
   }
}

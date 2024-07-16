// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_examples::bitseed_runner {

   use std::option;
   use moveos_std::object::{Self, Object};
   use moveos_std::event_queue::{Self, Subscriber};
   use bitcoin_move::ord::{Self, Inscription};
   use rooch_nursery::bitseed;

   struct BitseedRunnerStore has key {
      subscriber: Object<Subscriber<ord::NewInscriptionEvent>>,
   }

   fun init() {
      let subscriber = event_queue::subscribe<ord::NewInscriptionEvent>(bitseed::metaprotocol());
      let store = BitseedRunnerStore {
         subscriber,
      };
      let store_obj = object::new_named_object(store);
      object::to_shared(store_obj);
   }

   public entry fun run() {
      let object_id = object::named_object_id<BitseedRunnerStore>();
      let bitseed_runner_store = object::borrow_mut_object_shared<BitseedRunnerStore>(object_id);
      let runner = object::borrow_mut(bitseed_runner_store);
     
      let event_opt = event_queue::consume(&mut runner.subscriber);
      
      if (option::is_some(&event_opt)){
         let event = option::destroy_some(event_opt);
         let (_metaprotocol, _sequence_number, inscription_obj_id) = ord::upack_new_inscription_event(event);
         let inscription_obj = object::borrow_object<Inscription>(inscription_obj_id);
         let inscription = object::borrow(inscription_obj);
         bitseed::process_inscription(inscription);
      };
   }
}

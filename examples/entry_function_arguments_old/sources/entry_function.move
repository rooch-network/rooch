// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_examples::entry_function {
   use moveos_std::event;
   use moveos_std::object_id::ObjectID;
   

   fun init() {
   }

   struct BoolEvent has drop {
      value: bool
   }
   public entry fun emit_bool(value: bool) {
      event::emit<BoolEvent>(BoolEvent { value });
   }

   struct U8Event has drop {
        value: u8
   }
   public entry fun emit_u8(value: u8) {
      event::emit<U8Event>(U8Event { value });
   }

   struct U16Event has drop {
      value: u16
   }
   public entry fun emit_u16(value: u16) {
      event::emit<U16Event>(U16Event { value });
   }

   struct U32Event has drop {
      value: u32
   }
   public entry fun emit_u32(value: u32) {
      event::emit<U32Event>(U32Event { value });
   }

   struct U64Event has drop {
      value: u64
   }
   public entry fun emit_u64(value: u64) {
      event::emit<U64Event>(U64Event { value });
   }

   struct U128Event has drop {
      value: u128
   }
   public entry fun emit_u128(value: u128) {
      event::emit<U128Event>(U128Event { value });
   }

   struct U256Event has drop {
      value: u256
   }
   public entry fun emit_u256(value: u256) {
      event::emit<U256Event>(U256Event { value });
   }

   struct AddressEvent has drop {
      value: address
   }
   public entry fun emit_address(value: address) {
      event::emit<AddressEvent>(AddressEvent { value });
   }

   struct ObjectIdEvent has drop {
      value: ObjectID
   }
   public entry fun emit_object_id(value: ObjectID) {
      event::emit<ObjectIdEvent>(ObjectIdEvent { value });
   }

   struct StringEvent has drop {
      value: std::string::String
   }
   public entry fun emit_string(value: std::string::String) {
      event::emit<StringEvent>(StringEvent { value });
   }

   struct VecU8Event has drop {
      value: vector<u8>
   }
   public entry fun emit_vec_u8(value: vector<u8>) {
      event::emit<VecU8Event>(VecU8Event { value });
   }

   struct VecObjectIDEvent has drop {
      value: vector<ObjectID>
   }
   
   public entry fun emit_vec_object_id(value: vector<ObjectID>) {
      event::emit<VecObjectIDEvent>(VecObjectIDEvent { value });
   }

   // public entry fun emit_mix(value1: u8, value2: vector<ObjectID>) {
   //    event::emit<U8Event>(U8Event { value: value1 });
   //    event::emit<VecObjectIDEvent>(VecObjectIDEvent { value: value2 });     
   // }
}

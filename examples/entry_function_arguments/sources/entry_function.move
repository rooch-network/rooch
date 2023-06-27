module rooch_examples::entry_function {
   use moveos_std::event;
   use moveos_std::object_id::ObjectID;
   use moveos_std::storage_context::StorageContext;

   struct BoolEvent has key {
      value: bool
   }
   public entry fun emit_bool(ctx: &mut StorageContext, value: bool) {
      events::emit_event<BoolEvent>(ctx, BoolEvent { value });
   }

   struct U8Event has key {
        value: u8
   }
   public entry fun emit_u8(ctx: &mut StorageContext, value: u8) {
      events::emit_event<U8Event>(ctx, U8Event { value });
   }

   struct U16Event has key {
      value: u16
   }
   public entry fun emit_u16(ctx: &mut StorageContext, value: u16) {
      events::emit_event<U16Event>(ctx, U16Event { value });
   }

   struct U32Event has key {
      value: u32
   }
   public entry fun emit_u32(ctx: &mut StorageContext, value: u32) {
      events::emit_event<U32Event>(ctx, U32Event { value });
   }

   struct U64Event has key {
      value: u64
   }
   public entry fun emit_u64(ctx: &mut StorageContext, value: u64) {
      events::emit_event<U64Event>(ctx, U64Event { value });
   }

   struct U128Event has key {
      value: u128
   }
   public entry fun emit_u128(ctx: &mut StorageContext, value: u128) {
      events::emit_event<U128Event>(ctx, U128Event { value });
   }

   struct U256Event has key {
      value: u256
   }
   public entry fun emit_u256(ctx: &mut StorageContext, value: u256) {
      events::emit_event<U256Event>(ctx, U256Event { value });
   }

   struct AddressEvent has key {
      value: address
   }
   public entry fun emit_address(ctx: &mut StorageContext, value: address) {
      events::emit_event<AddressEvent>(ctx, AddressEvent { value });
   }

   struct ObjectIdEvent has key {
      value: ObjectID
   }
   public entry fun emit_object_id(ctx: &mut StorageContext, value: ObjectID) {
      events::emit_event<ObjectIdEvent>(ctx, ObjectIdEvent { value });
   }

   struct StringEvent has key {
      value: std::string::String
   }
   public entry fun emit_string(ctx: &mut StorageContext, value: std::string::String) {
      events::emit_event<StringEvent>(ctx, StringEvent { value });
   }

   struct VecU8Event has key {
      value: vector<u8>
   }
   public entry fun emit_vec_u8(ctx: &mut StorageContext, value: vector<u8>) {
      events::emit_event<VecU8Event>(ctx, VecU8Event { value });
   }

   struct VecObjectIDEvent has key {
      value: vector<ObjectID>
   }
   public entry fun emit_vec_object_id(ctx: &mut StorageContext, value: vector<ObjectID>) {
      events::emit_event<VecObjectIDEvent>(ctx, VecObjectIDEvent { value });
   }

   public entry fun emit_mix(ctx: &mut StorageContext, value1: u8, value2: vector<ObjectID>) {
      events::emit_event<U8Event>(ctx, U8Event { value: value1 });
      events::emit_event<VecObjectIDEvent>(ctx, VecObjectIDEvent { value: value2 });     
   }
}
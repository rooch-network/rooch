module rooch_examples::C1 {
   struct Data has key, store {
      value:u64,
   }

   #[private_generics(T2)]
   public fun f1<T1, T2>() {

   }
}

module rooch_examples::counter {
   use moveos_std::account_storage;
   use moveos_std::storage_context::{StorageContext};
   use rooch_examples::C1::f1;

   struct Counter has key, store {
      value:u64,
   }

   public fun test_init(_account: &signer) {
      f1<Counter, Counter>();
   }

   fun init(ctx: &mut StorageContext, account: &signer){
      account_storage::global_move_to(ctx, account, Counter{value:0});
   }

   public fun increase_(ctx: &mut StorageContext) {
      let counter = account_storage::global_borrow_mut<Counter>(ctx, @rooch_examples);
      counter.value = counter.value + 1;
   }

   public entry fun increase(ctx: &mut StorageContext) {
      Self::increase_(ctx)
   }

   public fun value(ctx: &StorageContext): u64 {
      let counter = account_storage::global_borrow<Counter>(ctx, @rooch_examples);
      counter.value
   }
}
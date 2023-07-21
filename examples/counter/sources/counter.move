module rooch_examples::counter {
   use moveos_std::account_storage;
   use moveos_std::storage_context::{StorageContext};

   struct Counter has key, store {
      value:u64,
   }

   #[test_only]
   public fun init_for_test(ctx: &mut StorageContext, account: &signer) {
      account_storage::global_move_to(ctx, account, Counter { value: 0 });
   }

   fun init(ctx: &mut StorageContext, account: &signer) {
      account_storage::global_move_to(ctx, account, Counter { value: 0 });
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
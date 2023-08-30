module rooch_framework::account{
   use std::error;
   use std::hash;
   use std::vector;
   use std::signer;
   use rooch_framework::gas_coin::{GasCoin};

   use moveos_std::bcs;
   use moveos_std::storage_context::{Self, StorageContext};
   use moveos_std::account_storage;
   use rooch_framework::account_authentication;
   use rooch_framework::coin::{Self};

   friend rooch_framework::transaction_validator;

   /// Resource representing an account.
   struct Account has key, store {
      sequence_number: u64,
   }

   // ResourceAccount can only be stored under address, not in other structs.
   struct ResourceAccount has key {}
   // SignerCapability can only be stored in other structs, not under address.
   // So that the capability is always controlled by contracts, not by some EOA.
   struct SignerCapability has store { addr: address }

   const MAX_U64: u128 = 18446744073709551615;
   const ZERO_AUTH_KEY: vector<u8> = x"0000000000000000000000000000000000000000000000000000000000000000";
   // cannot be dummy key, or empty key
   const CONTRACT_ACCOUNT_AUTH_KEY_PLACEHOLDER:vector<u8> = x"0000000000000000000000000000000000000000000000000000000000000001";

   /// Scheme identifier used when hashing an account's address together with a seed to derive the address (not the
   /// authentication key) of a resource account. This is an abuse of the notion of a scheme identifier which, for now,
   /// serves to domain separate hashes used to derive resource account addresses from hashes used to derive
   /// authentication keys. Without such separation, an adversary could create (and get a signer for) a resource account
   /// whose address matches an existing address of a MultiEd25519 wallet.
   const SCHEME_DERIVE_RESOURCE_ACCOUNT: u8 = 255;
   
   /// Account already exists
   const ErrorAccountAlreadyExists: u64 = 1;
   /// Account does not exist
   const ErrorAccountNotExist: u64 = 2;
   /// Sequence number exceeds the maximum value for a u64
   const ErrorSequenceNumberTooBig: u64 = 3; 
   /// Cannot create account because address is reserved
   const ErrorAddressReseved: u64 = 5;
   /// An attempt to create a resource account on an account that has a committed transaction
   const ErrorResourceAccountAlreadyUsed: u64 = 6;
   /// Resource Account can't derive resource account
   const ErrorAccountIsAlreadyResourceAccount: u64 = 7;
   /// Address to create is not a valid reserved address for Rooch framework
   const ErrorNoValidFrameworkReservedAddress: u64 = 11;


   //TODO should we provide create account from arbitrary address?
   /// A entry function to create an account under `new_address`
   public entry fun create_account_entry(ctx: &mut StorageContext, new_address: address){
      // If account already exists, do nothing
      // Because if the new address is the same as the sender, the account must already created in the `transaction_validator::pre_execute` function
      if(!exists_at(ctx, new_address)){
         create_account(ctx, new_address);
      };
   }

   /// Publishes a new `Account` resource under `new_address`. A signer representing `new_address`
   /// is returned. This way, the caller of this function can publish additional resources under
   /// `new_address`.
   public(friend) fun create_account(ctx: &mut StorageContext, new_address: address): signer {
      assert!(
         new_address != @vm_reserved && new_address != @rooch_framework,
         error::invalid_argument(ErrorAddressReseved)
      );

      // there cannot be an Account resource under new_addr already.
      assert!(
         !account_storage::global_exists<Account>(ctx, new_address),
         error::already_exists(ErrorAccountAlreadyExists)
      ); 

      let new_account = create_account_unchecked(ctx, new_address);
      // initialize account coin store
      coin::init_account_coin_store(ctx, &new_account);
      new_account
   }

   fun create_account_unchecked(ctx: &mut StorageContext, new_address: address): signer {
      let new_account = create_signer(new_address);

      account_storage::ensure_account_storage(ctx, new_address);
      account_storage::global_move_to<Account>(ctx,
         &new_account,
         Account {
            sequence_number: 0,
      });
      account_authentication::init_authentication_keys(ctx, &new_account);
      new_account
   }

   /// create the account for system reserved addresses
   public(friend) fun create_framework_reserved_account(ctx: &mut StorageContext, addr: address): (signer, SignerCapability) {
      assert!(
         addr == @0x1 ||
             addr == @0x2 ||
             addr == @0x3 ||
             addr == @0x4 ||
             addr == @0x5 ||
             addr == @0x6 ||
             addr == @0x7 ||
             addr == @0x8 ||
             addr == @0x9 ||
             addr == @0xa,
         error::permission_denied(ErrorNoValidFrameworkReservedAddress),
      );
      let signer = create_account_unchecked(ctx, addr);
      let signer_cap = SignerCapability { addr };
      (signer, signer_cap)
   }


   /// Return the current sequence number at `addr`
   public fun sequence_number(ctx: &StorageContext, addr: address): u64 {
      // if account does not exist, return 0 as sequence number
      // TODO: refactor this after we decide how to handle account create.
      if (!account_storage::global_exists<Account>(ctx, addr)) {
         return 0
      };
      let account = account_storage::global_borrow<Account>(ctx, addr);
      sequence_number_for_account(account)
   }

   public fun sequence_number_for_sender(ctx: &StorageContext): u64 {
      let sender = storage_context::sender(ctx);
      sequence_number(ctx, sender)
   }

   public(friend) fun increment_sequence_number(ctx: &mut StorageContext) {
      let sender = storage_context::sender(ctx);

      let sequence_number = &mut account_storage::global_borrow_mut<Account>(ctx, sender).sequence_number;

      assert!(
         (*sequence_number as u128) < MAX_U64,
         error::out_of_range(ErrorSequenceNumberTooBig)
      );

      *sequence_number = *sequence_number + 1;
   }

   /// Helper to return the sequence number field for given `account`
   fun sequence_number_for_account(account: &Account): u64 {
      account.sequence_number
   }

   public fun signer_address(cap: &SignerCapability): address {
      cap.addr
   }

   public fun is_resource_account(ctx: &StorageContext, addr: address): bool {
      // for resource account , account storage maybe not exist when create,
      // so need check account storage eixst befor call global exist function
      if(account_storage::exist_account_storage(ctx, addr)){
         account_storage::global_exists<ResourceAccount>(ctx, addr)
      } else {
         false
      }
   }


   #[view]
   public fun exists_at(ctx: &StorageContext, addr: address): bool {
      if(account_storage::exist_account_storage(ctx, addr)){
         account_storage::global_exists<Account>(ctx, addr)
      } else {
         false
      }
   }


   native fun create_signer(addr: address): signer;


   /// A resource account is used to manage resources independent of an account managed by a user.
   /// In Rooch a resource account is created based upon the sha3 256 of the source's address and additional seed data.
   /// A resource account can only be created once
   // public fun create_resource_account(source: &signer): (signer, SignerCapability) {
   public fun create_resource_account(ctx: &mut StorageContext, source: &signer): (signer, SignerCapability) {
      let source_addr = signer::address_of(source);
      let seed = generate_seed_bytes(ctx, &source_addr);
      let resource_addr = create_resource_address(&source_addr, seed);
      assert!(!is_resource_account(ctx, resource_addr), error::invalid_state(ErrorAccountIsAlreadyResourceAccount));
      let resource_signer = if (exists_at(ctx, resource_addr)) {
         let account = account_storage::global_borrow<Account>(ctx, resource_addr);
         assert!(account.sequence_number == 0, error::invalid_state(ErrorResourceAccountAlreadyUsed));
         create_signer(resource_addr)
      } else {
         create_account_unchecked(ctx, resource_addr)
      };

      account_storage::global_move_to<ResourceAccount>(ctx,
         &resource_signer,
         ResourceAccount {}
      );

      let signer_cap = SignerCapability { addr: resource_addr };
      (resource_signer, signer_cap)
   }

   /// This is a helper function to generate seed for resource address
   fun generate_seed_bytes(ctx: &StorageContext, addr: &address): vector<u8> {
      let sequence_number = Self::sequence_number(ctx, *addr);
      // use rooch gas coin balance as part of seed, just for new address more random.
      let balance = coin::balance<GasCoin>(ctx, *addr);

      let seed_bytes = bcs::to_bytes(addr);
      vector::append(&mut seed_bytes, bcs::to_bytes(&sequence_number));
      vector::append(&mut seed_bytes, bcs::to_bytes(&balance));

      hash::sha3_256(seed_bytes)
   }

   /// This is a helper function to compute resource addresses. Computation of the address
   /// involves the use of a cryptographic hash operation and should be use thoughtfully.
   public fun create_resource_address(source: &address, seed: vector<u8>): address {
      let bytes = bcs::to_bytes(source);
      vector::append(&mut bytes, seed);
      vector::push_back(&mut bytes, SCHEME_DERIVE_RESOURCE_ACCOUNT);
      bcs::to_address(hash::sha3_256(bytes))
   }

   public fun create_signer_with_capability(capability: &SignerCapability): signer {
      let addr = &capability.addr;
      create_signer(*addr)
   }

   public fun get_signer_capability_address(capability: &SignerCapability): address {
      capability.addr
   }

   #[test_only]
   public fun init_account_for_test(ctx: &mut StorageContext, account: &signer) {
      // initialize account coin store
      coin::init_account_coin_store(ctx, account);
   }

   #[test_only]
   /// Create signer for testing, independently of an Rooch-style `Account`.
   public fun create_signer_for_test(addr: address): signer { create_signer(addr) }

   #[test_only]
   public fun create_account_for_test(ctx: &mut StorageContext, new_address: address): signer {
      create_account_unchecked(ctx, new_address)
   }

   #[test]
   /// Assert correct signer creation.
   fun test_create_signer_for_test() {
      assert!(signer::address_of(&create_signer_for_test(@rooch_framework)) == @0x3, 100);
      assert!(signer::address_of(&create_signer_for_test(@0x123)) == @0x123, 101);
   }

   #[test]
   /// Assert correct account creation.
   fun test_create_account_for_test() {
      let alice_addr = @123456;
      let ctx = storage_context::new_test_context(alice_addr);
      let alice = create_account_for_test(&mut ctx, alice_addr);
      let alice_addr_actual = signer::address_of(&alice);
      let sequence_number = sequence_number(&mut ctx, alice_addr);
      //std::debug::print(&get_authentication_key(&mut ctx, alice_addr));
      std::debug::print(&sequence_number);
      assert!(alice_addr_actual == alice_addr, 103);
      assert!(sequence_number >= 0, 104);
      storage_context::drop_test_context(ctx);
   }

   #[test_only]
   struct CapResponsbility has key {
      cap: SignerCapability
   }

   #[test]
   fun test_create_resource_account()  {
      let alice_addr = @123456;
      let ctx = storage_context::new_test_context(alice_addr);
      let alice = create_account_for_test(&mut ctx, alice_addr);
      let (resource_account, resource_account_cap) = create_resource_account(&mut ctx, &alice);
      let signer_cap_addr = get_signer_capability_address(&resource_account_cap);
      account_storage::global_move_to<CapResponsbility>(&mut ctx,
         &resource_account,
         CapResponsbility {
            cap: resource_account_cap
         }
      );

      let resource_addr = signer::address_of(&resource_account);
      std::debug::print(&100100);
      std::debug::print(&resource_addr);
      assert!(resource_addr != signer::address_of(&alice), 106);
      assert!(resource_addr == signer_cap_addr, 107);
      storage_context::drop_test_context(ctx);
   }

   #[test(sender=@0x42)]
   fun test_create_account_entry(sender: address){
      let ctx = storage_context::new_test_context(sender);
      create_account_entry(&mut ctx, sender);
      storage_context::drop_test_context(ctx);
   }

   #[test(sender=@0x0)]
   #[expected_failure(abort_code = 0x10005, location = Self)]
   fun test_failure_entry_account_creation_reserved(sender: address){
      let ctx = storage_context::new_test_context(sender);
      create_account_entry(&mut ctx, sender);
      storage_context::drop_test_context(ctx);
   }

   #[test(sender=@0x42, resource_account=@0xbb6e573f7feb9d8474ac20813fc086cc3100b8b7d49c246b0f4aee8ea19eaef4)]
   // #[expected_failure(abort_code = 0x30006, location = Self)]
   fun test_failure_create_resource_account_wrong_sequence_number(sender: address, resource_account: address){
      {
         let ctx = storage_context::new_test_context(resource_account);
         create_account_for_test(&mut ctx, resource_account);
         increment_sequence_number(&mut ctx);
         storage_context::drop_test_context(ctx);
      };
      let ctx = storage_context::new_test_context(sender);
      let sender_signer = create_account_for_test(&mut ctx, sender);
      let (signer, cap) = create_resource_account(&mut ctx, &sender_signer);
      account_storage::global_move_to<CapResponsbility>(&mut ctx,
         &signer,
         CapResponsbility {
            cap
         }
      );
      storage_context::drop_test_context(ctx);
   }
}
// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module moveos_std::account {
   use std::hash;
   use std::vector;
   use std::signer;
   use moveos_std::core_addresses;
   use moveos_std::bcs;
   use moveos_std::context::{Self, Context};
   use moveos_std::context::exists_object;
   use moveos_std::object_id::ObjectID;
   use moveos_std::object_id;
   use moveos_std::type_table::{key};
   use moveos_std::object::{Self, Object, borrow_object};
   #[test_only]
   use moveos_std::object::{take_object, borrow_mut_object};

   //FIXME remove the store ability from Account
   /// Account is part of the StorageAbstraction
   /// It is also used to store the account's resources
   struct Account has key,store {
      sequence_number: u64,
   }

   /// ResourceAccount can only be stored under address, not in other structs.
   struct ResourceAccount has key {}
   /// SignerCapability can only be stored in other structs, not under address.
   /// So that the capability is always controlled by contracts, not by some EOA.
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
   /// Account does not exists
   const ErrorAccountNotExists: u64 = 2;
   /// Sequence number exceeds the maximum value for a u64
   const ErrorSequenceNumberTooBig: u64 = 3;
   /// Cannot create account because address is reserved
   const ErrorAddressReserved: u64 = 4;
   /// An attempt to create a resource account on an account that has a committed transaction
   const ErrorResourceAccountAlreadyUsed: u64 = 5;
   /// Resource Account can't derive resource account
   const ErrorAccountIsAlreadyResourceAccount: u64 = 6;
   /// Address to create is not a valid reserved address
   const ErrorNotValidSystemReservedAddress: u64 = 7;


   /// The resource with the given type already exists
   const ErrorResourceAlreadyExists: u64 = 8;
   /// The resource with the given type not exists
   const ErrorResourceNotExists: u64 = 9;


   /// Publishes a new `Account` resource under `new_address`. A signer representing `new_address`
   /// is returned. This way, the caller of this function can publish additional resources under
   /// `new_address`.
   public(friend) fun create_account(ctx: &mut Context, new_address: address): signer {
      create_account_internal(ctx, new_address)
   }

   /// Publishes a new `Account` resource under `new_address` via system. A signer representing `new_address`
   /// is returned. This way, the caller of this function can publish additional resources under
   /// `new_address`.
   public fun create_account_for_system(ctx: &mut Context, system: &signer, new_address: address): signer {
      core_addresses::assert_system_reserved(system);
      create_account_internal(ctx, new_address)
   }

   fun create_account_internal(ctx: &mut Context, new_address: address): signer {
      assert!(
         !core_addresses::is_vm_address(new_address),
         ErrorAddressReserved
      );

      // Make sure the Account is not already created.
      assert!(
         !exist_account_object(ctx, new_address),
         ErrorAccountAlreadyExists
      );

      let new_account = create_account_unchecked(ctx, new_address);
      new_account
   }

   fun create_account_unchecked(_ctx: &mut Context, new_address: address): signer {
      let new_account = create_signer(new_address);

      create_account_object(new_address);
      new_account
   }

   /// create the account for system reserved addresses
   public fun create_system_reserved_account(ctx: &mut Context, system: &signer, addr: address): (signer, SignerCapability) {
      core_addresses::assert_system_reserved(system);
      assert!(
         core_addresses::is_system_reserved_address(addr),
         ErrorNotValidSystemReservedAddress,
      );
      let signer = create_account_unchecked(ctx, addr);
      let signer_cap = SignerCapability { addr };
      (signer, signer_cap)
   }


   /// Return the current sequence number at `addr`
   public fun sequence_number(ctx: &Context, addr: address): u64 {
      // if account does not exist, return 0 as sequence number
      // TODO: refactor this after we decide how to handle account create.
      if (!exist_account_object(ctx, addr)) {
         return 0
      };
      let object_id = account_object_id(addr);
      let obj = object::borrow_object<Account>(object_id);
      let account = object::borrow<Account>(obj);
      sequence_number_for_account(account)
   }

   public fun sequence_number_for_sender(ctx: &Context): u64 {
      let sender = context::sender(ctx);
      sequence_number(ctx, sender)
   }

   public(friend) fun increment_sequence_number(ctx: &mut Context) {
      increment_sequence_number_internal(ctx)
   }

   public fun increment_sequence_number_for_system(ctx: &mut Context, system: &signer) {
      core_addresses::assert_system_reserved(system);
      increment_sequence_number_internal(ctx)
   }

   fun increment_sequence_number_internal(ctx: &mut Context) {
      let sender = context::sender(ctx);
      let tx_sequence_number = context::sequence_number(ctx);

      let object_id = account_object_id(sender);
      let object_entity = object::borrow_mut_from_global<Account>(object_id);
      let obj_mut = object::as_mut_ref(object_entity);
      let account = object::borrow_mut<Account>(obj_mut);

      assert!(
         (account.sequence_number as u128) < MAX_U64,
         ErrorSequenceNumberTooBig
      );

      account.sequence_number = tx_sequence_number + 1;
   }

   /// Helper to return the sequence number field for given `account`
   fun sequence_number_for_account(account: &Account): u64 {
      account.sequence_number
   }

   public fun signer_address(cap: &SignerCapability): address {
      cap.addr
   }

   public fun is_resource_account(ctx: &Context, addr: address): bool {
      exists_resource<ResourceAccount>(ctx, addr)
   }

   public fun exists_at(ctx: &Context, addr: address): bool {
      exist_account_object(ctx, addr)
   }

   public fun create_signer_for_system(system: &signer, addr: address): signer {
      core_addresses::assert_system_reserved(system);
      create_signer(addr)
   }

   native public(friend) fun create_signer(addr: address): signer;

   /// A resource account is used to manage resources independent of an account managed by a user.
   /// In Rooch a resource account is created based upon the sha3 256 of the source's address and additional seed data.
   /// A resource account can only be created once
   public fun create_resource_account(ctx: &mut Context, source: &signer): (signer, SignerCapability) {
      let source_addr = signer::address_of(source);
      let seed = generate_seed_bytes(ctx, &source_addr);
      let resource_addr = create_resource_address(&source_addr, seed);
      assert!(!is_resource_account(ctx, resource_addr), ErrorAccountIsAlreadyResourceAccount);
      let resource_signer = if (exists_at(ctx, resource_addr)) {
         let object_id = account_object_id(resource_addr);
         let obj = object::borrow_object<Account>(object_id);
         let account = object::borrow<Account>(obj);
         assert!(account.sequence_number == 0, ErrorResourceAccountAlreadyUsed);
         create_signer(resource_addr)
      } else {
         create_account_unchecked(ctx, resource_addr)
      };

      move_resource_to<ResourceAccount>(ctx,
         &resource_signer,
         ResourceAccount {}
      );

      let signer_cap = SignerCapability { addr: resource_addr };
      (resource_signer, signer_cap)
   }


   /// This is a helper function to generate seed for resource address
   fun generate_seed_bytes(ctx: &Context, addr: &address): vector<u8> {
      let sequence_number = Self::sequence_number(ctx, *addr);

      let seed_bytes = bcs::to_bytes(addr);
      vector::append(&mut seed_bytes, bcs::to_bytes(&sequence_number));

      hash::sha3_256(seed_bytes)
   }

   /// This is a helper function to compute resource addresses. Computation of the address
   /// involves the use of a cryptographic hash operation and should be use thoughtfully.
   fun create_resource_address(source: &address, seed: vector<u8>): address {
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



   public fun account_object_id(account: address): ObjectID {
      object_id::address_to_object_id(account)
   }

   /// Create a new account object space
   public(friend) fun create_account_object(account: address) {
      let object_id = object_id::address_to_object_id(account);
      let obj = object::new_with_id(object_id, Account {sequence_number: 0});
      object::transfer_extend(obj, account)
   }

   // === Account Object Functions

   public fun account_borrow_resource<T: key>(self: &Object<Account>): &T {
      object::borrow_field(self, key<T>())
   }

   public fun account_borrow_mut_resource<T: key>(self: &mut Object<Account>): &mut T {
      object::borrow_mut_field(self, key<T>())
   }

   public fun account_move_resource_to<T: key>(self: &mut Object<Account>, resource: T){
      assert!(!object::contains_field(self, key<T>()), ErrorResourceAlreadyExists);
      object::add_field(self, key<T>(), resource)
   }

   public fun account_move_resource_from<T: key>(self: &mut Object<Account>): T {
      assert!(object::contains_field(self, key<T>()), ErrorResourceNotExists);
      object::remove_field(self, key<T>())
   }

   public fun account_exists_resource<T: key>(self: &Object<Account>) : bool {
      object::contains_field(self, key<T>())
   }

   public(friend) fun transfer(obj: Object<Account>, account: address) {
      object::transfer_extend(obj, account);
   }

   // === Account Storage functions ===

   // #[private_generics(T)]
   /// Borrow a resource from the account's storage
   /// This function equates to `borrow_global<T>(address)` instruction in Move
   public fun borrow_resource<T: key>(_self: &Context, account: address): &T {
      let obj = borrow_object<Account>(account_object_id(account));
      account_borrow_resource<T>(obj)
   }

   #[private_generics(T)]
   /// Borrow a mut resource from the account's storage
   /// This function equates to `borrow_global_mut<T>(address)` instruction in Move
   public fun borrow_mut_resource<T: key>(_self: &mut Context, account: address): &mut T {
      let object_id = account_object_id(account);
      let object_entity = object::borrow_mut_from_global<Account>(object_id);
      let obj_mut = object::as_mut_ref(object_entity);
      account_borrow_mut_resource<T>(obj_mut)
   }

   #[private_generics(T)]
   /// Move a resource to the account's resource object
   /// This function equates to `move_to<T>(&signer, resource)` instruction in Move
   public fun move_resource_to<T: key>(self: &mut Context, account: &signer, resource: T){
      let account_address = signer::address_of(account);
      //Auto create the resource object when move resource to the account
      ensure_account_object(self, account_address);
      let object_id = account_object_id(account_address);
      let object_entity = object::borrow_mut_from_global<Account>(object_id);
      let obj_mut = object::as_mut_ref(object_entity);
      account_move_resource_to(obj_mut, resource);
   }

   #[private_generics(T)]
   /// Move a resource from the account's storage
   /// This function equates to `move_from<T>(address)` instruction in Move
   public fun move_resource_from<T: key>(_self: &mut Context, account: address): T {
      let object_id = account_object_id(account);
      let object_entity = object::borrow_mut_from_global<Account>(object_id);
      let obj_mut = object::as_mut_ref(object_entity);
      account_move_resource_from<T>(obj_mut)
   }

   #[private_generics(T)]
   /// Check if the account has a resource of the given type
   /// This function equates to `exists<T>(address)` instruction in Move
   public fun exists_resource<T: key>(self: &Context, account: address) : bool {
      if (exist_account_object(self, account)) {
         let obj = borrow_object<Account>(account_object_id(account));
         account_exists_resource<T>(obj)
      }else{
         false
      }
   }

   // == Internal functions ==

   fun ensure_account_object(self: &mut Context, account: address) {
      if (!exist_account_object(self, account)) {
         create_account_object(account);
      }
   }

   fun exist_account_object(self: &Context, account: address): bool {
      exists_object<Account>(self, account_object_id(account))
   }


   #[test_only]
   /// Create signer for testing, independently of an Rooch-style `Account`.
   public fun create_signer_for_test(addr: address): signer { create_signer(addr) }

   #[test_only]
   public fun create_only_account_for_test(ctx: &mut Context, new_address: address): signer {
      create_account_unchecked(ctx, new_address)
   }

   #[test]
   /// Assert correct signer creation.
   fun test_create_signer_for_test() {
      assert!(signer::address_of(&create_signer_for_test(@moveos_std)) == @0x2, 100);
      assert!(signer::address_of(&create_signer_for_test(@0x123)) == @0x123, 101);
   }

   #[test]
   /// Assert correct account creation.
   fun test_create_only_account_for_test() {
      let alice_addr = @123456;
      let ctx = context::new_test_context(alice_addr);
      let alice = create_only_account_for_test(&mut ctx, alice_addr);
      let alice_addr_actual = signer::address_of(&alice);
      let sequence_number = sequence_number(&mut ctx, alice_addr);
      //std::debug::print(&get_authentication_key(&mut ctx, alice_addr));
      std::debug::print(&sequence_number);
      assert!(alice_addr_actual == alice_addr, 103);
      assert!(sequence_number >= 0, 104);
      context::drop_test_context(ctx);
   }

   #[test_only]
   struct CapResponsbility has key {
      cap: SignerCapability
   }

   #[test]
   fun test_create_resource_account()  {
      let alice_addr = @123456;
      let ctx = context::new_test_context(alice_addr);
      let alice = create_only_account_for_test(&mut ctx, alice_addr);
      let (resource_account, resource_account_cap) = create_resource_account(&mut ctx, &alice);
      let signer_cap_addr = get_signer_capability_address(&resource_account_cap);
      move_resource_to<CapResponsbility>(&mut ctx,
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
      context::drop_test_context(ctx);
   }

   //TODO figure out why this test should failed
   #[test(sender=@0x42, resource_account=@0xbb6e573f7feb9d8474ac20813fc086cc3100b8b7d49c246b0f4aee8ea19eaef4)]
   #[expected_failure(abort_code = ErrorResourceAccountAlreadyUsed, location = Self)]
   fun test_failure_create_resource_account_wrong_sequence_number(sender: address, resource_account: address){
      {
         let ctx = context::new_test_context(resource_account);
         create_only_account_for_test(&mut ctx, resource_account);
         increment_sequence_number(&mut ctx);
         context::drop_test_context(ctx);
      };
      let ctx = context::new_test_context(sender);
      let sender_signer = create_only_account_for_test(&mut ctx, sender);
      let (signer, cap) = create_resource_account(&mut ctx, &sender_signer);
      move_resource_to<CapResponsbility>(&mut ctx,
         &signer,
         CapResponsbility {
            cap
         }
      );
      context::drop_test_context(ctx);
   }

   #[test_only]
   fun drop_account_object(self: Object<Account>) {
      let obj = object::remove_unchecked(self);
      let Account {sequence_number:_} = obj;
   }

   #[test_only]
   struct Test has key{
      addr: address,
      version: u64
   }

   #[test(sender=@0x42)]
   fun test_account_object(sender: signer){
      let sender_addr = signer::address_of(&sender);
      create_account_object(sender_addr);
      let obj_mut = borrow_mut_object<Account>(&sender, account_object_id(sender_addr));
      account_move_resource_to(obj_mut, Test{
         addr: sender_addr,
         version: 1,
      });
      let obj = take_object<Account>(&sender, account_object_id(sender_addr));
      Self::drop_account_object(obj);
   }

   #[test(sender=@0x42)]
   fun test_move_to_account_object(sender: signer){
      let sender_addr = signer::address_of(&sender);
      create_account_object(sender_addr);
      let obj_mut = borrow_mut_object<Account>(&sender, account_object_id(sender_addr));
      account_move_resource_to(obj_mut, Test{
         addr: sender_addr,
         version: 1,
      });
      let obj = take_object<Account>(&sender, account_object_id(sender_addr));
      Self::drop_account_object(obj);
   }

   #[test(sender=@0x42)]
   fun test_move_from_account_object(sender: signer){
      let sender_addr = signer::address_of(&sender);
      create_account_object(sender_addr);
      let obj_mut = borrow_mut_object<Account>(&sender, account_object_id(sender_addr));
      account_move_resource_to(obj_mut, Test{
         addr: sender_addr,
         version: 1,
      });
      let Test {
         addr,
         version
      } = account_move_resource_from<Test>(obj_mut);
      assert!(addr == sender_addr, 0x10);
      assert!(version == 1, 0x11);
      let obj = take_object<Account>(&sender, account_object_id(sender_addr));
      Self::drop_account_object(obj);
   }

   #[test(sender=@0x42)]
   #[expected_failure(abort_code = ErrorResourceAlreadyExists, location = Self)]
   fun test_failure_repeatedly_move_to_account_object(sender: signer){
      let sender_addr = signer::address_of(&sender);
      create_account_object(sender_addr);
      let obj_mut = borrow_mut_object<Account>(&sender, account_object_id(sender_addr));
      account_move_resource_to(obj_mut, Test{
         addr: sender_addr,
         version: 1,
      });
      account_move_resource_to(obj_mut, Test{
         addr: sender_addr,
         version: 1,
      });
      let obj = take_object<Account>(&sender, account_object_id(sender_addr));
      Self::drop_account_object(obj);
   }

   #[test(sender=@0x42)]
   #[expected_failure(abort_code = ErrorResourceNotExists, location = Self)]
   fun test_failure_repeatedly_move_from_account_object(sender: signer){
      let sender_addr = signer::address_of(&sender);
      create_account_object(sender_addr);
      let obj_mut = borrow_mut_object<Account>(&sender, account_object_id(sender_addr));
      account_move_resource_to(obj_mut, Test{
         addr: sender_addr,
         version: 1,
      });
      let Test {
         addr: _,
         version: _
      } = account_move_resource_from<Test>(obj_mut);
      let Test {
         addr: _,
         version: _
      } = account_move_resource_from<Test>(obj_mut);
      let obj = take_object<Account>(&sender, account_object_id(sender_addr));
      Self::drop_account_object(obj);
   }

   #[test(sender=@0x42)]
   fun test_borrow_resource(sender: signer){
      let sender_addr = signer::address_of(&sender);
      create_account_object(sender_addr);
      let obj_mut = borrow_mut_object<Account>(&sender, account_object_id(sender_addr));
      account_move_resource_to(obj_mut, Test{
         addr: sender_addr,
         version: 1,
      });

      let ref_test = account_borrow_resource<Test>(obj_mut);
      assert!( ref_test.version == 1, 1);
      assert!( ref_test.addr == sender_addr, 2);
      let obj = take_object<Account>(&sender, account_object_id(sender_addr));
      Self::drop_account_object(obj);
   }

   #[test(sender=@0x42)]
   fun test_borrow_mut_resource(sender: signer){
      let sender_addr = signer::address_of(&sender);
      create_account_object(sender_addr);
      let obj_mut = borrow_mut_object<Account>(&sender, account_object_id(sender_addr));
      account_move_resource_to(obj_mut, Test{
         addr: sender_addr,
         version: 1,
      });
      {
         let ref_test = account_borrow_mut_resource<Test>(obj_mut);
         assert!( ref_test.version == 1, 1);
         assert!( ref_test.addr == sender_addr, 2);
         ref_test.version = 2;
      };
      {
         let ref_test = account_borrow_resource<Test>(obj_mut);
         assert!( ref_test.version == 2, 3);
      };
      let obj = take_object<Account>(&sender, account_object_id(sender_addr));
      Self::drop_account_object(obj);
   }

   #[test(sender=@0x42)]
   #[expected_failure(abort_code = 2, location = moveos_std::raw_table)]
   fun test_failure_borrow_resource_no_exists(sender: signer){
      let sender_addr = signer::address_of(&sender);
      create_account_object(sender_addr);
      let obj_ref = borrow_object<Account>(account_object_id(sender_addr));
      account_borrow_resource<Test>(obj_ref);
      let obj = take_object<Account>(&sender, account_object_id(sender_addr));
      Self::drop_account_object(obj);
   }

   #[test(sender=@0x42)]
   #[expected_failure(abort_code = 2, location = moveos_std::raw_table)]
   fun test_failure_borrow_mut_resource_no_exists(sender: signer){
      let sender_addr = signer::address_of(&sender);
      create_account_object(sender_addr);
      let obj_mut = borrow_mut_object<Account>(&sender, account_object_id(sender_addr));
      account_borrow_mut_resource<Test>(obj_mut);
      let obj = take_object<Account>(&sender, account_object_id(sender_addr));
      Self::drop_account_object(obj);
   }

   #[test(sender=@0x42)]
   fun test_ensure_move_from_and_exists(sender: signer){
      let sender_addr = signer::address_of(&sender);
      create_account_object(sender_addr);
      let obj_mut = borrow_mut_object<Account>(&sender, account_object_id(sender_addr));
      let test_exists = account_exists_resource<Test>(obj_mut);
      assert!(!test_exists, 1);
      account_move_resource_to(obj_mut, Test{
         addr: sender_addr,
         version: 1,
      });
      let test_exists = account_exists_resource<Test>(obj_mut);
      assert!(test_exists, 2);
      let test = account_move_resource_from<Test>(obj_mut);
      let test_exists = account_exists_resource<Test>(obj_mut);
      assert!(!test_exists, 3);
      let Test{
         addr: _,
         version: _
      } = test;
      let obj = take_object<Account>(&sender, account_object_id(sender_addr));
      Self::drop_account_object(obj);
   }

   #[test(sender=@0x42)]
   fun test_ensure_account_object(sender: signer){
      let sender_addr = signer::address_of(&sender);
      let ctx = context::new_test_context(sender_addr);
      ensure_account_object(&mut ctx , sender_addr);
      assert!(exist_account_object(&ctx , sender_addr), 1);
      context::drop_test_context(ctx);
   }

}

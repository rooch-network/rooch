// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_framework::account {
   use std::hash;
   use std::vector;
   use std::signer;
   use moveos_std::core_addresses;
   use moveos_std::bcs;
   // use moveos_std::context::{Self, Context};
   // use rooch_framework::account_authentication;
   // use rooch_framework::account_coin_store;

   use std::ascii::String;
   use moveos_std::account;
   use moveos_std::context;
   use moveos_std::context::{exists_object, Context};
   use moveos_std::object_id::ObjectID;
   use moveos_std::object_id;
   use moveos_std::type_table::{key};
   use moveos_std::object::{Self, Object};
   #[test_only]
   use moveos_std::object::{borrow_object, borrow_mut_object, take_object};
   #[test_only]
   use moveos_std::signer;

   friend moveos_std::context;

   // friend rooch_framework::transaction_validator;
   // friend rooch_framework::transfer;
   // friend rooch_framework::genesis;
   // friend rooch_framework::upgrade;

   /// Account is part of the StorageAbstraction
   /// It is also used to store the account's resources
   struct Account has key, store {
      sequence_number: u64,
   }

   // /// AccountAccount can only be stored under address, not in other structs.
   // struct AccountAccount has key {}
   /// SignerCapability can only be stored in other structs, not under address.
   /// So that the capability is always controlled by contracts, not by some EOA.
   struct SignerCapability has store { addr: address }

   const MAX_U64: u128 = 18446744073709551615;
   const ZERO_AUTH_KEY: vector<u8> = x"0000000000000000000000000000000000000000000000000000000000000000";
   // cannot be dummy key, or empty key
   const CONTRACT_ACCOUNT_AUTH_KEY_PLACEHOLDER:vector<u8> = x"0000000000000000000000000000000000000000000000000000000000000001";

   /// Account already exists
   const ErrorAccountAlreadyExists: u64 = 1;
   /// Account does not exist
   const ErrorAccountNotExist: u64 = 2;
   /// Sequence number exceeds the maximum value for a u64
   const ErrorSequenceNumberTooBig: u64 = 3;
   /// Cannot create account because address is reserved
   const ErrorAddressReserved: u64 = 4;
   /// Address to create is not a valid reserved address
   const ErrorNotValidSystemReservedAddress: u64 = 5;


   /// The resource with the given type already exists
   const ErrorResourceAlreadyExists: u64 = 6;
   /// The resource with the given type not exists
   const ErrorResourceNotExists: u64 = 7;


   //TODO should we provide create account from arbitrary address?
   // TODO Can create accounts arbitrary. Is this a security risk?
   /// A entry function to create an account under `new_address`
   public entry fun create_account_entry(ctx: &mut Context, new_address: address){
      // If account already exists, do nothing
      // Because if the new address is the same as the sender, the account must already created in the `transaction_validator::pre_execute` function
      if(!exists_at(ctx, new_address)){
         create_account_internal(ctx, new_address);
      };
   }

   /// Publishes a new `Account` resource under `new_address`. A signer representing `new_address`
   /// is returned. This way, the caller of this function can publish additional resources under
   /// `new_address`.
   public fun create_account(ctx: &mut Context, system: &signer, new_address: address): signer {
      core_addresses::assert_system_reserved(system);
      create_account_internal(ctx, new_address)
   }

   fun create_account_internal(ctx: &mut Context, new_address: address): signer {
      // assert!(
      //    new_address != @vm_reserved,
      //    ErrorAddressReseved
      // );
      assert!(
         !core_addresses::is_reserved_address(new_address),
         ErrorAddressReserved
      );

      // Make sure the Account is not already created.
      assert!(
         !context::exists_resource<Account>(ctx, new_address),
         ErrorAccountAlreadyExists
      );

      let new_account = create_account_unchecked(ctx, new_address);
      new_account
   }

   fun create_account_unchecked(ctx: &mut Context, new_address: address): signer {
      let new_account = create_signer(new_address);

      context::move_resource_to<Account>(ctx,
         &new_account,
         Account {
            sequence_number: 0,
      });

      create_account_object(new_address);
      // account_authentication::init_authentication_keys(ctx, &new_account);
      // account_coin_store::init_account_coin_stores(ctx, &new_account);
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


   // /// Return the current sequence number at `addr`
   // public fun sequence_number(ctx: &Context, addr: address): u64 {
   //    // if account does not exist, return 0 as sequence number
   //    // TODO: refactor this after we decide how to handle account create.
   //    if (!context::exists_resource<Account>(ctx, addr)) {
   //       return 0
   //    };
   //    let account = context::borrow_resource<Account>(ctx, addr);
   //    sequence_number_for_account(account)
   // }

   public fun sequence_number_for_sender(ctx: &Context): u64 {
      let sender = context::sender(ctx);
      account::sequence_number(sender)
   }

   public(friend) fun increment_sequence_number(ctx: &mut Context) {
      let sender = context::sender(ctx);
      let tx_sequence_number = context::sequence_number(ctx);

      let account = context::borrow_mut_resource<Account>(ctx, sender);

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

   public fun exists_at(_ctx: &Context, addr: address): bool {
      // context::exists_resource<Account>(ctx, addr)
      //
      // if (exist_account_object(self, account)) {
      //    let obj = borrow_object<Account>(account::account_object_id(account));
      //    account::exists_resource<T>(obj)
      // }else{
      //    false
      // }

      // exists_object<Account>(self, account::account_object_id(account))

      // let object_id = account_object_id(addr);
      // object::contains_global(object_id)

      // context::exists_resource<Account>(ctx, addr)
      account::exists_at(addr)
   }


   // native public(friend) fun create_signer(addr: address): signer;


   // /// This is a helper function to generate seed for resource address
   // fun generate_seed_bytes(ctx: &Context, addr: &address): vector<u8> {
   //    let sequence_number = Self::sequence_number(ctx, *addr);
   //
   //    let seed_bytes = bcs::to_bytes(addr);
   //    vector::append(&mut seed_bytes, bcs::to_bytes(&sequence_number));
   //
   //    hash::sha3_256(seed_bytes)
   // }


   // public fun create_signer_with_capability(capability: &SignerCapability): signer {
   //    let addr = &capability.addr;
   //    create_signer(*addr)
   // }

   // public fun get_signer_capability_address(capability: &SignerCapability): address {
   //    capability.addr
   // }



   // public fun account_object_id(account: address): ObjectID {
   //    object_id::address_to_object_id(account)
   // }
   //
   // /// Create a new account object space
   // public(friend) fun create_account_object(account: address) {
   //    let object_id = object_id::address_to_object_id(account);
   //    let obj = object::new_with_id(object_id, Account {sequence_number: 0});
   //    object::transfer(obj, account)
   // }
   //
   // // === Account Object Functions
   //
   // public fun borrow_resource<T: key>(self: &Object<Account>): &T {
   //    object::borrow_field<String, T>(object::id(self), key<T>())
   // }
   //
   // public fun borrow_mut_resource<T: key>(self: &mut Object<Account>): &mut T {
   //    object::borrow_mut_field<String, T>(object::id(self), key<T>())
   // }
   //
   // public fun move_resource_to<T: key>(self: &mut Object<Account>, resource: T){
   //    assert!(!object::contains_field<String>(object::id(self), key<T>()), ErrorResourceAlreadyExists);
   //    object::add_field<String, T>(object::id(self), key<T>(), resource)
   // }
   //
   // public fun move_resource_from<T: key>(self: &mut Object<Account>): T {
   //    assert!(object::contains_field<String>(object::id(self), key<T>()), ErrorResourceNotExists);
   //    object::remove_field<String, T>(object::id(self), key<T>())
   // }
   //
   // public fun exists_resource<T: key>(self: &Object<Account>) : bool {
   //    object::contains_field<String>(object::id(self), key<T>())
   // }
   //
   // public(friend) fun transfer(obj: Object<Account>, account: address) {
   //    object::transfer_extend(obj, account);
   // }



   #[test_only]
   /// Create signer for testing, independently of an Rooch-style `Account`.
   public fun create_signer_for_test(addr: address): signer { create_signer(addr) }

   #[test_only]
   public fun create_account_for_test(ctx: &mut Context, new_address: address): signer {
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
   fun test_create_account_for_test() {
      let alice_addr = @123456;
      let ctx = context::new_test_context(alice_addr);
      let alice = create_account_for_test(&mut ctx, alice_addr);
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

   #[test(sender=@0x42)]
   fun test_create_account_entry(sender: address){
      let ctx = context::new_test_context(sender);
      create_account_entry(&mut ctx, sender);
      context::drop_test_context(ctx);
   }

   #[test(sender=@0x0)]
   #[expected_failure(abort_code = ErrorAddressReseved, location = Self)]
   fun test_failure_entry_account_creation_reserved(sender: address){
      let ctx = context::new_test_context(sender);
      create_account_entry(&mut ctx, sender);
      context::drop_test_context(ctx);
   }



   #[test_only]
   fun drop_account_object(self: Object<Account>) {
      object::drop_unchecked_table(object::id(&self));
      let obj = object::remove(self);
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
      move_resource_to(obj_mut, Test{
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
      move_resource_to(obj_mut, Test{
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
      move_resource_to(obj_mut, Test{
         addr: sender_addr,
         version: 1,
      });
      let Test {
         addr,
         version
      } = move_resource_from<Test>(obj_mut);
      assert!(addr == sender_addr, 0x10);
      assert!(version == 1, 0x11);
      let obj = take_object<Account>(&sender, account_object_id(sender_addr));
      Self::drop_account_object(obj);
   }

   #[test(sender=@0x42)]
   #[expected_failure(abort_code = ErrorAccountAlreadyExists, location = Self)]
   fun test_failure_repeatedly_move_to_account_object(sender: signer){
      let sender_addr = signer::address_of(&sender);
      create_account_object(sender_addr);
      let obj_mut = borrow_mut_object<Account>(&sender, account_object_id(sender_addr));
      move_resource_to(obj_mut, Test{
         addr: sender_addr,
         version: 1,
      });
      move_resource_to(obj_mut, Test{
         addr: sender_addr,
         version: 1,
      });
      let obj = take_object<Account>(&sender, account_object_id(sender_addr));
      Self::drop_account_object(obj);
   }

   #[test(sender=@0x42)]
   #[expected_failure(abort_code = ErrorAccountNotExists, location = Self)]
   fun test_failure_repeatedly_move_from_account_object(sender: signer){
      let sender_addr = signer::address_of(&sender);
      create_account_object(sender_addr);
      let obj_mut = borrow_mut_object<Account>(&sender, account_object_id(sender_addr));
      move_resource_to(obj_mut, Test{
         addr: sender_addr,
         version: 1,
      });
      let Test {
         addr: _,
         version: _
      } = move_resource_from<Test>(obj_mut);
      let Test {
         addr: _,
         version: _
      } = move_resource_from<Test>(obj_mut);
      let obj = take_object<Account>(&sender, account_object_id(sender_addr));
      Self::drop_account_object(obj);
   }

   #[test(sender=@0x42)]
   fun test_borrow_resource(sender: signer){
      let sender_addr = signer::address_of(&sender);
      create_account_object(sender_addr);
      let obj_mut = borrow_mut_object<Account>(&sender, account_object_id(sender_addr));
      move_resource_to(obj_mut, Test{
         addr: sender_addr,
         version: 1,
      });

      let ref_test = borrow_resource<Test>(obj_mut);
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
      move_resource_to(obj_mut, Test{
         addr: sender_addr,
         version: 1,
      });
      {
         let ref_test = borrow_mut_resource<Test>(obj_mut);
         assert!( ref_test.version == 1, 1);
         assert!( ref_test.addr == sender_addr, 2);
         ref_test.version = 2;
      };
      {
         let ref_test = borrow_resource<Test>(obj_mut);
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
      borrow_resource<Test>(obj_ref);
      let obj = take_object<Account>(&sender, account_object_id(sender_addr));
      Self::drop_account_object(obj);
   }

   #[test(sender=@0x42)]
   #[expected_failure(abort_code = 2, location = moveos_std::raw_table)]
   fun test_failure_borrow_mut_resource_no_exists(sender: signer){
      let sender_addr = signer::address_of(&sender);
      create_account_object(sender_addr);
      let obj_mut = borrow_mut_object<Account>(&sender, account_object_id(sender_addr));
      borrow_mut_resource<Test>(obj_mut);
      let obj = take_object<Account>(&sender, account_object_id(sender_addr));
      Self::drop_account_object(obj);
   }

   #[test(sender=@0x42)]
   fun test_ensure_move_from_and_exists(sender: signer){
      let sender_addr = signer::address_of(&sender);
      create_account_object(sender_addr);
      let obj_mut = borrow_mut_object<Account>(&sender, account_object_id(sender_addr));
      let test_exists = exists_resource<Test>(obj_mut);
      assert!(!test_exists, 1);
      move_resource_to(obj_mut, Test{
         addr: sender_addr,
         version: 1,
      });
      let test_exists = exists_resource<Test>(obj_mut);
      assert!(test_exists, 2);
      let test = move_resource_from<Test>(obj_mut);
      let test_exists = exists_resource<Test>(obj_mut);
      assert!(!test_exists, 3);
      let Test{
         addr: _,
         version: _
      } = test;
      let obj = take_object<Account>(&sender, account_object_id(sender_addr));
      Self::drop_account_object(obj);
   }

}

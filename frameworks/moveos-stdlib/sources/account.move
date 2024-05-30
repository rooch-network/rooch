// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module moveos_std::account {
   use std::signer;
   use moveos_std::core_addresses;
   use moveos_std::type_table::{key};
   use moveos_std::object::{Self, ObjectID, Object};

   /// Account is part of the StorageAbstraction
   /// It is also used to store the account's resources
   struct Account has key {
      sequence_number: u64,
   }

   /// SignerCapability can only be stored in other structs, not under address.
   /// So that the capability is always controlled by contracts, not by some EOA.
   struct SignerCapability has store { addr: address }

   const MAX_U64: u128 = 18446744073709551615;
   const ZERO_AUTH_KEY: vector<u8> = x"0000000000000000000000000000000000000000000000000000000000000000";
   // cannot be dummy key, or empty key
   const CONTRACT_ACCOUNT_AUTH_KEY_PLACEHOLDER:vector<u8> = x"0000000000000000000000000000000000000000000000000000000000000001";

   /// Account already exists
   const ErrorAccountAlreadyExists: u64 = 1;
   /// Sequence number exceeds the maximum value for a u64
   const ErrorSequenceNumberTooBig: u64 = 2;
   /// Cannot create account because address is reserved
   const ErrorAddressReserved: u64 = 3;
   /// Address to create is not a valid reserved address
   const ErrorNotValidSystemReservedAddress: u64 = 4;
   /// The resource with the given type already exists
   const ErrorResourceAlreadyExists: u64 = 5;
   /// The resource with the given type not exists
   const ErrorResourceNotExists: u64 = 6;


   /// Publishes a new `Account` resource under `new_address` via system. A signer representing `new_address`
   /// is returned. This way, the caller of this function can publish additional resources under
   /// `new_address`.
   public fun create_account_by_system(system: &signer, new_address: address): signer {
      core_addresses::assert_system_reserved(system);
      create_account_internal(new_address)
   }

   fun create_account_internal(new_address: address): signer {
      assert!(
         !core_addresses::is_vm_address(new_address),
         ErrorAddressReserved
      );

      // Make sure the Account is not already created.
      assert!(
         !exist_account_object(new_address),
         ErrorAccountAlreadyExists
      );

      let new_account = create_account_unchecked(new_address);
      new_account
   }

   fun create_account_unchecked(new_address: address): signer {
      let new_account = create_signer(new_address);

      create_account_object(new_address);
      new_account
   }

   /// create the account for system reserved addresses
   public fun create_system_reserved_account(system: &signer, addr: address): (signer, SignerCapability) {
      core_addresses::assert_system_reserved(system);
      assert!(
         core_addresses::is_system_reserved_address(addr),
         ErrorNotValidSystemReservedAddress,
      );
      let signer = create_account_unchecked(addr);
      let signer_cap = SignerCapability { addr };
      (signer, signer_cap)
   }

   /// Return the current sequence number at `addr`
   public fun sequence_number(addr: address): u64 {
      // if account does not exist, return 0 as sequence number
      // TODO: refactor this after we decide how to handle account create.
      if (!exist_account_object(addr)) {
         return 0
      };
      let object_id = account_object_id(addr);
      let obj = object::borrow_object<Account>(object_id);
      let account = object::borrow<Account>(obj);
      sequence_number_for_account(account)
   }

   public fun increment_sequence_number_for_system(system: &signer, sender: address) {
      core_addresses::assert_system_reserved(system);
      increment_sequence_number_internal(sender)
   }

   fun increment_sequence_number_internal(sender: address) {
      let tx_sequence_number = sequence_number(sender);
      let account_obj = borrow_mut_account_internal(sender);
      let account = object::borrow_mut(account_obj);

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

   public fun exists_at(addr: address): bool {
      exist_account_object(addr)
   }

   public fun create_signer_for_system(system: &signer, addr: address): signer {
      core_addresses::assert_system_reserved(system);
      create_signer(addr)
   }

   native public(friend) fun create_signer(addr: address): signer;


   public fun create_signer_with_capability(capability: &SignerCapability): signer {
      let addr = &capability.addr;
      create_signer(*addr)
   }

   public fun get_signer_capability_address(capability: &SignerCapability): address {
      capability.addr
   }

   public fun account_object_id(account: address): ObjectID {
      object::address_to_object_id(account)
   }

   /// Create a new account object space
   public(friend) fun create_account_object(account: address) {
      let object_id = object::address_to_object_id(account);
      let obj = object::new_with_object_id(object_id, Account {sequence_number: 0});
      object::transfer_extend(obj, account)
   }

   // === Account Object Functions

   public fun account_borrow_resource<T: key>(self: &Object<Account>): &T {
      object::borrow_field_internal(object::id(self), key<T>())
   }

   public fun account_borrow_mut_resource<T: key>(self: &mut Object<Account>): &mut T {
      object::borrow_mut_field_internal(object::id(self), key<T>())
   }

   public fun account_move_resource_to<T: key>(self: &mut Object<Account>, resource: T){
      assert!(!object::contains_field(self, key<T>()), ErrorResourceAlreadyExists);
      object::add_field_internal<Account, std::ascii::String, T>(object::id(self), key<T>(), resource)
   }

   public fun account_move_resource_from<T: key>(self: &mut Object<Account>): T {
      assert!(object::contains_field(self, key<T>()), ErrorResourceNotExists);
      object::remove_field_internal<Account, std::ascii::String, T>(object::id(self), key<T>())
   }

   public fun account_exists_resource<T: key>(self: &Object<Account>) : bool {
      object::contains_field_internal(object::id(self), key<T>())
   }

   public(friend) fun transfer(obj: Object<Account>, account: address) {
      object::transfer_extend(obj, account);
   }

   // === Account Storage functions ===

   public fun borrow_account(account: address): &Object<Account>{
      object::borrow_object<Account>(account_object_id(account))
   }

   fun borrow_mut_account_internal(account: address): &mut Object<Account>{
      object::borrow_mut_object_extend<Account>(account_object_id(account))
   }

   /// Borrow a resource from the account's storage
   /// This function equates to `borrow_global<T>(address)` instruction in Move
   /// But we remove the restriction of the caller must be the module of T
   public fun borrow_resource<T: key>(account: address): &T {
      let account_obj = borrow_account(account);
      account_borrow_resource<T>(account_obj)
   }

   #[private_generics(T)]
   /// Borrow a mut resource from the account's storage
   /// This function equates to `borrow_global_mut<T>(address)` instruction in Move
   public fun borrow_mut_resource<T: key>(account: address): &mut T {
      let account_obj = borrow_mut_account_internal(account); 
      account_borrow_mut_resource<T>(account_obj)
   }

   #[private_generics(T)]
   /// Move a resource to the account's resource object
   /// This function equates to `move_to<T>(&signer, resource)` instruction in Move
   public fun move_resource_to<T: key>(account: &signer, resource: T){
      let account_address = signer::address_of(account);
      //Auto create the resource object when move resource to the account
      //TODO should we auto create the account?
      ensure_account_object(account_address);
      let account_obj = borrow_mut_account_internal(account_address); 
      account_move_resource_to(account_obj, resource);
   }

   #[private_generics(T)]
   /// Move a resource from the account's storage
   /// This function equates to `move_from<T>(address)` instruction in Move
   public fun move_resource_from<T: key>(account: address): T {
      let account_obj = borrow_mut_account_internal(account); 
      account_move_resource_from<T>(account_obj)
   }

   /// Check if the account has a resource of the given type
   /// This function equates to `exists<T>(address)` instruction in Move
   /// But we remove the restriction of the caller must be the module of T
   public fun exists_resource<T: key>(account: address) : bool {
      if (exist_account_object(account)) {
         let account_obj = borrow_account(account);
         account_exists_resource<T>(account_obj)
      }else{
         false
      }
   }

   // == Internal functions ==

   fun ensure_account_object(account: address) {
      if (!exist_account_object(account)) {
         create_account_object(account);
      }
   }

   fun exist_account_object(account: address): bool {
      object::exists_object(account_object_id(account))
   }


   #[test_only]
   /// Create signer for testing, independently of an Rooch-style `Account`.
   public fun create_signer_for_testing(addr: address): signer { create_signer(addr) }

   #[test_only]
   public fun create_account_for_testing(new_address: address): signer {
      create_account_unchecked(new_address)
   }

   #[test]
   /// Assert correct signer creation.
   fun test_create_signer_for_testing() {
      assert!(signer::address_of(&create_signer_for_testing(@moveos_std)) == @0x2, 100);
      assert!(signer::address_of(&create_signer_for_testing(@0x123)) == @0x123, 101);
   }

   #[test]
   /// Assert correct account creation.
   fun test_create_account_for_testing() {

      let alice_addr = @123456;
      let alice = create_account_for_testing(alice_addr);
      let alice_addr_actual = signer::address_of(&alice);
      let sequence_number = sequence_number(alice_addr);
      assert!(alice_addr_actual == alice_addr, 103);
      assert!(sequence_number >= 0, 104);
   }

   #[test_only]
   struct CapResponsbility has key {
      cap: SignerCapability
   }

   #[test_only]
   fun drop_account_object(self: Object<Account>) {
      let obj = object::drop_unchecked(self);
      let Account {sequence_number:_} = obj;
   }

   #[test_only]
   struct Test has key {
      addr: address,
      version: u64
   }

   #[test(sender=@0x42)]
   fun test_account_object(sender: signer){
      let sender_addr = signer::address_of(&sender);
      create_account_object(sender_addr);
      let obj_mut = object::borrow_mut_object<Account>(&sender, account_object_id(sender_addr));
      account_move_resource_to(obj_mut, Test{
         addr: sender_addr,
         version: 1,
      });
   }

   #[test(sender=@0x42)]
   fun test_move_to_account_object(sender: signer){
      let sender_addr = signer::address_of(&sender);
      create_account_object(sender_addr);
      let obj_mut = object::borrow_mut_object<Account>(&sender, account_object_id(sender_addr));
      account_move_resource_to(obj_mut, Test{
         addr: sender_addr,
         version: 1,
      });
   }

   #[test(sender=@0x42)]
   fun test_move_from_account_object(sender: signer){
      let sender_addr = signer::address_of(&sender);
      create_account_object(sender_addr);
      let obj_mut = object::borrow_mut_object<Account>(&sender, account_object_id(sender_addr));
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
   }

   #[test(sender=@0x42)]
   #[expected_failure(abort_code = ErrorResourceAlreadyExists, location = Self)]
   fun test_failure_repeatedly_move_to_account_object(sender: signer){

      let sender_addr = signer::address_of(&sender);
      create_account_object(sender_addr);
      let obj_mut = object::borrow_mut_object<Account>(&sender, account_object_id(sender_addr));
      account_move_resource_to(obj_mut, Test{
         addr: sender_addr,
         version: 1,
      });
      account_move_resource_to(obj_mut, Test{
         addr: sender_addr,
         version: 1,
      });
   }

   #[test(sender=@0x42)]
   #[expected_failure(abort_code = ErrorResourceNotExists, location = Self)]
   fun test_failure_repeatedly_move_from_account_object(sender: signer){
      let sender_addr = signer::address_of(&sender);
      create_account_object(sender_addr);
      let obj_mut = object::borrow_mut_object<Account>(&sender, account_object_id(sender_addr));
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
   }

   #[test(sender=@0x42)]
   fun test_borrow_resource(sender: signer){
      let sender_addr = signer::address_of(&sender);
      create_account_object(sender_addr);
      let obj_mut = object::borrow_mut_object<Account>(&sender, account_object_id(sender_addr));
      account_move_resource_to(obj_mut, Test{
         addr: sender_addr,
         version: 1,
      });

      let ref_test = account_borrow_resource<Test>(obj_mut);
      assert!(ref_test.version == 1, 1);
      assert!(ref_test.addr == sender_addr, 2);
   }

   #[test(sender=@0x42)]
   fun test_borrow_mut_resource(sender: signer){
      let sender_addr = signer::address_of(&sender);
      create_account_object(sender_addr);
      let obj_mut = object::borrow_mut_object<Account>(&sender, account_object_id(sender_addr));
      account_move_resource_to(obj_mut, Test{
         addr: sender_addr,
         version: 1,
      });
      {
         let ref_test = account_borrow_mut_resource<Test>(obj_mut);
         assert!(ref_test.version == 1, 1);
         assert!(ref_test.addr == sender_addr, 2);
         ref_test.version = 2;
      };
      {
         let ref_test = account_borrow_resource<Test>(obj_mut);
         assert!(ref_test.version == 2, 3);
      };
   }

   #[test(sender=@0x42)]
   #[expected_failure(abort_code = 2, location = moveos_std::object)]
   fun test_failure_borrow_resource_no_exists(sender: signer){
      let sender_addr = signer::address_of(&sender);
      create_account_object(sender_addr);
      let obj_ref = object::borrow_object<Account>(account_object_id(sender_addr));
      account_borrow_resource<Test>(obj_ref);
   }

   #[test(sender=@0x42)]
   #[expected_failure(abort_code = 2, location = moveos_std::object)]
   fun test_failure_borrow_mut_resource_no_exists(sender: signer){
      let sender_addr = signer::address_of(&sender);
      create_account_object(sender_addr);
      let obj_mut = object::borrow_mut_object<Account>(&sender, account_object_id(sender_addr));
      account_borrow_mut_resource<Test>(obj_mut);
   }

   #[test(sender=@0x42)]
   fun test_ensure_move_from_and_exists(sender: signer){
      let sender_addr = signer::address_of(&sender);
      create_account_object(sender_addr);
      let obj_mut = object::borrow_mut_object<Account>(&sender, account_object_id(sender_addr));
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
   }

   #[test(sender=@0x42)]
   fun test_ensure_account_object(sender: signer){
      let sender_addr = signer::address_of(&sender);
      ensure_account_object(sender_addr);
      assert!(exist_account_object(sender_addr), 1);
   }

}

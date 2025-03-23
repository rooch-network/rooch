// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module moveos_std::account {
   use std::signer;
   use moveos_std::core_addresses;
   use moveos_std::type_table::key;
   use moveos_std::object::{Self, ObjectID, Object};
   use moveos_std::tx_context;

   /// Account is a struct that holds the sequence number for an address
   struct Account has key {
      addr: address,
      sequence_number: u64,
   }

   /// AccountCap is a capability for Account
   /// The contract that has AccountCap can access the Account object
   struct AccountCap has key, store{
      addr: address,
   }

   const MAX_U64: u128 = 18446744073709551615;

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
   /// The function is deprecated
   const ErrorDeprecateFunction: u64 = 7;

   /// Create a new account for the given address, only callable by the system account
   public fun create_account_by_system(system: &signer, new_address: address): signer {
      core_addresses::assert_system_reserved(system);
      create_account_internal(new_address)
   }

   /// This function is deprecated, please use `create_account_and_return_cap` instead
   public fun create_account(): Object<Account> {
      //We shoud not return Object<Account> directly,
      //Becase if other struct hold the Object<Account>, and the resource functions will not work
      abort ErrorDeprecateFunction
   }

   /// Create a new account and return the AccountCap
   public fun create_account_and_return_cap(): AccountCap {
      let new_address = tx_context::fresh_address();
      let account_obj = create_account_object(new_address);
      object::transfer_extend(account_obj, new_address);
      AccountCap{addr: new_address}
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
      create_account_object_to(new_address);
      create_signer(new_address)
   }

   /// Return the current sequence number at `addr`
   public fun sequence_number(addr: address): u64 {
      // if account does not exist, return 0 as sequence number
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

   public fun exists_at(addr: address): bool {
      exist_account_object(addr)
   }

   public fun create_signer_for_system(system: &signer, addr: address): signer {
      core_addresses::assert_system_reserved(system);
      create_signer(addr)
   }

   public fun create_signer_with_account(account: &mut Object<Account>): signer{
      create_signer(object::borrow(account).addr)
   }

   /// Create a signer with the given account capability
   public fun create_signer_with_account_cap(cap: &mut AccountCap): signer{
      create_signer(cap.addr)
   }


   native fun create_signer(addr: address): signer;


   public fun account_object_id(account: address): ObjectID {
      object::address_to_object_id(account)
   }

   // === Account Object Functions

   public fun account_address(self: &Object<Account>): address {
      object::borrow(self).addr
   }

   public fun account_cap_address(self: &AccountCap): address {
      self.addr
   }

   public fun account_sequence_number(self: &Object<Account>): u64 {
      object::borrow(self).sequence_number
   }

   public fun account_borrow_resource<T: key>(self: &Object<Account>): &T {
      account_borrow_resource_internal(self)
   }

   fun account_borrow_resource_internal<T: key>(self: &Object<Account>): &T {
      let key = key<T>();
      assert!(object::contains_field(self, key), ErrorResourceNotExists);
      object::borrow_field_internal(object::id(self), key)
   }

   #[private_generics(T)]
   public fun account_borrow_mut_resource<T: key>(self: &mut Object<Account>): &mut T {
      account_borrow_mut_resource_interal(self)
   }

   fun account_borrow_mut_resource_interal<T: key>(self: &mut Object<Account>): &mut T {
      assert!(object::contains_field(self, key<T>()), ErrorResourceNotExists);
      object::borrow_mut_field_internal(object::id(self), key<T>())
   }


   #[private_generics(T)]
   public fun account_move_resource_to<T: key>(self: &mut Object<Account>, resource: T){
      account_move_resource_to_internal(self, resource)
   }

   fun account_move_resource_to_internal<T: key>(self: &mut Object<Account>, resource: T){
      let key = key<T>();
      assert!(!object::contains_field(self, key), ErrorResourceAlreadyExists);
      object::add_field_internal<std::string::String, T>(object::id(self), key, resource)
   }

   
   #[private_generics(T)]
   public fun account_move_resource_from<T: key>(self: &mut Object<Account>): T {
      account_move_resource_from_internal(self)
   }

   fun account_move_resource_from_internal<T: key>(self: &mut Object<Account>): T {
      assert!(object::contains_field(self, key<T>()), ErrorResourceNotExists);
      object::remove_field_internal<Account, std::string::String, T>(object::id(self), key<T>())
   }

   public fun account_exists_resource<T: key>(self: &Object<Account>) : bool {
      object::contains_field_internal(object::id(self), key<T>())
   }
   
   fun transfer(obj: Object<Account>, account: address) {
      object::transfer_extend(obj, account);
   }

   /// Deprecated: Direct destruction of account objects is not allowed.
   public fun destroy_account(_account_obj: Object<Account>){
      abort ErrorDeprecateFunction
   }

   /// Destroy the account capability
   public fun destroy_account_cap(account_cap: AccountCap){
      let AccountCap{addr:_} = account_cap;
   }

   public fun borrow_account(account: address): &Object<Account>{
      object::borrow_object<Account>(account_object_id(account))
   }

   public fun borrow_mut_account(account: &signer): &mut Object<Account>{
      borrow_mut_account_internal(signer::address_of(account))
   }

   fun borrow_mut_account_internal(account: address): &mut Object<Account>{
      object::borrow_mut_object_extend<Account>(account_object_id(account))
   }

   /// Borrow a resource from the account's storage
   /// This function equates to `borrow_global<T>(address)` instruction in Move
   /// But we remove the restriction of the caller must be the module of T
   public fun borrow_resource<T: key>(account: address): &T {
      let account_obj = borrow_account(account);
      account_borrow_resource_internal<T>(account_obj)
   }

   #[private_generics(T)]
   /// Borrow a mut resource from the account's storage
   /// This function equates to `borrow_global_mut<T>(address)` instruction in Move
   public fun borrow_mut_resource<T: key>(account: address): &mut T {
      let account_obj = borrow_mut_account_internal(account); 
      account_borrow_mut_resource_interal<T>(account_obj)
   }

   #[private_generics(T)]
   /// Move a resource to the account's resource object
   /// This function equates to `move_to<T>(&signer, resource)` instruction in Move
   public fun move_resource_to<T: key>(account: &signer, resource: T){
      let account_address = signer::address_of(account);
      //Auto create the resource object when move resource to the account
      ensure_account_object(account_address);
      let account_obj = borrow_mut_account_internal(account_address); 
      account_move_resource_to_internal(account_obj, resource)
   }

   #[private_generics(T)]
   /// Move a resource from the account's storage
   /// This function equates to `move_from<T>(address)` instruction in Move
   public fun move_resource_from<T: key>(account: address): T {
      let account_obj = borrow_mut_account_internal(account); 
      account_move_resource_from_internal<T>(account_obj)
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

   fun create_account_object_to(addr: address) {
      let obj = create_account_object(addr);
      object::transfer_extend(obj, addr);
   }

   fun create_account_object(addr: address): Object<Account> {
      let object_id = object::address_to_object_id(addr);
      object::new_with_object_id(object_id, Account { addr, sequence_number: 0})
   }

   fun ensure_account_object(account: address) {
      if (!exist_account_object(account)) {
         create_account_object_to(account);
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
      create_account_internal(new_address)
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
   fun drop_account_object(self: Object<Account>) {
      let obj = object::drop_unchecked(self);
      let Account {addr: _, sequence_number:_} = obj;
   }

   #[test_only]
   struct Test has key {
      addr: address,
      version: u64
   }

   #[test(sender=@0x42)]
   fun test_move_resource_to(sender: signer){
      let sender_addr = signer::address_of(&sender);
      create_account_object_to(sender_addr);
      move_resource_to(&sender, Test{
         addr: sender_addr,
         version: 1,
      });
   }

   #[test(sender=@0x42)]
   fun test_move_resource_from(sender: signer){
      let sender_addr = signer::address_of(&sender);
      create_account_object_to(sender_addr);
   
      move_resource_to(&sender, Test{
         addr: sender_addr,
         version: 1,
      });
      let Test {
         addr,
         version
      } = move_resource_from<Test>(sender_addr);
      assert!(addr == sender_addr, 0x10);
      assert!(version == 1, 0x11);
   }

   #[test(sender=@0x42)]
   #[expected_failure(abort_code = ErrorResourceAlreadyExists, location = Self)]
   fun test_failure_repeatedly_move_resource_to(sender: signer){

      let sender_addr = signer::address_of(&sender);
      create_account_object_to(sender_addr);
   
      move_resource_to(&sender, Test{
         addr: sender_addr,
         version: 1,
      });
      move_resource_to(&sender, Test{
         addr: sender_addr,
         version: 1,
      });
   }

   #[test(sender=@0x42)]
   #[expected_failure(abort_code = ErrorResourceNotExists, location = Self)]
   fun test_failure_repeatedly_move_resource_from(sender: signer){
      let sender_addr = signer::address_of(&sender);
      create_account_object_to(sender_addr);
      move_resource_to(&sender, Test{
         addr: sender_addr,
         version: 1,
      });
      let Test {
         addr,
         version: _
      } = move_resource_from<Test>(sender_addr);
      assert!(addr == sender_addr, 1);
      let Test {
         addr: _,
         version: _
      } = move_resource_from<Test>(sender_addr);
   }

   #[test(sender=@0x42)]
   fun test_borrow_resource(sender: signer){
      let sender_addr = signer::address_of(&sender);
      create_account_object_to(sender_addr);
   
      move_resource_to(&sender, Test{
         addr: sender_addr,
         version: 1,
      });

      let ref_test = borrow_resource<Test>(sender_addr);
      assert!(ref_test.version == 1, 1);
      assert!(ref_test.addr == sender_addr, 2);
   }

   #[test(sender=@0x42)]
   fun test_borrow_mut_resource(sender: signer){
      let sender_addr = signer::address_of(&sender);
      create_account_object_to(sender_addr);
      move_resource_to(&sender, Test{
         addr: sender_addr,
         version: 1,
      });
      {
         let ref_test = borrow_mut_resource<Test>(sender_addr);
         assert!(ref_test.version == 1, 1);
         assert!(ref_test.addr == sender_addr, 2);
         ref_test.version = 2;
      };
      {
         let ref_test = borrow_resource<Test>(sender_addr);
         assert!(ref_test.version == 2, 3);
      };
   }

   #[test(sender=@0x42)]
   #[expected_failure(abort_code = ErrorResourceNotExists, location = Self)]
   fun test_failure_borrow_resource_no_exists(sender: signer){
      let sender_addr = signer::address_of(&sender);
      create_account_object_to(sender_addr);
      borrow_resource<Test>(sender_addr);
   }

   #[test(sender=@0x42)]
   #[expected_failure(abort_code = ErrorResourceNotExists, location = Self)]
   fun test_failure_borrow_mut_resource_no_exists(sender: signer){
      let sender_addr = signer::address_of(&sender);
      create_account_object_to(sender_addr);
      borrow_mut_resource<Test>(sender_addr);
   }

   #[test(sender=@0x42)]
   fun test_ensure_move_from_and_exists(sender: signer){
      let sender_addr = signer::address_of(&sender);
      create_account_object_to(sender_addr);
      let test_exists = exists_resource<Test>(sender_addr);
      assert!(!test_exists, 1);
      move_resource_to(&sender, Test{
         addr: sender_addr,
         version: 1,
      });
      let test_exists = exists_resource<Test>(sender_addr);
      assert!(test_exists, 2);
      let test = move_resource_from<Test>(sender_addr);
      let test_exists = exists_resource<Test>(sender_addr);
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

   #[test_only]
   struct AccountHolder has store{
      alice: AccountCap,
      bob: AccountCap,
   }

   #[test]
   fun test_account_cap_holder(){
      let alice = create_account_and_return_cap();
      let bob = create_account_and_return_cap();
      let holder = AccountHolder{
         alice: alice,
         bob: bob,
      };
      let alice_signer = create_signer_with_account_cap(&mut holder.alice);
      let alice_addr = signer::address_of(&alice_signer);
      move_resource_to(&alice_signer, Test{
         addr: alice_addr,
         version: 1,
      });
      
      let Test{addr:_, version:_} = move_resource_from<Test>(alice_addr);

      let AccountHolder{
         alice: alice_obj,
         bob: bob_obj,
      } = holder;
      destroy_account_cap(alice_obj);
      destroy_account_cap(bob_obj);
   }

}

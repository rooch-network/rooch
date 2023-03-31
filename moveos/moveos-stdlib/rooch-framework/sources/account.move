module rooch_framework::account{
   
   use std::error;
   use std::bcs;
   use std::vector;

   /// Account already exists
   const EAccountAlreadyExists: u64 = 1;
   /// Account does not exist
   const EAccountNotExist: u64 = 2;
   /// The provided authentication key has an invalid length
   const EMalformedAuthenticationKey: u64 = 4;
   /// Cannot create account because address is reserved
   const EAddressReseved: u64 = 5;

   /// Resource representing an account.
   struct Account has key, store {
      authentication_key: vector<u8>,
      //TODO do we need a global account sequence number? every object has a sequence number
      //sequence_number: u64,
      //guid_creation_num: u64,
   }

   /// TODO should provide a entry function at this module
   /// How to provide account extension?
   public entry fun create_account_entry(auth_key: address): signer{
      Self::create_account(auth_key)
   }

   /// Publishes a new `Account` resource under `new_address`. A signer representing `new_address`
   /// is returned. This way, the caller of this function can publish additional resources under
   /// `new_address`.
   public(friend) fun create_account(new_address: address): signer {
      assert!(
         new_address != @vm_reserved && new_address != @rooch_framework,
         error::invalid_argument(EAddressReseved)
      );

      // there cannot be an Account resource under new_addr already.
      assert!(!exists<Account>(new_address), error::already_exists(EAccountAlreadyExists));      

      create_account_unchecked(new_address)
   }

   fun create_account_unchecked(new_address: address): signer {
      let new_account = create_signer(new_address);
      let authentication_key = bcs::to_bytes(&new_address);
      assert!(
         vector::length(&authentication_key) == 32,
         error::invalid_argument(EMalformedAuthenticationKey)
      );
      move_to(
         &new_account,
         Account {
               authentication_key,
         }
      );

      new_account
   }

   #[view]
   public fun exists_at(addr: address): bool {
      exists<Account>(addr)
   }


   native fun create_signer(addr: address): signer;
}
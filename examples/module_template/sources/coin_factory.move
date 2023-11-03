// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_examples::coin_factory {
   use std::string::{Self, String};
   use std::vector;
   use std::signer;
   use moveos_std::context;
   use moveos_std::move_module;

   const TEMPLATE_MODULE_ADDRESS: address = @0xdeadeadeadeadeadeadeadeadeadeadeadeadeadead;
   const TEMPLATE_MODULE_IDENTIFIER: vector<u8> = b"coin_module_identifier_placeholder";
   const TEMPLATE_COIN_STRUCT_IDENTIFIER_PLACEHOLDER: vector<u8> = b"TEMPLATE_COIN_STRUCT_IDENTIFIER_PLACEHOLDER";
   const TEMPLATE_COIN_NAME_PLACEHOLDER: vector<u8> = b"COIN_NAME_PLACEHOLDER";
   const TEMPLATE_COIN_SYMBOL_PLACEHOLDER: vector<u8> = b"COIN_SYMBOL_PLACEHOLDER";
   const TEMPLATE_COIN_SUPPLY_PLACEHOLDER: u256 = 123_321_123_456u256;
   const TEMPLATE_COIN_DECIMALS: u8 = 222u8;


   public register_fixed_supply_coin(ctx: &mut Context, account: &signer, 
      module_name: String, coin_name: String, 
      coin_symbol: String, total_supply: u256, decimals: u8
   ) {
      let sender = signer::address_of(account)
      let template_bytes = x"abc";
      let template_module = move_module::new(template_bytes);

      let modules = vector::singleton(template_module);
      let modules = move_module::binding_module_address(modules, TEMPLATE_MODULE_ADDRESS, sender);
      let modules = move_module::replace_module_identiner(
         modules, 
         vector::singleton(string::utf8(TEMPLATE_MODULE_IDENTIFIER)), 
         vector::singleton(module_name)
      );
      let modules = move_module::replace_struct_identifier(
         modules,
         vector::singleton(string::utf8(TEMPLATE_COIN_STRUCT_IDENTIFIER_PLACEHOLDER)),
         vector::singleton(string::utf8(coin_name))
      );
      let modules = move_module::replace_struct_identifier(
         modules,
         vector::singleton(string::utf8(TEMPLATE_COIN_NAME_PLACEHOLDER)),
         vector::singleton(string::utf8(coin_name))
      );
   }
}

// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_examples::coin_factory {
   use std::string::{Self, String};
   use std::vector;
   use moveos_std::signer;
   use moveos_std::table::{Self, Table};
   use moveos_std::context::{Self, Context};
   use moveos_std::move_module;

   const TEMPLATE_MODULE_ADDRESS: address = @0xdeadeadeadeadeadeadeadeadeadeadeadeadeadeadeadeadeadeadeadeadead;
   const TEMPLATE_MODULE_IDENTIFIER: vector<u8> = b"coin_module_identifier_placeholder";
   const TEMPLATE_COIN_STRUCT_IDENTIFIER_PLACEHOLDER: vector<u8> = b"COIN_STRUCT_IDENTIFIER_PLACEHOLDER";
   const TEMPLATE_COIN_NAME_PLACEHOLDER: vector<u8> = b"COIN_NAME_PLACEHOLDER";
   const TEMPLATE_COIN_SYMBOL_PLACEHOLDER: vector<u8> = b"COIN_SYMBOL_PLACEHOLDER";
   const TEMPLATE_COIN_SUPPLY_PLACEHOLDER: u256 = 123_321_123_456u256;
   const TEMPLATE_COIN_DECIMALS: u8 = 222u8;

   struct TemplateStore has key, store {
      templates: Table<String, vector<u8>>,
   }

   fun init(ctx: &mut Context) {
      let module_signer = signer::module_signer<TemplateStore>();
      let templates = context::new_table<String, vector<u8>>(ctx);
      context::move_resource_to(ctx, &module_signer, TemplateStore { templates });
      //register default template
      let name = string::utf8(b"fixed_supply_coin");
      //rooch move build -p examples/module_template/template
      //xxd -c 99999 -p examples/module_template/template/build/template/bytecode_modules/coin_module_identifier_placeholder.mv
      let template_bytes = x"a11ceb0b060000000b01001002102c033c50048c011405a001980107b802f30208ab05800106ab06590a8407110c9507780d8d08020000010102020203020403050306030700080c0000090800020a0000030e0c010001070f080108010610000108010311070001180700061a08010801000b000100000c02010004120405000313010701000214080901080315090a010807160c0d010c05170e01010c0119101100061b12130108061c140d0108071d0215010c07171601010c021e17090108030604060506060b070b090b0a0b0b0b0c0b0d0602070802060c000107080203050b05010800080601060c010501080101080602070802080601070b030109000107090001080002070b03010b040109000f010b0501090003070802050b05010900020b050108000b03010b04010800010a0201080704070802080708070201070b0801090002070b080109000f010b03010b0401090002070b03010b040109000b0501090002070802090022636f696e5f6d6f64756c655f6964656e7469666965725f706c616365686f6c64657206737472696e6707636f6e74657874066f626a656374067369676e6572126163636f756e745f636f696e5f73746f726504636f696e0a636f696e5f73746f726522434f494e5f5354525543545f4944454e5449464945525f504c414345484f4c44455208547265617375727907436f6e746578740666617563657404696e69740b64756d6d795f6669656c64064f626a65637409436f696e53746f726504436f696e084f626a65637449440a616464726573735f6f661373696e676c65746f6e5f6f626a6563745f696418626f72726f775f6d75745f6f626a6563745f657874656e640a626f72726f775f6d7574087769746864726177076465706f73697406537472696e67047574663808436f696e496e666f0f72656769737465725f657874656e640b6d696e745f657874656e64116372656174655f636f696e5f73746f72650d6e65775f73696e676c65746f6edeadeadeadeadeadeadeadeadeadeadeadeadeadeadeadeadeadeadeadeadead0000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000030201de0f20800283b61c0000000000000000000000000000000000000000000000000000000a021615434f494e5f4e414d455f504c414345484f4c4445520a021817434f494e5f53594d424f4c5f504c414345484f4c4445520002010d01010201070b03010b040108000001040003120b0111020c0238000c040a000b04380138020f004a102700000000000000000000000000000000000000000000000000000000000038030c030b000b020b03380402010000000f160a00070211080703110807003805070138060c010a0038070c020d020b0138080b000b02120138090102010000";
      register_template(ctx, name, template_bytes);
   }

   fun register_template(ctx: &mut Context, name: String, template_bytes: vector<u8>) {
      let template_store = context::borrow_mut_resource<TemplateStore>(ctx, @rooch_examples);
      table::add(&mut template_store.templates, name, template_bytes);
   }

   public entry fun register_template_entry(ctx: &mut Context, name: String, template_bytes: vector<u8>) {
      register_template(ctx, name, template_bytes);
   }

   public entry fun issue_fixed_supply_coin(ctx: &mut Context, account: &signer, 
      module_name: String, coin_name: String, 
      coin_symbol: String, total_supply: u256, decimals: u8
   ) {
      let template_store = context::borrow_mut_resource<TemplateStore>(ctx, @rooch_examples);
      let template_bytes = *table::borrow(&template_store.templates, string::utf8(b"fixed_supply_coin"));
      let template_module = move_module::new(template_bytes);

      let sender = signer::address_of(account);
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
         vector::singleton(coin_symbol)
      );

      let old_strings = vector::singleton(string::utf8(TEMPLATE_COIN_NAME_PLACEHOLDER));
      vector::push_back(&mut old_strings, string::utf8(TEMPLATE_COIN_SYMBOL_PLACEHOLDER));
      let new_strings = vector::singleton(coin_name);
      vector::push_back(&mut new_strings, coin_symbol);
      let modules = move_module::replace_constant_string(
         modules,
         old_strings,
         new_strings
      );

      let new_supply = vector::singleton(total_supply);
      let old_supply = vector::singleton(TEMPLATE_COIN_SUPPLY_PLACEHOLDER);
      let modules = move_module::replace_constant_u256(modules, old_supply, new_supply);

      let new_decimal = vector::singleton(decimals);
      let old_decimal = vector::singleton(TEMPLATE_COIN_DECIMALS);
      let modules = move_module::replace_constant_u8(modules, old_decimal, new_decimal);

      // publish modules
      context::publish_modules(ctx, account, modules);
   }
}

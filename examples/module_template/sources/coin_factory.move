// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_examples::coin_factory {
   use std::string::{Self, String};
   use std::vector;
   use moveos_std::account;
   use moveos_std::signer;
   use moveos_std::table::{Self, Table};
   
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

   fun init() {
      let module_signer = signer::module_signer<TemplateStore>();
      let templates = table::new<String, vector<u8>>();
      account::move_resource_to(&module_signer, TemplateStore { templates });
      //register default template
      let name = string::utf8(b"fixed_supply_coin");
      //rooch move build -p examples/module_template/template
      //xxd -c 99999 -p examples/module_template/template/build/template/bytecode_modules/coin_module_identifier_placeholder.mv
      let template_bytes = x"a11ceb0b060000000b01000e020e24033250048201140596019c0107b202c40208f604800106f605590acf06110ce006710dd10702000001010202020303040305030600070c000008080002090c010001060d08010801050e0001080105130c01080101140700000a000100000b010100030f0304000210060701080611090a010c04120b01010c01150d0e0005160f1001080517110a010802181301010806190114010c06121501010c021a16130108021b130101080305040805080708080809120a080b080c050d0502060c070b020108010002050b0401080001060c010501080101070b020109000107090001080002070b02010b030109000f010b0401090002050b04010900030b040108000b02010b050108000b02010b03010800010a02010806030806080602010b02010b0501090002070b02010b050109000f010b05010800010b02010900010b02010b0301090002070b02010b030109000b0401090001090022636f696e5f6d6f64756c655f6964656e7469666965725f706c616365686f6c64657206737472696e67066f626a656374067369676e6572126163636f756e745f636f696e5f73746f726504636f696e0a636f696e5f73746f726522434f494e5f5354525543545f4944454e5449464945525f504c414345484f4c444552085472656173757279064f626a6563740666617563657404696e69740b64756d6d795f6669656c6409436f696e53746f726504436f696e0a616464726573735f6f660a626f72726f775f6d7574087769746864726177076465706f73697408436f696e496e666f06537472696e6704757466380f72656769737465725f657874656e640b6d696e745f657874656e6409746f5f66726f7a656e116372656174655f636f696e5f73746f7265106e65775f6e616d65645f6f626a65637409746f5f736861726564deadeadeadeadeadeadeadeadeadeadeadeadeadeadeadeadeadeadeadeadead0000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000030201de0f20800283b61c0000000000000000000000000000000000000000000000000000000a021615434f494e5f4e414d455f504c414345484f4c4445520a021817434f494e5f53594d424f4c5f504c414345484f4c4445520002010c01010201060b02010b0301080000010400020d0b0011020c020b0138000f004a102700000000000000000000000000000000000000000000000000000000000038010c030b020b03380202010000000c170702110607031106070038030c010d01070138040c000b01380538060c020d020b0038070b0212013808380902010000";
      register_template(name, template_bytes);
   }

   fun register_template(name: String, template_bytes: vector<u8>) {
      let template_store = account::borrow_mut_resource<TemplateStore>(@rooch_examples);
      table::add(&mut template_store.templates, name, template_bytes);
   }

   public entry fun register_template_entry(name: String, template_bytes: vector<u8>) {
      register_template(name, template_bytes);
   }

   // TODO: uncomment this once move_module::binding_module_address is ready
   // public entry fun issue_fixed_supply_coin(account: &signer, 
   //    module_name: String, coin_name: String, 
   //    coin_symbol: String, total_supply: u256, decimals: u8
   // ) {
   //    let template_store = account::borrow_mut_resource<TemplateStore>(@rooch_examples);
   //    let template_bytes = *table::borrow(&template_store.templates, string::utf8(b"fixed_supply_coin"));
   //    let template_module = move_module::new(template_bytes);

   //    let sender = signer::address_of(account);
   //    let modules = vector::singleton(template_module);
   //    let modules = move_module::binding_module_address(modules, TEMPLATE_MODULE_ADDRESS, sender);
   //    let modules = move_module::replace_module_identiner(
   //       modules, 
   //       vector::singleton(string::utf8(TEMPLATE_MODULE_IDENTIFIER)), 
   //       vector::singleton(module_name)
   //    );
   //    let modules = move_module::replace_struct_identifier(
   //       modules,
   //       vector::singleton(string::utf8(TEMPLATE_COIN_STRUCT_IDENTIFIER_PLACEHOLDER)),
   //       vector::singleton(coin_symbol)
   //    );

   //    let old_strings = vector::singleton(string::utf8(TEMPLATE_COIN_NAME_PLACEHOLDER));
   //    vector::push_back(&mut old_strings, string::utf8(TEMPLATE_COIN_SYMBOL_PLACEHOLDER));
   //    let new_strings = vector::singleton(coin_name);
   //    vector::push_back(&mut new_strings, coin_symbol);
   //    let modules = move_module::replace_constant_string(
   //       modules,
   //       old_strings,
   //       new_strings
   //    );

   //    let new_supply = vector::singleton(total_supply);
   //    let old_supply = vector::singleton(TEMPLATE_COIN_SUPPLY_PLACEHOLDER);
   //    let modules = move_module::replace_constant_u256(modules, old_supply, new_supply);

   //    let new_decimal = vector::singleton(decimals);
   //    let old_decimal = vector::singleton(TEMPLATE_COIN_DECIMALS);
   //    let modules = move_module::replace_constant_u8(modules, old_decimal, new_decimal);

   //    let module_store = move_module::borrow_mut_module_store();
   //    // publish modules
   //    move_module::publish_modules(module_store, account, modules);
   // }
}

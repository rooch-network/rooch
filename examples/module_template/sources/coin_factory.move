// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_examples::coin_factory {
   use std::string::{Self, String};
   use moveos_std::account;
   use moveos_std::signer;
   use moveos_std::table::{Self, Table};

   const TEMPLATE_MODULE_ADDRESS: address = @0xdeadeadeadeadeadeadeadeadeadeadeadeadeadeadeadeadeadeadeadeadead;
   const TEMPLATE_MODULE_IDENTIFIER: vector<u8> = b"coin_module_identifier_placeholder";
   const TEMPLATE_COIN_STRUCT_IDENTIFIER_PLACEHOLDER: vector<u8> = b"COIN_STRUCT_IDENTIFIER_PLACEHOLDER";
   const TEMPLATE_COIN_NAME_PLACEHOLDER: vector<u8> = b"COIN_NAME_PLACEHOLDER";
   const TEMPLATE_COIN_SYMBOL_PLACEHOLDER: vector<u8> = b"COIN_SYMBOL_PLACEHOLDER";
   const TEMPLATE_COIN_SUPPLY_PLACEHOLDER: u256 = 123_321_123_456u256;
   const TEMPLATE_COIN_DECIMALS: u8 = 222u8;

   struct TemplateStore has key {
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
      let template_bytes = x"a11ceb0b0700000a0c01001002102a033a65049f011605b501a00107d502d70208ac05800106ac063410e006280a8807110c99078a010da308020000010602040309030c020f011a021e00010c000003080001050c010001020708010801030b0700040e0701000005110c010801051300010801000800000001030a01020001040d0003010001051005060108010512070801080101140a000108010215000b010c0102160c00010c0101170e0a01080101180a00010801001910000001061b11120001011c1314010801021d1508010c0107161600010c01020203040404050906040704080d090d0c0d0d040e0400010a02010804010b0501090001080004080408040b0501080402010b02010b0601090002070b02010b060109000f010b07010900010b06010800010b02010900010b02010b0301090002070b02010b030109000b07010900010801010900030b02010b060108000b070108000b02010b0301080002060c070b0201080101060c010501070b020109000107090002070b02010b030109000f02050b0701090022636f696e5f6d6f64756c655f6964656e7469666965725f706c616365686f6c64657222434f494e5f5354525543545f4944454e5449464945525f504c414345484f4c4445520b64756d6d795f6669656c640854726561737572790a636f696e5f73746f7265064f626a656374066f626a65637409436f696e53746f726504696e697406737472696e67047574663806537472696e67066f7074696f6e046e6f6e65064f7074696f6e04636f696e0f72656769737465725f657874656e6408436f696e496e666f0b6d696e745f657874656e6404436f696e09746f5f66726f7a656e116372656174655f636f696e5f73746f7265076465706f736974106e65775f6e616d65645f6f626a65637409746f5f73686172656406666175636574067369676e65720a616464726573735f6f660a626f72726f775f6d7574087769746864726177126163636f756e745f636f696e5f73746f7265deadeadeadeadeadeadeadeadeadeadeadeadeadeadeadeadeadeadeadeadead0000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000300000000000000000000000000000000000000000000000000000000000000010a021615434f494e5f4e414d455f504c414345484f4c4445520a021817434f494e5f53594d424f4c5f504c414345484f4c44455214636f6d70696c6174696f6e5f6d6574616461746112010c322e312d756e737461626c6503322e310002010201010201040b02010b03010800000000000f180700110107011101380031de38010c000d004a800283b61c00000000000000000000000000000000000000000000000000000038020c010b00380338040c020d020b0138050b02120138063807020a01040000090b00110b0b0138080f004a10270000000000000000000000000000000000000000000000000000000000003809380a02010000";
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

   //    let module_store = module_store::borrow_mut_module_store();
   //    // publish modules
   //    module_store::publish_modules(module_store, account, modules);
   // }
}

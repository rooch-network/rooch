// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module bitcoin_move::brc20 {
    use std::option::{Self, Option};
    use std::string::{Self, String};
    use moveos_std::object_id;
    use moveos_std::context::{Self, Context};
    use moveos_std::object::{Self, Object};
    use moveos_std::table::{Self, Table};
    use moveos_std::simple_map::{Self, SimpleMap};
    use moveos_std::string_utils;
    #[test_only]
    use moveos_std::json;

    friend bitcoin_move::genesis;
    friend bitcoin_move::ord;

    //TODO should we register the BRC20 as a CoinInfo?
    struct BRC20CoinInfo has store, copy{
        tick: String,
        max: u256,
        lim: u256,
        dec: u64,
        supply: u256,
    }

    struct BRC20Balance has store{
        info: BRC20CoinInfo,
        balance: Table<address, u256>, 
    } 

    struct BRC20Store has key {
        coins: Table<String, BRC20Balance>,
    }

    public(friend) fun genesis_init(ctx: &mut Context, _genesis_account: &signer){
        let brc20_store = BRC20Store{
            coins: context::new_table(ctx),
        }; 
        let obj = context::new_named_object(ctx, brc20_store);
        object::to_shared(obj);
    }

    fun borrow_store(ctx: &mut Context) : &mut BRC20Store {
        let brc20_store_object_id = object_id::named_object_id<BRC20Store>();
        let brc20_store_obj = context::borrow_mut_object_shared<BRC20Store>(ctx, brc20_store_object_id);
        object::borrow_mut(brc20_store_obj)
    }

    /// The brc20 operation
    struct Op has store {
        from: address,
        to: address,
        json_map: SimpleMap<String, String>,
    }

    public(friend) fun new_op(from: address, to: address, json_map: SimpleMap<String, String>) : Op {
        Op { from, to, json_map }
    }

    public fun clone_op(self: &Op) : Op {
        let json_map = simple_map::clone(&self.json_map);
        Op { from: self.from, to: self.to, json_map }
    }

    public fun drop_op(op: Op){
        let Op{from:_, to:_, json_map} = op;
        simple_map::drop(json_map);
    }

    /// The brc20 deploy operation
    /// https://domo-2.gitbook.io/brc-20-experiment/
    /// ```json
    /// { 
    /// "p": "brc-20",
    /// "op": "deploy",
    /// "tick": "ordi",
    /// "max": "21000000",
    /// "lim": "1000"
    ///}
    /// ```
    struct DeployOp has store,copy,drop {
        from: address,
        to: address,
        tick: String,
        max: String,
        //Mint limit: If letting users mint to themsleves, limit per ordinal
        lim: String,
        //Decimals: set decimal precision, default to 18
        dec: String,
    }

    /// The brc20 mint operation
    /// https://domo-2.gitbook.io/brc-20-experiment/
    /// ```json
    /// { 
    /// "p": "brc-20",
    /// "op": "mint",
    /// "tick": "ordi",
    /// "amt": "1000"
    /// }
    /// ```
    struct MintOp has store,copy,drop {
        from: address,
        to: address,
        tick: String,
        amt: String,
    }

    /// The brc20 transfer operation
    /// https://domo-2.gitbook.io/brc-20-experiment/
    /// ```json
    /// {
    /// "p": "brc-20",
    /// "op": "transfer",
    /// "tick": "ordi",
    /// "to": "", 
    /// "amt": "100"
    /// }
    struct TransferOp has store,copy,drop {
        from: address,
        to: address,
        tick: String,
        amt: String,
        //TODO we need the to field?
    }

    public fun is_brc20(json_map: &SimpleMap<String,String>) : bool {
        let protocol_key = string::utf8(b"p");
        simple_map::contains_key(json_map, &protocol_key) && simple_map::borrow(json_map, &protocol_key) == &string::utf8(b"brc-20")
    }

    fun is_deploy(self: &Op) : bool {
        let op_key = string::utf8(b"op");
        simple_map::contains_key(&self.json_map, &op_key) && simple_map::borrow(&self.json_map, &op_key) == &string::utf8(b"deploy")
    }

    fun as_deploy(self: &Op) : Option<DeployOp> {
        let deploy_op = if (is_deploy(self)) {
            let tick_key = string::utf8(b"tick");
            let max_key = string::utf8(b"max");
            if(simple_map::contains_key(&self.json_map, &tick_key) && simple_map::contains_key(&self.json_map,&max_key)) {
                let tick = *simple_map::borrow(&self.json_map, &tick_key);
                let tick = string_utils::to_lower_case(&tick);
                let dec = *simple_map::borrow_with_default(&self.json_map, &string::utf8(b"dec"), &string::utf8(b"18"));
                let max = *simple_map::borrow(&self.json_map,&max_key);
                let lim = *simple_map::borrow_with_default(&self.json_map, &string::utf8(b"lim"), &string::utf8(b"0"));
                option::some(DeployOp { from: self.from, to: self.to, tick, max, lim, dec })
            } else {
                option::none()
            }
        } else {
            option::none()
        };
        deploy_op
    }

    fun execute_deploy(ctx: &mut Context, deploy: DeployOp): bool{
        let balance_table_id = context::fresh_uid(ctx);
        let brc20_store = borrow_store(ctx);
        if(table::contains(&brc20_store.coins, deploy.tick)){
            std::debug::print(&string::utf8(b"brc20 already exists"));
            return false
        };
        
        let tick = deploy.tick;

        let dec = option::destroy_with_default(string_utils::parse_u64_option(&deploy.dec), 18u64);
        let max_opt = string_utils::parse_decimal_option(&deploy.max, dec);
        if(option::is_none(&max_opt)){
            return false
        };
        let lim = option::destroy_with_default(string_utils::parse_decimal_option(&deploy.lim, dec), 0u256);
        let max = option::destroy_some(max_opt);
        let coin_info = BRC20CoinInfo{ tick, max, lim, dec , supply: 0u256};
        let balance_info = BRC20Balance{ info: coin_info, balance:table::new(balance_table_id) };
        table::add(&mut brc20_store.coins, tick, balance_info);
        true
    }

    fun is_mint(self: &Op) : bool {
        let op_key = string::utf8(b"op");
        simple_map::contains_key(&self.json_map, &op_key) && simple_map::borrow(&self.json_map, &op_key) == &string::utf8(b"mint")
    }

    fun as_mint(self: &Op) : Option<MintOp> {
        let mint_op = if (is_mint(self)) {
            let tick_key = string::utf8(b"tick");
            let amt_key = string::utf8(b"amt");
            if(simple_map::contains_key(&self.json_map, &tick_key) && simple_map::contains_key(&self.json_map,&amt_key)) {
                let tick = *simple_map::borrow(&self.json_map, &tick_key);
                let tick = string_utils::to_lower_case(&tick);
                let amt = *simple_map::borrow(&self.json_map,&amt_key);
                option::some(MintOp { from: self.from, to: self.to, tick, amt })
            } else {
                option::none()
            }
        } else {
            option::none()
        };
        mint_op
    }

    fun execute_mint(ctx: &mut Context, mint: MintOp): bool{
        let brc20_store = borrow_store(ctx);
        if(!table::contains(&brc20_store.coins, mint.tick)){
            std::debug::print(&string::utf8(b"brc20 does not exist"));
            return false
        };

        let balance_info = table::borrow_mut(&mut brc20_store.coins, mint.tick);
        let coin_info = &mut balance_info.info;
        let lim = coin_info.lim;
       
        let amt_opt = string_utils::parse_decimal_option(&mint.amt, coin_info.dec);
        if(option::is_none(&amt_opt)){
            return false
        };
        let amt = option::destroy_some(amt_opt);

         if(lim > 0 && amt > lim){
            std::debug::print(&string::utf8(b"brc20 mint lim exceeded"));
            return false
        };
        let new_total_supply = coin_info.supply + amt;
        if(new_total_supply > coin_info.max){
            std::debug::print(&string::utf8(b"brc20 max exceeded"));
            return false
        };
        coin_info.supply = new_total_supply;
        let balance = table::borrow_mut_with_default(&mut balance_info.balance, mint.to, 0);
        *balance = *balance + amt;
        true
    }

    fun is_transfer(self: &Op) : bool {
        let op_key = string::utf8(b"op");
        simple_map::contains_key(&self.json_map, &op_key) && simple_map::borrow(&self.json_map, &op_key) == &string::utf8(b"transfer")
    }

    fun as_transfer(self: &Op) : Option<TransferOp> {
        let transfer_op = if (is_transfer(self)) {
            let tick_key = string::utf8(b"tick");
            let amt_key = string::utf8(b"amt");
            if(simple_map::contains_key(&self.json_map, &tick_key) && simple_map::contains_key(&self.json_map,&amt_key)) {
                let tick = *simple_map::borrow(&self.json_map, &tick_key); 
                let tick = string_utils::to_lower_case(&tick);
                let amt = *simple_map::borrow(&self.json_map,&amt_key);
                option::some(TransferOp { from: self.from, to: self.to, tick, amt })
            } else {
                option::none()
            }
        } else {
            option::none()
        };
        transfer_op
    }

    fun execute_transfer(ctx: &mut Context, transfer: TransferOp): bool{
        let from = transfer.from;
        let to = transfer.to;
        let brc20_store = borrow_store(ctx);
        if(!table::contains(&brc20_store.coins, transfer.tick)){
            std::debug::print(&string::utf8(b"brc20 does not exist"));
            return false
        };
        
        let balance_info = table::borrow_mut(&mut brc20_store.coins, transfer.tick);
        let coin_info = &balance_info.info;
        let amt_opt = string_utils::parse_decimal_option(&transfer.amt, coin_info.dec);
        if(option::is_none(&amt_opt)){
            return false
        };
        let amt = option::destroy_some(amt_opt);

        let from_balance = table::borrow_mut_with_default(&mut balance_info.balance, from, 0);
        if(*from_balance < amt){
            std::debug::print(&string::utf8(b"brc20 insufficient balance"));
            false
        }else{
            *from_balance = *from_balance - amt;
            let to_balance = table::borrow_mut_with_default(&mut balance_info.balance, to, 0);
            *to_balance = *to_balance + amt;
            true
        }
    }

    public(friend) fun process_utxo_op(ctx: &mut Context, op: Op) : bool {
        let result = if(is_transfer(&op)){
            let transfer_op_opt = as_transfer(&op);
            if(option::is_some(&transfer_op_opt)){
                let transfer_op = option::destroy_some(transfer_op_opt);
                execute_transfer(ctx, transfer_op)
            }else{
                std::debug::print(&string::utf8(b"invalid transfer op"));
                std::debug::print(&op);
                false
            }
        }else{
            // UTXO op is not a transfer, so we ignore it
            true
        };
        drop_op(op);
        result
    }

    public(friend) fun process_inscribe_op(ctx: &mut Context, op: Op) :bool {
        
        let result = if(is_deploy(&op)){
            let deploy_op_opt = as_deploy(&op);
            if(option::is_none(&deploy_op_opt)){
                std::debug::print(&string::utf8(b"invalid deploy op"));
                std::debug::print(&op);
                false
            }else{
                let deploy_op = option::destroy_some(deploy_op_opt);
                execute_deploy(ctx, deploy_op)
            }
        }else if(is_mint(&op)){
            let mint_op_opt = as_mint(&op);
            if(option::is_none(&mint_op_opt)){
                std::debug::print(&string::utf8(b"invalid mint op"));
                std::debug::print(&op);
                false
            }else{
                let mint_op = option::destroy_some(mint_op_opt);
                execute_mint(ctx, mint_op)
            }
        }else if(is_transfer(&op)){
            let transfer_op_opt = as_transfer(&op);
            if(option::is_none(&transfer_op_opt)){
                std::debug::print(&string::utf8(b"invalid transfer op"));
                std::debug::print(&op);
                false
            }else{
                let transfer_op = option::destroy_some(transfer_op_opt);
                execute_transfer(ctx, transfer_op)
            }
        }else{
            std::debug::print(&string::utf8(b"unknown brc20 op"));
            false
        };
        if(!result){
            std::debug::print(&string::utf8(b"failed to progress brc20 op"));
            std::debug::print(&op);
        };
        drop_op(op);
        result
    }

    //=== Brc20 store ===

    public fun get_tick_info(brc20_store_obj:&Object<BRC20Store>, tick: &String) : Option<BRC20CoinInfo> {
        let tick = string_utils::to_lower_case(tick);
        let brc20_store = object::borrow(brc20_store_obj);
        if(table::contains(&brc20_store.coins, tick)){
            option::some(*&table::borrow(&brc20_store.coins, tick).info)
        }else{
            option::none()
        }
    }

    public fun get_balance(brc20_store_obj:&Object<BRC20Store>, tick: &String, address: address) : u256 {
        let tick = string_utils::to_lower_case(tick);
        let brc20_store = object::borrow(brc20_store_obj);
        if(table::contains(&brc20_store.coins, tick)){
            let balance_info = table::borrow(&brc20_store.coins, tick);
            *table::borrow_with_default(&balance_info.balance, address, &0u256)
        }else{
            0u256
        }
    }

    #[test]
    fun test_deploy_op(){
        let deploy_op_json = b"{\"p\":\"brc-20\",\"op\":\"deploy\",\"tick\":\"ordi\",\"max\":\"21000000\",\"lim\":\"1000\"}";
        let json_map = json::to_map(deploy_op_json);
        assert!(is_brc20(&json_map), 1);
        let from = @0x42;
        let to = @0x42;
        let op = Op { from, to, json_map };
        
        assert!(is_deploy(&op), 2);
        let deploy_op_opt = as_deploy(&op);
        assert!(option::is_some(&deploy_op_opt), 3);
        let deploy_op = option::destroy_some(deploy_op_opt);
        assert!(deploy_op.tick == string::utf8(b"ordi"), 4);
        assert!(deploy_op.max == string::utf8(b"21000000"), 5);
        assert!(deploy_op.lim == string::utf8(b"1000"), 6);
        assert!(deploy_op.dec == string::utf8(b"18"), 7);

        drop_op(op);
    }

    #[test]
    fun test_mint_op(){
        let mint_op_json = b"{\"p\":\"brc-20\",\"op\":\"mint\",\"tick\":\"ordi\",\"amt\":\"1000\"}";
        let json_map = json::to_map(mint_op_json);
        assert!(is_brc20(&json_map), 1);
        let from = @0x42;
        let to = @0x42;
        let op = Op { from, to, json_map };
        assert!(is_mint(&op), 2);
        let mint_op_opt = as_mint(&op);
        assert!(option::is_some(&mint_op_opt), 3);
        let mint_op = option::destroy_some(mint_op_opt);
        assert!(mint_op.tick == string::utf8(b"ordi"), 4);
        assert!(mint_op.amt == string::utf8(b"1000"), 5);
        drop_op(op);
    }

    #[test]
    fun test_transfer_op(){
        let transfer_op_json = b"{\"p\":\"brc-20\",\"op\":\"transfer\",\"tick\":\"ordi\",\"amt\":\"1000\"}";
        let json_map = json::to_map(transfer_op_json);
        assert!(is_brc20(&json_map), 1);
        let from = @0x42;
        let to = @0x43;
        let op = Op { from, to, json_map };
        assert!(is_transfer(&op), 2);
        let transfer_op_opt = as_transfer(&op);
        assert!(option::is_some(&transfer_op_opt), 3);
        let transfer_op = option::destroy_some(transfer_op_opt);
        assert!(transfer_op.tick == string::utf8(b"ordi"), 4);
        assert!(transfer_op.amt == string::utf8(b"1000"), 5);
        drop_op(op);
    }

    #[test(genesis_account=@0x4)]
    fun test_brc20_roundtrip(genesis_account: &signer){
        let ctx = moveos_std::context::new_test_context(@rooch_framework);

        genesis_init(&mut ctx, genesis_account);
        
        let deploy_op_json = b"{\"p\":\"brc-20\",\"op\":\"deploy\",\"tick\":\"ordi\",\"max\":\"21000000\",\"lim\":\"1000\"}";
        let deployer = @0x42;
        let minter = @0x43;
        let transfer_to = @0x44;
        let op = Op { from: deployer, to: deployer, json_map: json::to_map(deploy_op_json) };
        let deploy_op = option::destroy_some(as_deploy(&op));
        assert!(execute_deploy(&mut ctx, deploy_op), 1);
        drop_op(op);

        let mint_op_json = b"{\"p\":\"brc-20\",\"op\":\"mint\",\"tick\":\"ordi\",\"amt\":\"1000\"}";
        let op = Op { from: minter, to: minter, json_map: json::to_map(mint_op_json) };
        let mint_op = option::destroy_some(as_mint(&op));
        assert!(execute_mint(&mut ctx, mint_op), 2);
        drop_op(op);
        
        let transfer_op_json = b"{\"p\":\"brc-20\",\"op\":\"transfer\",\"tick\":\"ordi\",\"amt\":\"1000\"}";
        let op = Op { from: minter, to: transfer_to, json_map: json::to_map(transfer_op_json) };
        let transfer_op = option::destroy_some(as_transfer(&op));
        assert!(execute_transfer(&mut ctx, transfer_op), 3);
        drop_op(op);
        
        let brc20_store = borrow_store(&mut ctx);
        let balance_info = table::borrow(&brc20_store.coins, string::utf8(b"ordi"));
        let coin_info = &balance_info.info;
        assert!(coin_info.supply == 1000000000000000000000u256, 4);
        let balance1 = *table::borrow(&balance_info.balance, minter);
        assert!(balance1 == 0u256, 5);
        let balance2 = *table::borrow(&balance_info.balance, transfer_to);
        assert!(balance2 == 1000000000000000000000u256, 6);
        context::drop_test_context(ctx);

    }
}
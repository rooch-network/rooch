// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_framework::brc20 {
    //use std::vector;
    use std::option::{Self, Option};
    use std::string::{Self, String};
    use moveos_std::json;
    use moveos_std::context::{Self, Context};
    use moveos_std::object::{Self, Object};
    use moveos_std::table::{Self, Table};
    use moveos_std::table_vec;
    use moveos_std::simple_map::{Self, SimpleMap};
    use moveos_std::string_utils;
    use rooch_framework::bitcoin_address::{BTCAddress};
    use rooch_framework::ord::{Self, Inscription, InscriptionStore};
    // use rooch_framework::bitcoin_types::{Self, Witness, Transaction};
    // use rooch_framework::bitcoin_light_client::{Self, BitcoinBlockStore};

    friend rooch_framework::genesis;

    //TODO should we register the BRC20 as a CoinInfo?
    struct BRC20CoinInfo has store{
        tick: String,
        max: u64,
        lim: u64,
        dec: u64,
    } 

    struct BRC20Store has key {
        next_inscription_index: u64,
        coins: Table<String, BRC20CoinInfo>,
        balance: Table<BTCAddress, u64>,
    }

    public(friend) fun genesis_init(ctx: &mut Context, _genesis_account: &signer){
        let brc20_store = BRC20Store{
            next_inscription_index: 0,
            coins: context::new_table(ctx),
            balance: context::new_table(ctx),
        }; 
        let obj = context::new_named_object(ctx, brc20_store);
        object::to_shared(obj);
    }

    /// The brc20 operation
    struct Op has store, copy, drop {
        json_map: SimpleMap<String, String>,
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
        tick: String,
        max: u64,
        //Mint limit: If letting users mint to themsleves, limit per ordinal
        lim: u64,
        //Decimals: set decimal precision, default to 18
        dec: u64,
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
        tick: String,
        amt: u64,
    }

    /// The brc20 transfer operation
    /// https://domo-2.gitbook.io/brc-20-experiment/
    /// ```json
    /// {
    /// "p": "brc-20",
    /// "op": "transfer",
    /// "tick": "ordi",
    /// "amt": "100"
    /// }
    struct TransferOp has store,copy,drop {
        tick: String,
        amt: u64,
    }

    public fun is_brc20(self: &Op) : bool {
        let protocol_key = string::utf8(b"p");
        simple_map::contains_key(&self.json_map, &protocol_key) && simple_map::borrow(&self.json_map, &protocol_key) == &string::utf8(b"brc-20")
    }

    public fun is_deploy(self: &Op) : bool {
        let op_key = string::utf8(b"op");
        simple_map::contains_key(&self.json_map, &op_key) && simple_map::borrow(&self.json_map, &op_key) == &string::utf8(b"deploy")
    }

    public fun as_deploy(self: &Op) : Option<DeployOp> {
        if (is_brc20(self) && is_deploy(self)) {
            let tick_key = string::utf8(b"tick");
            let max_key = string::utf8(b"max");
            if(simple_map::contains_key(&self.json_map, &tick_key) && simple_map::contains_key(&self.json_map,&max_key)) {
                let tick = *simple_map::borrow(&self.json_map, &tick_key);
                let max_str = simple_map::borrow(&self.json_map,&max_key);
                let lim_str = simple_map::borrow_with_default(&self.json_map, &string::utf8(b"lim"), &string::utf8(b"0"));
                let dec_str = simple_map::borrow_with_default(&self.json_map, &string::utf8(b"dec"), &string::utf8(b"18"));
                let max_opt = string_utils::to_u64_option(max_str);
                if(option::is_none(&max_opt)){
                    return option::none()
                };
                let max = option::destroy_some(max_opt);
                let lim = option::destroy_with_default(string_utils::to_u64_option(lim_str), 0u64);
                let dec = option::destroy_with_default(string_utils::to_u64_option(dec_str), 18u64);
                option::some(DeployOp { tick, max, lim, dec })
            } else {
                option::none()
            }
        } else {
            option::none()
        }
    }

    fun execute_deploy(brc20_store: &mut BRC20Store, deploy: DeployOp): bool{
        if(table::contains(&brc20_store.coins, deploy.tick)){
            std::debug::print(&string::utf8(b"brc20 already exists"));
            return false
        };
        
        let tick = deploy.tick;
        let max = deploy.max;
        let lim = deploy.lim;
        let dec = deploy.dec;
        let coin_info = BRC20CoinInfo{ tick, max, lim, dec };
        table::add(&mut brc20_store.coins, tick, coin_info);
        true
    }

    public fun is_mint(self: &Op) : bool {
        let op_key = string::utf8(b"op");
        simple_map::contains_key(&self.json_map, &op_key) && simple_map::borrow(&self.json_map, &op_key) == &string::utf8(b"mint")
    }

    public fun as_mint(self: &Op) : Option<MintOp> {
        if (is_brc20(self) && is_mint(self)) {
            let tick_key = string::utf8(b"tick");
            let amt_key = string::utf8(b"amt");
            if(simple_map::contains_key(&self.json_map, &tick_key) && simple_map::contains_key(&self.json_map,&amt_key)) {
                let tick = *simple_map::borrow(&self.json_map, &tick_key);
                let amt_str = simple_map::borrow(&self.json_map,&amt_key);
                let amt_opt = string_utils::to_u64_option(amt_str);
                if(option::is_none(&amt_opt)){
                    return option::none()
                };
                let amt = option::destroy_some(amt_opt);
                option::some(MintOp { tick, amt })
            } else {
                option::none()
            }
        } else {
            option::none()
        }
    }

    fun execute_mint(brc20_store: &mut BRC20Store, mint: MintOp, sender: BTCAddress): bool{
        if(!table::contains(&brc20_store.coins, mint.tick)){
            std::debug::print(&string::utf8(b"brc20 does not exist"));
            return false
        };

        let coin_info = table::borrow(&brc20_store.coins, mint.tick);
        let max = coin_info.max;
        let lim = coin_info.lim;
        if(lim > 0 && mint.amt > lim){
            std::debug::print(&string::utf8(b"brc20 mint lim exceeded"));
            return false
        };
        let amt = mint.amt;
        let balance = table::borrow_mut_with_default(&mut brc20_store.balance, sender, 0);
        let next_balance = *balance + amt;
        if(next_balance > max){
            std::debug::print(&string::utf8(b"brc20 max exceeded"));
            false
        }else{
            *balance = next_balance;
            true
        }
        
    }

    public fun is_transfer(self: &Op) : bool {
        let op_key = string::utf8(b"op");
        simple_map::contains_key(&self.json_map, &op_key) && simple_map::borrow(&self.json_map, &op_key) == &string::utf8(b"transfer")
    }

    public fun as_transfer(self: &Op) : Option<TransferOp> {
        if (is_brc20(self) && is_transfer(self)) {
            let tick_key = string::utf8(b"tick");
            let amt_key = string::utf8(b"amt");
            if(simple_map::contains_key(&self.json_map, &tick_key) && simple_map::contains_key(&self.json_map,&amt_key)) {
                let tick = *simple_map::borrow(&self.json_map, &tick_key);
                let amt_str = simple_map::borrow(&self.json_map,&amt_key);
                let amt_opt = string_utils::to_u64_option(amt_str);
                if(option::is_none(&amt_opt)){
                    return option::none()
                };
                let amt = option::destroy_some(amt_opt);
                option::some(TransferOp { tick, amt })
            } else {
                option::none()
            }
        } else {
            option::none()
        }
    }

    fun execute_transfer(brc20_store: &mut BRC20Store, transfer: TransferOp, sender: BTCAddress, receiver: BTCAddress): bool{
        if(!table::contains(&brc20_store.coins, transfer.tick)){
            std::debug::print(&string::utf8(b"brc20 does not exist"));
            return false
        };
    
        let amt = transfer.amt;
        let sender_balance = table::borrow_mut_with_default(&mut brc20_store.balance, sender, 0);
        if(*sender_balance < amt){
            std::debug::print(&string::utf8(b"brc20 insufficient balance"));
            false
        }else{
            *sender_balance = *sender_balance - amt;
            let receiver_balance = table::borrow_mut_with_default(&mut brc20_store.balance, receiver, 0);
            *receiver_balance = *receiver_balance + amt;
            true
        }
    }

    public fun from_inscription(inscription: &Inscription) : Option<Op> {
        let body_opt = ord::body(inscription);
        if (option::is_none(&body_opt)) {
            return option::none()
        };
        let body = option::destroy_some(body_opt);
        let json_map = json::to_map(body);
        if(simple_map::length(&json_map) == 0){
            return option::none()
        };
        option::some(Op { json_map })
    }

    fun progress_op(brc20_store: &mut BRC20Store, op: Op) {
        if(!is_brc20(&op)){
            std::debug::print(&string::utf8(b"not brc20 op"));
            std::debug::print(&op);
            return
        };
        if(is_deploy(&op)){
            let deploy_op_opt = as_deploy(&op);
            if(option::is_none(&deploy_op_opt)){
                std::debug::print(&string::utf8(b"invalid deploy op"));
                std::debug::print(&op);
                return
            };
            let deploy_op = option::destroy_some(deploy_op_opt);
            let result = execute_deploy(brc20_store, deploy_op);
            if(!result){
                std::debug::print(&string::utf8(b"failed to execute deploy op"));
                std::debug::print(&op);
                return
            };
        }else if(is_mint(&op)){
            let mint_op_opt = as_mint(&op);
            if(option::is_none(&mint_op_opt)){
                std::debug::print(&string::utf8(b"invalid mint op"));
                std::debug::print(&op);
                return
            };
            let mint_op = option::destroy_some(mint_op_opt);
            //TODO get sender from inscription and handle mint_op
            std::debug::print(&mint_op);
        }else if(is_transfer(&op)){
            let transfer_op_opt = as_transfer(&op);
            if(option::is_none(&transfer_op_opt)){
                std::debug::print(&string::utf8(b"invalid transfer op"));
                std::debug::print(&op);
                return
            };
            let transfer_op = option::destroy_some(transfer_op_opt);
            //TODO get sender and receiver from inscription and handle transfer_op
            std::debug::print(&transfer_op);
        }else{
            std::debug::print(&string::utf8(b"unknown brc20 op"));
            std::debug::print(&op);
            return
        }
    }

    public fun remaining_inscription_count(inscription_store_obj: &Object<InscriptionStore>, brc20_store_obj: &Object<BRC20Store>): u64{
        let brc20_store = object::borrow(brc20_store_obj);
        let start_inscription_index = brc20_store.next_inscription_index;
        let max_inscription_count = table_vec::length(ord::inscription_ids(inscription_store_obj));
        if(start_inscription_index < max_inscription_count){
            max_inscription_count - start_inscription_index
        }else{
            0
        }
    }

    entry fun progress_brc20_ops(inscription_store_obj: &Object<InscriptionStore>, brc20_store_obj: &mut Object<BRC20Store>, batch_size: u64){
        let brc20_store = object::borrow_mut(brc20_store_obj);
        let inscription_ids = ord::inscription_ids(inscription_store_obj);
        let inscriptions = ord::inscriptions(inscription_store_obj);
        let start_inscription_index = brc20_store.next_inscription_index;
        let max_inscription_count = table_vec::length(inscription_ids);
        if(start_inscription_index >= max_inscription_count){
            return
        };
        let progressed_inscription_count = 0;
        let progress_inscription_index = start_inscription_index;
        while(progressed_inscription_count < batch_size && progress_inscription_index < max_inscription_count){
            let inscription_id = *table_vec::borrow(inscription_ids, progress_inscription_index);
            let inscription = table::borrow(inscriptions, inscription_id);
            let op_opt = from_inscription(inscription);
            if(option::is_some(&op_opt)){
                let op = option::destroy_some(op_opt);
                progress_op(brc20_store, op);
            };
            progressed_inscription_count = progressed_inscription_count + 1;
            progress_inscription_index = progress_inscription_index + 1;
        };
        brc20_store.next_inscription_index = progress_inscription_index;
    }

    #[test]
    fun test_deploy_op(){
        let deploy_op_json = b"{\"p\":\"brc-20\",\"op\":\"deploy\",\"tick\":\"ordi\",\"max\":\"21000000\",\"lim\":\"1000\"}";
        let op = Op { json_map: json::to_map(deploy_op_json) };
        assert!(is_brc20(&op), 1);
        assert!(is_deploy(&op), 2);
        let deploy_op_opt = as_deploy(&op);
        assert!(option::is_some(&deploy_op_opt), 3);
        let deploy_op = option::destroy_some(deploy_op_opt);
        assert!(deploy_op.tick == string::utf8(b"ordi"), 4);
        assert!(deploy_op.max == 21000000, 5);
        assert!(deploy_op.lim == 1000, 6);
        assert!(deploy_op.dec == 18, 7);
    }

    #[test]
    fun test_mint_op(){
        let mint_op_json = b"{\"p\":\"brc-20\",\"op\":\"mint\",\"tick\":\"ordi\",\"amt\":\"1000\"}";
        let op = Op { json_map: json::to_map(mint_op_json) };
        assert!(is_brc20(&op), 1);
        assert!(is_mint(&op), 2);
        let mint_op_opt = as_mint(&op);
        assert!(option::is_some(&mint_op_opt), 3);
        let mint_op = option::destroy_some(mint_op_opt);
        assert!(mint_op.tick == string::utf8(b"ordi"), 4);
        assert!(mint_op.amt == 1000, 5);
    }

    #[test]
    fun test_transfer_op(){
        let transfer_op_json = b"{\"p\":\"brc-20\",\"op\":\"transfer\",\"tick\":\"ordi\",\"amt\":\"1000\"}";
        let op = Op { json_map: json::to_map(transfer_op_json) };
        assert!(is_brc20(&op), 1);
        assert!(is_transfer(&op), 2);
        let transfer_op_opt = as_transfer(&op);
        assert!(option::is_some(&transfer_op_opt), 3);
        let transfer_op = option::destroy_some(transfer_op_opt);
        assert!(transfer_op.tick == string::utf8(b"ordi"), 4);
        assert!(transfer_op.amt == 1000, 5);
    }

}
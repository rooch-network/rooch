// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module bitcoin_move::bbn {

    use std::option;
    use std::option::{Option, is_none, is_some, none, some};
    use std::vector;
    use std::vector::{length, borrow};
    use moveos_std::object::{Self, Object};
    use moveos_std::type_info;
    use moveos_std::bcs;
    use bitcoin_move::bitcoin;
    use bitcoin_move::types;
    use bitcoin_move::utxo;
    use bitcoin_move::opcode;
    use bitcoin_move::script_buf::{Self, ScriptBuf};
    use bitcoin_move::types::{
        Transaction,
        tx_output,
        txout_value,
        tx_lock_time,
        txout_script_pubkey
    };
    use bitcoin_move::temp_state;
    use rooch_framework::bitcoin_address::{
        derive_bitcoin_taproot_address_from_pubkey,
        to_rooch_address
    };

    friend bitcoin_move::genesis;

    //https://github.com/babylonlabs-io/networks/blob/28651b301bb2efa0542b2268793948bcda472a56/parameters/parser/ParamsParser.go#L117
    struct BBNGlobalParam has copy, drop, store {
        version: u64,
        activation_height: u64,
        staking_cap: u64,
        cap_height: u64,
        tag: vector<u8>,
        covenant_pks: vector<vector<u8>>,
        covenant_quorum: u32,
        unbonding_time: u16,
        unbonding_fee: u64,
        max_staking_amount: u64,
        min_staking_amount: u64,
        min_staking_time: u16,
        max_staking_time: u16,
        confirmation_depth: u16
    }

    struct BBNGlobalParams has key {
        bbn_global_param: vector<BBNGlobalParam>
    }

    struct BBNOpReturnOutput has copy, store, drop {
        vout: u32,
        op_return_data: BBNOpReturnData
    }

    struct BBNOpReturnData has copy, store, drop {
        tag: vector<u8>,
        staker_pub_key: vector<u8>,
        finality_provider_pub_key: vector<u8>,
        staking_time: u16
    }

    struct BBNStakeSeal has key {
        /// The stake transaction block height
        block_height: u64,
        /// The stake transaction hash
        txid: address,
        /// The stake utxo output index
        vout: u32,
        tag: vector<u8>,
        staker_pub_key: vector<u8>,
        finality_provider_pub_key: vector<u8>,
        /// The stake time in block count
        staking_time: u16,
        /// The stake amount in satoshi
        staking_amount: u64,
    }

    const UNSPENDABLEKEYPATHKEY: vector<u8> = b"0250929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0";
    const TEMPORARY_AREA: vector<u8> = b"temporary_area";

    const ErrorAlreadyInit: u64 = 1;
    const ErrorNoBabylonUTXO: u64 = 2;
    const ErrorTransactionNotFound: u64 = 3;
    const ErrorNoBabylonOpReturn: u64 = 4;
    const ErrorInvalidBabylonOpReturn: u64 = 5;
    const ErrorTransactionLockTime: u64 = 6;
    const ErrorInvalidBytesLen: u64 = 7;
    const ErrorNotBabylonTx: u64 = 8;
    

    //https://github.com/babylonlabs-io/networks/blob/main/bbn-1/parameters/global-params.json
    // {
    //     "version": 1,
    //     "activation_height": 864790,
    //     "cap_height": 864799,
    //     "tag": "62626e31",
    //     "covenant_pks": [
    //         "03d45c70d28f169e1f0c7f4a78e2bc73497afe585b70aa897955989068f3350aaa",
    //         "034b15848e495a3a62283daaadb3f458a00859fe48e321f0121ebabbdd6698f9fa",
    //         "0223b29f89b45f4af41588dcaf0ca572ada32872a88224f311373917f1b37d08d1",
    //         "02d3c79b99ac4d265c2f97ac11e3232c07a598b020cf56c6f055472c893c0967ae",
    //         "038242640732773249312c47ca7bdb50ca79f15f2ecc32b9c83ceebba44fb74df7",
    //         "03e36200aaa8dce9453567bba108bdc51f7f1174b97a65e4dc4402fc5de779d41c",
    //         "03cbdd028cfe32c1c1f2d84bfec71e19f92df509bba7b8ad31ca6c1a134fe09204",
    //         "03f178fcce82f95c524b53b077e6180bd2d779a9057fdff4255a0af95af918cee0",
    //         "03de13fc96ea6899acbdc5db3afaa683f62fe35b60ff6eb723dad28a11d2b12f8c"
    //     ],
    //     "covenant_quorum": 6,
    //     "unbonding_time": 1008,
    //     "unbonding_fee": 32000,
    //     "max_staking_amount": 50000000000,
    //     "min_staking_amount": 500000,
    //     "max_staking_time": 64000,
    //     "min_staking_time": 64000,
    //     "confirmation_depth": 10
    // }
    public(friend) fun genesis_init() {
        // bbn-1 version 1
        let bbn_global_params_1 = BBNGlobalParam {
            version: 1,
            activation_height: 864790,
            staking_cap: 0,
            cap_height: 864799,
            //bbn1
            tag: x"62626e31",
            covenant_pks: vector[
                b"03d45c70d28f169e1f0c7f4a78e2bc73497afe585b70aa897955989068f3350aaa",
                b"034b15848e495a3a62283daaadb3f458a00859fe48e321f0121ebabbdd6698f9fa",
                b"0223b29f89b45f4af41588dcaf0ca572ada32872a88224f311373917f1b37d08d1",
                b"02d3c79b99ac4d265c2f97ac11e3232c07a598b020cf56c6f055472c893c0967ae",
                b"038242640732773249312c47ca7bdb50ca79f15f2ecc32b9c83ceebba44fb74df7",
                b"03e36200aaa8dce9453567bba108bdc51f7f1174b97a65e4dc4402fc5de779d41c",
                b"03cbdd028cfe32c1c1f2d84bfec71e19f92df509bba7b8ad31ca6c1a134fe09204",
                b"03f178fcce82f95c524b53b077e6180bd2d779a9057fdff4255a0af95af918cee0",
                b"03de13fc96ea6899acbdc5db3afaa683f62fe35b60ff6eb723dad28a11d2b12f8c"
            ],
            covenant_quorum: 6,
            unbonding_time: 1008,
            unbonding_fee: 32000,
            max_staking_amount: 50000000000,
            min_staking_amount: 500000,
            min_staking_time: 64000,
            max_staking_time: 64000,
            confirmation_depth: 10
        };
        let obj =
            object::new_named_object(
                BBNGlobalParams { bbn_global_param: vector[bbn_global_params_1] }
            );
        object::to_shared(obj);
    }

    public fun init_for_upgrade(){
        let object_id = object::named_object_id<BBNGlobalParams>();
        assert!(!object::exists_object(object_id), ErrorAlreadyInit);
        genesis_init()
    }

    fun new_bbn_stake_seal(
        block_height: u64, txid: address, vout: u32, tag: vector<u8>, staker_pub_key: vector<u8>,
        finality_provider_pub_key: vector<u8>, staking_time: u16, staking_amount: u64
    ): Object<BBNStakeSeal> {
        object::new(BBNStakeSeal {
            block_height: block_height,
            txid: txid,
            vout: vout,
            tag: tag,
            staker_pub_key: staker_pub_key,
            finality_provider_pub_key: finality_provider_pub_key,
            staking_time: staking_time,
            staking_amount: staking_amount
        })
    }

    fun get_bbn_param(block_height: u64): Option<BBNGlobalParam> {
        let object_id = object::named_object_id<BBNGlobalParams>();
        let params = object::borrow(object::borrow_object<BBNGlobalParams>(object_id));
        let i = 0;
        let len = length(&params.bbn_global_param);
        while (i < len) {
            let param = borrow(&params.bbn_global_param, i);
            i = i + 1;
            if (param.cap_height !=0 && block_height > param.cap_height) {
                continue
            };
            if (block_height < param.activation_height) {
                continue
            };
            return some(*param)
        };
        none()
    }

    fun try_get_bbn_op_return_output(transaction: Transaction): Option<BBNOpReturnOutput> {
        let tx_output = tx_output(&transaction);
        if (vector::length(tx_output) < 2) {
            return none()
        };

        let result = none();
        let index = 0;
        let output_len = length(tx_output);
        while (index < output_len) {
            let output = borrow(tx_output, index);
            let op_return_data_opt = parse_bbn_op_return_data(txout_script_pubkey(output));
            if (is_some(&op_return_data_opt)) {
                if (is_some(&result)) {
                    // this case should not happen as standard bitcoin node propagation rules
                    // disallow multiple op return outputs in a single transaction. However, miner could
                    // include multiple op return outputs in a single transaction. In such case, we should
                    // ignore the tx.
                    return none()
                } else {
                    result = some(BBNOpReturnOutput{
                        vout: (index as u32),
                        op_return_data: option::destroy_some(op_return_data_opt)
                    });
                }
            };
            index = index + 1;
        };
        result
    }

    fun try_get_bbn_op_return_output_from_tx_bytes(bytes: vector<u8>): Option<BBNOpReturnOutput> {
        let transaction = bcs::from_bytes<Transaction>(bytes);
        try_get_bbn_op_return_output(transaction)
    }

    public fun is_bbn_tx(txid: address): bool {
        let block_height_opt = bitcoin::get_tx_height(txid);
        if (is_none(&block_height_opt)) {
            return false
        };
        let block_height = option::destroy_some(block_height_opt);
        let param_opt = get_bbn_param(block_height);
        if (is_none(&param_opt)) {
            return false
        };
        let param = option::destroy_some(param_opt);
        let tx_opt = bitcoin::get_tx(txid);
        if (is_none(&tx_opt)) {
            return false
        };
        let transaction = option::destroy_some(tx_opt);
        let output_opt = try_get_bbn_op_return_output(transaction);
        if (is_none(&output_opt)) {
            return false
        };
        let output = option::destroy_some(output_opt);
        validate_bbn_op_return_data(&param, &transaction, &output.op_return_data)
    }

    public entry fun process_bbn_tx_entry(txid: address){
        process_bbn_tx(txid)
    }

    fun validate_bbn_op_return_data(param: &BBNGlobalParam, tx: &Transaction, op_return_data: &BBNOpReturnData): bool {
        if (op_return_data.tag != param.tag) {
            return false
        };
        if (op_return_data.staking_time < param.min_staking_time
            || op_return_data.staking_time > param.max_staking_time) {
            return false
        };
        if (!vector::contains(&param.covenant_pks, &op_return_data.finality_provider_pub_key)) {
            return false
        };
        if (tx_lock_time(tx) < (op_return_data.staking_time as u32)) {
            return false
        };
        true
    }

    fun process_bbn_tx(txid: address) {
        let block_height_opt = bitcoin::get_tx_height(txid);
        assert!(is_some(&block_height_opt), ErrorTransactionNotFound);
        let block_height = option::destroy_some(block_height_opt);

        let param_opt = get_bbn_param(block_height);
        assert!(is_some(&param_opt), ErrorNotBabylonTx);
        let param = option::destroy_some(param_opt);

        let tx_opt = bitcoin::get_tx(txid);
        assert!(is_some(&tx_opt), ErrorTransactionNotFound);
        
        let transaction = option::destroy_some(tx_opt);

        let op_return_output_opt = try_get_bbn_op_return_output(transaction);
        assert!(is_some(&op_return_output_opt), ErrorNoBabylonOpReturn);
        let op_return_output = option::destroy_some(op_return_output_opt);
        
        let BBNOpReturnOutput{vout: op_return_vout, op_return_data} = op_return_output;

        let valid = validate_bbn_op_return_data(&param, &transaction, &op_return_data);
        
        assert!(valid, ErrorInvalidBabylonOpReturn);

        let seal_protocol = type_info::type_name<BBNStakeSeal>();
        let tx_outputs = tx_output(&transaction);
        let index = 0;
        let has_stake_seal = false;
        //bbn should not multiple staking outputs yet, we support it for in case
        while (index < length(tx_outputs)) {
            let tx_output = borrow(tx_outputs, index);
            let vout = (index as u32);
            index = index + 1;
            if (vout == op_return_vout) {
                continue
            };
            //TODO skip the change output
            let txout_value = txout_value(tx_output);
            let out_point = types::new_outpoint(txid, vout);
            let utxo_obj = utxo::borrow_mut_utxo(out_point);
            let utxo = object::borrow_mut(utxo_obj);
            if (utxo::has_seal_internal(utxo, &seal_protocol)){
                continue
            };
            let bbn_stake_seal_obj = new_bbn_stake_seal(
                block_height, txid, vout, op_return_data.tag, 
                op_return_data.staker_pub_key, op_return_data.finality_provider_pub_key,
                op_return_data.staking_time, txout_value
            );
            let seal_object_id = object::id(&bbn_stake_seal_obj);
            let staker_address = pubkey_to_rooch_address(&op_return_data.staker_pub_key);
            object::transfer_extend(bbn_stake_seal_obj, staker_address);
            
            let seal = utxo::new_utxo_seal(seal_protocol, seal_object_id);
            utxo::add_seal_internal(utxo, seal);
            has_stake_seal = true;
        };
        assert!(has_stake_seal, ErrorNoBabylonUTXO);
    }

    fun pubkey_to_rooch_address(pubkey: &vector<u8>): address {
        to_rooch_address(&derive_bitcoin_taproot_address_from_pubkey(pubkey))
    }

    fun parse_bbn_op_return_data(script_buf: &ScriptBuf): Option<BBNOpReturnData>{
        // 1. OP_RETURN opcode - which signalizes that data is provably unspendable
	    // 2. OP_DATA_71 opcode - which pushes 71 bytes of data to the stack
        let script_bytes = script_buf::bytes(script_buf);
        let script_len = vector::length(script_bytes);
        if (script_len != 73 || *vector::borrow(script_bytes, 0) != opcode::op_return() || *vector::borrow(script_bytes, 1) != opcode::op_pushbytes_71()){
            return none() 
        };
        let tag = vector::slice(script_bytes, 2, 6);
        let version = *vector::borrow(script_bytes, 6);
        if (version != 0u8){
            return none()
        };
        let staker_pub_key = vector::slice(script_bytes, 7, 39);
        let finality_provider_pub_key = vector::slice(script_bytes, 39, 71);
        let staking_time = bytes_to_u16(vector::slice(script_bytes, 71, 73));
        some(BBNOpReturnData{
            tag,
            staker_pub_key: staker_pub_key,
            finality_provider_pub_key: finality_provider_pub_key,
            staking_time: staking_time
        })
    }

    //TODO migrate to `std::u16`
    fun bytes_to_u16(bytes: vector<u8>): u16 {
        assert!(vector::length(&bytes) == 2, ErrorInvalidBytesLen);
        let high_byte = vector::borrow(&bytes, 0);
        let low_byte = vector::borrow(&bytes, 1);
        ((*high_byte as u16) << 8) | (*low_byte as u16)
    }

    // ==== Temporary Area ===

    #[private_generics(S)]
    public fun add_temp_state<S: store + drop>(stake: &mut Object<BBNStakeSeal>, state: S){
        if(object::contains_field(stake, TEMPORARY_AREA)){
            let temp_state = object::borrow_mut_field(stake, TEMPORARY_AREA);
            temp_state::add_state(temp_state, state);
        }else{
            let temp_state = temp_state::new();
            temp_state::add_state(&mut temp_state, state);
            object::add_field(stake, TEMPORARY_AREA, temp_state);
        }
    }

    public fun contains_temp_state<S: store + drop>(stake: &Object<BBNStakeSeal>) : bool {
        if(object::contains_field(stake, TEMPORARY_AREA)){
            let temp_state = object::borrow_field(stake, TEMPORARY_AREA);
            temp_state::contains_state<S>(temp_state)
        }else{
            false
        }
    }

    public fun borrow_temp_state<S: store + drop>(stake: &Object<BBNStakeSeal>) : &S {
        let temp_state = object::borrow_field(stake, TEMPORARY_AREA);
        temp_state::borrow_state(temp_state)
    }

    #[private_generics(S)]
    public fun borrow_mut_temp_state<S: store + drop>(stake: &mut Object<BBNStakeSeal>) : &mut S {
        let temp_state = object::borrow_mut_field(stake, TEMPORARY_AREA);
        temp_state::borrow_mut_state(temp_state)
    }

    #[private_generics(S)]
    public fun remove_temp_state<S: store + drop>(stake: &mut Object<BBNStakeSeal>) : S {
        let temp_state = object::borrow_mut_field(stake, TEMPORARY_AREA);
        temp_state::remove_state(temp_state)
    }

    // ============== BBNStakeSeal ==============

    public fun block_height(stake: &BBNStakeSeal): u64 {
        stake.block_height
    }

    public fun txid(stake: &BBNStakeSeal): address {
        stake.txid
    }

    public fun vout(stake: &BBNStakeSeal): u32 {
        stake.vout
    }

    public fun outpoint(stake: &BBNStakeSeal): types::OutPoint {
        types::new_outpoint(stake.txid, stake.vout)
    }

    public fun tag(stake: &BBNStakeSeal): &vector<u8> {
        &stake.tag
    }

    public fun staker_pub_key(stake: &BBNStakeSeal): &vector<u8> {
        &stake.staker_pub_key
    }

    public fun finality_provider_pub_key(stake: &BBNStakeSeal): &vector<u8> {
        &stake.finality_provider_pub_key
    }

    public fun staking_time(stake: &BBNStakeSeal): u16 {
        stake.staking_time
    }

    public fun staking_amount(stake: &BBNStakeSeal): u64 {
        stake.staking_amount
    }

    public fun is_expired(stake: &BBNStakeSeal): bool {
        let latest_block_opt = bitcoin::get_latest_block();
        if (is_none(&latest_block_opt)) {
            return false
        };
        let latest_block = option::destroy_some(latest_block_opt);
        let (current_block_height, _hash) = types::unpack_block_height_hash(latest_block);
        current_block_height > (stake.block_height + (stake.staking_time as u64))
    }


    #[test]
    fun test_parse_bbn_op_return_data(){
        //https://mempool.space/tx/7d90210b21aad480cd88fd8399aa6d47e6b3f2ecea2f9f9cfdd79598430e3003
        let script_buf = script_buf::new(x"6a4762626e31000b93d2d388a2b89c2ba2ef28e99c0cfc20735b693cb8b2350fe8aceca2d0f393db9160428e401753dc1a9952ffd4fa3386c7609cf8411d2b6d79c42323ca9923fa00");
        let bbn_opreturn_data_opt = parse_bbn_op_return_data(&script_buf);
        assert!(is_some(&bbn_opreturn_data_opt), 1000);
        let bbn_opreturn_data = option::destroy_some(bbn_opreturn_data_opt);
        let BBNOpReturnData{tag, staker_pub_key, finality_provider_pub_key, staking_time} = bbn_opreturn_data;
        assert!(tag == x"62626e31", 1001);
        assert!(tag == b"bbn1", 1002);
        assert!(vector::length(&staker_pub_key) == 32, 1003);
        assert!(vector::length(&finality_provider_pub_key) == 32, 1004);
        assert!(staking_time == 64000, 1005);
    }

}

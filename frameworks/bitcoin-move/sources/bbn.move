// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module bitcoin_move::bbn {

    use std::option;
    use std::option::{is_none, is_some};
    use std::vector;
    use std::vector::{length, borrow};
    use moveos_std::object::{Self, Object};
    use moveos_std::type_info;
    use bitcoin_move::bitcoin;
    use bitcoin_move::types;
    use bitcoin_move::utxo;
    use bitcoin_move::opcode;
    use bitcoin_move::script_buf::{Self, ScriptBuf};
    use bitcoin_move::types::{
        Transaction,
        tx_id,
        tx_output,
        txout_value,
        tx_lock_time,
        txout_script_pubkey
    };
    use bitcoin_move::bitcoin::get_tx_height;
    use bitcoin_move::temp_state;
    use rooch_framework::bitcoin_address::{
        derive_bitcoin_taproot_address_from_pubkey,
        to_rooch_address
    };

    friend bitcoin_move::genesis;

    struct BBNGlobalParam has key, store {
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

    struct BBNOpReturnData has copy, store, drop {
        tag: vector<u8>,
        version: u64,
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
        version: u64,
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
    const ErrorNotBabylonUTXO: u64 = 2;
    const ErrorTransactionNotFound: u64 = 3;
    const ErrorNotBabylonOpReturn: u64 = 4;
    const ErrorTransactionLockTime: u64 = 5;
    const ErrorInvalidBytesLen: u64 = 6;

    public(friend) fun genesis_init() {
        // TODO here just add bbn test-4 version 2
        let bbn_global_params_2 = BBNGlobalParam {
            version: 2,
            activation_height: 200665,
            staking_cap: 0,
            cap_height: 201385,
            tag: b"62627434",
            covenant_pks: vector[
                b"03fa9d882d45f4060bdb8042183828cd87544f1ea997380e586cab77d5fd698737",
                b"020aee0509b16db71c999238a4827db945526859b13c95487ab46725357c9a9f25",
                b"0217921cf156ccb4e73d428f996ed11b245313e37e27c978ac4d2cc21eca4672e4",
                b"02113c3a32a9d320b72190a04a020a0db3976ef36972673258e9a38a364f3dc3b0",
                b"0379a71ffd71c503ef2e2f91bccfc8fcda7946f4653cef0d9f3dde20795ef3b9f0",
                b"023bb93dfc8b61887d771f3630e9a63e97cbafcfcc78556a474df83a31a0ef899c",
                b"03d21faf78c6751a0d38e6bd8028b907ff07e9a869a43fc837d6b3f8dff6119a36",
                b"0340afaf47c4ffa56de86410d8e47baa2bb6f04b604f4ea24323737ddc3fe092df",
                b"03f5199efae3f28bb82476163a7e458c7ad445d9bffb0682d10d3bdb2cb41f8e8e"
            ],
            covenant_quorum: 6,
            unbonding_time: 1008,
            unbonding_fee: 10000,
            max_staking_amount: 5000000,
            min_staking_amount: 50000,
            min_staking_time: 64000,
            max_staking_time: 64000,
            confirmation_depth: 10
        };
        let obj =
            object::new_named_object(
                BBNGlobalParams { bbn_global_param: vector[bbn_global_params_2] }
            );
        object::to_shared(obj);
    }

    public fun init_for_upgrade(){
        let object_id = object::named_object_id<BBNGlobalParams>();
        assert!(!object::exists_object(object_id), ErrorAlreadyInit);
        genesis_init()
    }

    fun new_bbn_stake_seal(
        block_height: u64, txid: address, vout: u32, tag: vector<u8>, version: u64, staker_pub_key: vector<u8>,
        finality_provider_pub_key: vector<u8>, staking_time: u16, staking_amount: u64
    ): Object<BBNStakeSeal> {
        object::new(BBNStakeSeal {
            block_height: block_height,
            txid: txid,
            vout: vout,
            tag: tag,
            version: version,
            staker_pub_key: staker_pub_key,
            finality_provider_pub_key: finality_provider_pub_key,
            staking_time: staking_time,
            staking_amount: staking_amount
        })
    }

    fun borrow_bbn_params(): &Object<BBNGlobalParams> {
        let object_id = object::named_object_id<BBNGlobalParams>();
        object::borrow_object(object_id)
    }

    fun borrow_bbn_params_mut(): &mut Object<BBNGlobalParams> {
        let object_id = object::named_object_id<BBNGlobalParams>();
        object::borrow_mut_object_shared(object_id)
    }

    fun try_get_bbn_op_return_data(transaction: Transaction):
        (bool, u64, BBNOpReturnData) {
        let bbn_op_return_data = BBNOpReturnData {
            tag: vector[],
            version: 0,
            staker_pub_key: vector[],
            finality_provider_pub_key: vector[],
            staking_time: 0
        };
        let tx_output = tx_output(&transaction);
        if (vector::length(tx_output) < 2) {
            return (false, 0, bbn_op_return_data)
        };

        // this case should not happen as standard bitcoin node propagation rules
        // disallow multiple op return outputs in a single transaction. However, miner could
        // include multiple op return outputs in a single transaction. In such case, we should
        // return an error.
        let index = 0;
        let i = 0;
        while (i < length(tx_output)) {
            let output = borrow(tx_output, i);
            let (tag, version, staker_pub_key, finality_provider_pub_key, staking_time) =
                unpack_bbn_stake_data(txout_script_pubkey(output));
            bbn_op_return_data.tag = tag;
            bbn_op_return_data.version = version;
            bbn_op_return_data.staker_pub_key = staker_pub_key;
            bbn_op_return_data.finality_provider_pub_key = finality_provider_pub_key;
            bbn_op_return_data.staking_time = staking_time;
            if (vector::length(&bbn_op_return_data.tag) != 0) { break };
            index = index + 1;
            i = i + 1;
        };
        if (vector::length(&bbn_op_return_data.tag) == 0) {
            return (false, 0, bbn_op_return_data)
        };
        let option_tx_height = get_tx_height(tx_id(&transaction));
        if (is_none(&option_tx_height)) {
            return (false, 0, bbn_op_return_data)
        };
        let tx_height = option::destroy_some(option_tx_height);
        let bbn_params = object::borrow(borrow_bbn_params());
        let i = 0;
        while (i < length(&bbn_params.bbn_global_param)) {
            let param = borrow(&bbn_params.bbn_global_param, i);
            i = i + 1;
            if (bbn_op_return_data.version != param.version
                || bbn_op_return_data.tag != param.tag
                || tx_height < param.activation_height
                || param.covenant_quorum > (length(&param.covenant_pks) as u32)) {
                continue
            };
            if (param.cap_height != 0
                && tx_height > (param.activation_height + param.cap_height)) {
                continue
            };
            if (bbn_op_return_data.staking_time < param.min_staking_time
                || bbn_op_return_data.staking_time > param.max_staking_time) {
                continue
            };
            if (!vector::contains(
                &param.covenant_pks, &bbn_op_return_data.finality_provider_pub_key
            )) {
                continue
            };
            return (true, index, bbn_op_return_data)
        };
        return (false, 0, bbn_op_return_data)
    }

    fun try_get_bbn_op_return_data_from_tx_bytes(bytes: vector<u8>): (bool, u64, BBNOpReturnData) {
        let transaction = bcs::from_bytes<Transaction>(&bytes).expect("should be a valid transaction");
        try_get_bbn_op_return_data(transaction)
    }

    public fun is_bbn_tx(txid: address): bool {
        let tx_opt = bitcoin::get_tx(txid);
        if (is_none(&tx_opt)) {
            return false
        };
        let transaction = option::destroy_some(tx_opt);
        let (is_true, _, _) = try_get_bbn_op_return_data(transaction);
        is_true
    }

    public entry fun process_bbn_tx_entry(txid: address){
        process_bbn_tx(txid)
    }

    fun process_bbn_tx(txid: address) {
        let block_height_opt = bitcoin::get_tx_height(txid);
        assert!(is_some(&block_height_opt), ErrorTransactionNotFound);
        let block_height = option::destroy_some(block_height_opt);
        let tx_opt = bitcoin::get_tx(txid);
        assert!(is_some(&tx_opt), ErrorTransactionNotFound);
        let transaction = option::destroy_some(tx_opt);

        let (is_true, op_return_index, op_return_data) =
            try_get_bbn_op_return_data(transaction);
        
        assert!(is_true, ErrorNotBabylonOpReturn);
        
        assert!(tx_lock_time(&transaction) >= (op_return_data.staking_time as u32), ErrorTransactionLockTime);

        let seal_protocol = type_info::type_name<BBNStakeSeal>();
        let tx_outputs = tx_output(&transaction);
        let index = 0;
        let has_stake_seal = false;
        //bbn should not multiple staking outputs yet, we support it for in case
        while (index < length(tx_outputs)) {
            let tx_output = borrow(tx_outputs, index);
            if (index == op_return_index) {
                continue
            };
            let vout = (index as u32);
            let txout_value = txout_value(tx_output);
            let out_point = types::new_outpoint(txid, vout);
            let utxo_obj = utxo::borrow_mut_utxo(out_point);
            let utxo = object::borrow_mut(utxo_obj);
            if (utxo::has_seal_internal(utxo, &seal_protocol)){
                continue
            };
            let bbn_stake_seal_obj = new_bbn_stake_seal(
                block_height, txid, vout, op_return_data.tag, op_return_data.version,
                op_return_data.staker_pub_key, op_return_data.finality_provider_pub_key,
                op_return_data.staking_time, txout_value
            );
            let seal_object_id = object::id(&bbn_stake_seal_obj);
            let staker_address = pubkey_to_rooch_address(&op_return_data.staker_pub_key);
            object::transfer_extend(bbn_stake_seal_obj, staker_address);
            
            let seal = utxo::new_utxo_seal(seal_protocol, seal_object_id);
            utxo::add_seal_internal(utxo, seal);
            has_stake_seal = true;
            index = index + 1;
        };
        assert!(has_stake_seal, ErrorNotBabylonUTXO);
    }

    fun pubkey_to_rooch_address(pubkey: &vector<u8>): address {
        to_rooch_address(&derive_bitcoin_taproot_address_from_pubkey(pubkey))
    }

    fun unpack_bbn_stake_data(script_buf: &ScriptBuf): (vector<u8>, u64, vector<u8>, vector<u8>, u16){
        let script_bytes = script_buf::bytes(script_buf);
        // 1. OP_RETURN opcode - which signalizes that data is provably unspendable
	    // 2. OP_DATA_71 opcode - which pushes 71 bytes of data to the stack
        if (vector::length(script_bytes) != 73 || *vector::borrow(script_bytes, 0) != opcode::op_return() || *vector::borrow(script_bytes, 1) != opcode::op_pushbytes_71()){
            return (vector[], 0, vector[], vector[], 0)
        };
        let tag = vector::slice(script_bytes, 2, 6);
        let version = bytes_to_u64(vector::slice(script_bytes, 6, 7));
        let staker_pub_key = vector::slice(script_bytes, 7, 39);
        let finality_provider_pub_key = vector::slice(script_bytes, 39, 71);
        let staking_time = bytes_to_u16(vector::slice(script_bytes, 71, 73));
        return (tag, version, staker_pub_key, finality_provider_pub_key, staking_time)
    }

    //TODO migrate to `std::u16`
    fun bytes_to_u16(bytes: vector<u8>): u16 {
        assert!(vector::length(&bytes) == 2, ErrorInvalidBytesLen);
        let high_byte = vector::borrow(&bytes, 0);
        let low_byte = vector::borrow(&bytes, 1);
        ((*high_byte as u16) << 8) | (*low_byte as u16)
    }

    //TODO migrate to `std::u64`
    fun bytes_to_u64(bytes: vector<u8>): u64 {
        let value = 0u64;
        let i = 0u64;
        while (i < 8) {
            value = value | ((*vector::borrow(&bytes, i) as u64) << ((8 * (7 - i)) as u8));
            i = i + 1;
        };
        return value
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

    public fun version(stake: &BBNStakeSeal): u64 {
        stake.version
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

}

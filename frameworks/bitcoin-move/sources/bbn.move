// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module bitcoin_move::bbn {

    use std::option;
    use std::option::{Option, is_none, is_some, none, some};
    use std::vector;
    use std::vector::{length, borrow};
    use std::string::String;
    use moveos_std::object::{Self, Object, ObjectID};
    use moveos_std::type_info;
    use moveos_std::bcs;
    use moveos_std::result::{Self, Result, err_str, ok, is_err, as_err};
    use moveos_std::sort;
    use moveos_std::event;
    use moveos_std::event_queue;
    use bitcoin_move::types;
    use bitcoin_move::utxo::{Self, UTXO};
    use bitcoin_move::opcode;
    use bitcoin_move::script_buf::{Self, ScriptBuf};
    use bitcoin_move::types::{
        Transaction,
        txout_value,
        txout_script_pubkey,
        TxOut
    };
    use bitcoin_move::temp_state;
    use bitcoin_move::taproot_builder::{Self, TaprootBuilder};
    use rooch_framework::bitcoin_address::{
        Self,
        derive_bitcoin_taproot_address_from_pubkey,
        to_rooch_address
    };

    friend bitcoin_move::genesis;
    friend bitcoin_move::bitcoin;
    friend bitcoin_move::bbn_updater;

    const UNSPENDABLEKEYPATHKEY: vector<u8> = x"50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0";
    const TEMPORARY_AREA: vector<u8> = b"temporary_area";

    const ErrorAlreadyInit: u64 = 1;
    const ErrorNoBabylonUTXO: u64 = 2;
    const ErrorTransactionNotFound: u64 = 3;
    const ErrorNoBabylonOpReturn: u64 = 4;
    const ErrorInvalidBabylonOpReturn: u64 = 5;
    const ErrorTransactionLockTime: u64 = 6;
    const ErrorInvalidBytesLen: u64 = 7;
    const ErrorNotBabylonTx: u64 = 8;
    const ErrorNoKeysProvided: u64 = 9;
    const ErrorInvalidKeysLen: u64 = 10;
    const ErrorInvalidThreshold: u64 = 11;
    const ErrorFailedToFinalizeTaproot: u64 = 12;
    const ErrorUTXOAlreadySealed: u64 = 13;
    const ErrorNoBabylonStakingOutput: u64 = 14;
    const ErrorOutBlockRange: u64 = 15;
    const DeprecatedFunction: u64 = 16;

    //https://github.com/babylonlabs-io/networks/blob/28651b301bb2efa0542b2268793948bcda472a56/parameters/parser/ParamsParser.go#L117
    struct BBNGlobalParamV0 has copy, drop, store {
        version: u64,
        activation_height: u64,
        staking_cap: u64,
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

    struct BBNGlobalParamV1 has copy, drop, store {
        version: u64,
        activation_height: u64,
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
        max_version: u64,
    }

    struct BBNOpReturnOutput has copy, store, drop {
        op_return_output_idx: u32,
        op_return_data: BBNV0OpReturnData
    } 

    struct BBNV0OpReturnData has copy, store, drop {
        tag: vector<u8>,
        version: u8,
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
        staking_output_index: u32,
        tag: vector<u8>,
        staker_pub_key: vector<u8>,
        finality_provider_pub_key: vector<u8>,
        /// The stake time in block count
        staking_time: u16,
        /// The stake amount in satoshi
        staking_value: u64,
    }

    struct BBNScriptPaths has store, copy, drop {
        time_lock_path_script: ScriptBuf,
        unbonding_path_script: ScriptBuf,
        slashing_path_script: ScriptBuf,
    }

    struct BBNStakingEvent has store, copy, drop{
        block_height: u64,
        txid: address,
        /// BBNStakeSeal object id
        stake_object_id: ObjectID,
    }

    struct BBNStakingFailedEvent has store, copy, drop{
        block_height: u64,
        txid: address,
        error: String,
    }

    struct BBNStakingUnbondingEvent has store, copy, drop{
        block_height: u64,
        txid: address,
        staking_output_index: u32,
        staking_time: u16,
        staking_value: u64,
        stake_object_id: ObjectID,
    }

    /// Event emitted when the temporary state of a BBNStakeSeal is dropped
    /// The temporary state is dropped when the UTXO is spent
    /// The event is onchain event, and the event_queue name is type_name of the temporary state
    struct TempStateDropEvent has drop, store, copy {
        stake_object_id: ObjectID,
        staking_time: u16,
        staking_value: u64,
    } 

    const BBN_V1_ACTIVATION_HEIGHT: u64 = 864790;
    const BBN_V1_CAP_HEIGHT: u64 = 864799;
    const BBN_V2_ACTIVATION_HEIGHT: u64 = 874088;
    const BBN_V2_CAP_HEIGHT: u64 = 875087;

    //https://github.com/babylonlabs-io/networks/blob/main/bbn-1/parameters/global-params.json
    // {
    //     "versions": [
    //         {
    //             "version": 0,
    //             "activation_height": 857910,
    //             "staking_cap": 100000000000,
    //             "tag": "62626e31",
    //             "covenant_pks": [
    //                 "03d45c70d28f169e1f0c7f4a78e2bc73497afe585b70aa897955989068f3350aaa",
    //                 "034b15848e495a3a62283daaadb3f458a00859fe48e321f0121ebabbdd6698f9fa",
    //                 "0223b29f89b45f4af41588dcaf0ca572ada32872a88224f311373917f1b37d08d1",
    //                 "02d3c79b99ac4d265c2f97ac11e3232c07a598b020cf56c6f055472c893c0967ae",
    //                 "038242640732773249312c47ca7bdb50ca79f15f2ecc32b9c83ceebba44fb74df7",
    //                 "03e36200aaa8dce9453567bba108bdc51f7f1174b97a65e4dc4402fc5de779d41c",
    //                 "03cbdd028cfe32c1c1f2d84bfec71e19f92df509bba7b8ad31ca6c1a134fe09204",
    //                 "03f178fcce82f95c524b53b077e6180bd2d779a9057fdff4255a0af95af918cee0",
    //                 "03de13fc96ea6899acbdc5db3afaa683f62fe35b60ff6eb723dad28a11d2b12f8c"
    //             ],
    //             "covenant_quorum": 6,
    //             "unbonding_time": 1008,
    //             "unbonding_fee": 64000,
    //             "max_staking_amount": 5000000,
    //             "min_staking_amount": 500000,
    //             "max_staking_time": 64000,
    //             "min_staking_time": 64000,
    //             "confirmation_depth": 10
    //         },
    //         {
    //             "version": 1,
    //             "activation_height": 864790,
    //             "cap_height": 864799,
    //             "tag": "62626e31",
    //             "covenant_pks": [
    //                 "03d45c70d28f169e1f0c7f4a78e2bc73497afe585b70aa897955989068f3350aaa",
    //                 "034b15848e495a3a62283daaadb3f458a00859fe48e321f0121ebabbdd6698f9fa",
    //                 "0223b29f89b45f4af41588dcaf0ca572ada32872a88224f311373917f1b37d08d1",
    //                 "02d3c79b99ac4d265c2f97ac11e3232c07a598b020cf56c6f055472c893c0967ae",
    //                 "038242640732773249312c47ca7bdb50ca79f15f2ecc32b9c83ceebba44fb74df7",
    //                 "03e36200aaa8dce9453567bba108bdc51f7f1174b97a65e4dc4402fc5de779d41c",
    //                 "03cbdd028cfe32c1c1f2d84bfec71e19f92df509bba7b8ad31ca6c1a134fe09204",
    //                 "03f178fcce82f95c524b53b077e6180bd2d779a9057fdff4255a0af95af918cee0",
    //                 "03de13fc96ea6899acbdc5db3afaa683f62fe35b60ff6eb723dad28a11d2b12f8c"
    //             ],
    //             "covenant_quorum": 6,
    //             "unbonding_time": 1008,
    //             "unbonding_fee": 32000,
    //             "max_staking_amount": 50000000000,
    //             "min_staking_amount": 500000,
    //             "max_staking_time": 64000,
    //             "min_staking_time": 64000,
    //             "confirmation_depth": 10
    //         },
    //         {
    //             "version": 2,
    //             "activation_height": 874088,
    //             "cap_height": 875087,
    //             "tag": "62626e31",
    //             "covenant_pks": [
    //                 "03d45c70d28f169e1f0c7f4a78e2bc73497afe585b70aa897955989068f3350aaa",
    //                 "034b15848e495a3a62283daaadb3f458a00859fe48e321f0121ebabbdd6698f9fa",
    //                 "0223b29f89b45f4af41588dcaf0ca572ada32872a88224f311373917f1b37d08d1",
    //                 "02d3c79b99ac4d265c2f97ac11e3232c07a598b020cf56c6f055472c893c0967ae",
    //                 "038242640732773249312c47ca7bdb50ca79f15f2ecc32b9c83ceebba44fb74df7",
    //                 "03e36200aaa8dce9453567bba108bdc51f7f1174b97a65e4dc4402fc5de779d41c",
    //                 "03cbdd028cfe32c1c1f2d84bfec71e19f92df509bba7b8ad31ca6c1a134fe09204",
    //                 "03f178fcce82f95c524b53b077e6180bd2d779a9057fdff4255a0af95af918cee0",
    //                 "03de13fc96ea6899acbdc5db3afaa683f62fe35b60ff6eb723dad28a11d2b12f8c"
    //             ],
    //             "covenant_quorum": 6,
    //             "unbonding_time": 1008,
    //             "unbonding_fee": 32000,
    //             "max_staking_amount": 500000000000,
    //             "min_staking_amount": 500000,
    //             "max_staking_time": 64000,
    //             "min_staking_time": 64000,
    //             "confirmation_depth": 10
    //         }
    //     ]
    // }
    public(friend) fun genesis_init() {
        //bbn-1 version 0
        let bbn_global_param_0 = BBNGlobalParamV0 {
            version: 0,
            activation_height: 857910,
            staking_cap: 100000000000,
            //bbn1
            tag: x"62626e31",
            //we keep the x-only pubkey in the vector
            covenant_pks: vector[
                x"03d45c70d28f169e1f0c7f4a78e2bc73497afe585b70aa897955989068f3350aaa",
                x"034b15848e495a3a62283daaadb3f458a00859fe48e321f0121ebabbdd6698f9fa",
                x"0223b29f89b45f4af41588dcaf0ca572ada32872a88224f311373917f1b37d08d1",
                x"02d3c79b99ac4d265c2f97ac11e3232c07a598b020cf56c6f055472c893c0967ae",
                x"038242640732773249312c47ca7bdb50ca79f15f2ecc32b9c83ceebba44fb74df7",
                x"03e36200aaa8dce9453567bba108bdc51f7f1174b97a65e4dc4402fc5de779d41c",
                x"03cbdd028cfe32c1c1f2d84bfec71e19f92df509bba7b8ad31ca6c1a134fe09204",
                x"03f178fcce82f95c524b53b077e6180bd2d779a9057fdff4255a0af95af918cee0",
                x"03de13fc96ea6899acbdc5db3afaa683f62fe35b60ff6eb723dad28a11d2b12f8c"
            ],
            covenant_quorum: 6,
            unbonding_time: 1008,
            unbonding_fee: 64000,
            max_staking_amount: 5000000,
            min_staking_amount: 500000,
            min_staking_time: 64000,
            max_staking_time: 64000,
            confirmation_depth: 10
        };
        // bbn-1 version 1
        let bbn_global_params_1 = BBNGlobalParamV1 {
            version: 1,
            activation_height: 864790,
            cap_height: 864799,
            //bbn1
            tag: x"62626e31",
            //we keep the x-only pubkey in the vector
            covenant_pks: vector[
                x"d45c70d28f169e1f0c7f4a78e2bc73497afe585b70aa897955989068f3350aaa",
                x"4b15848e495a3a62283daaadb3f458a00859fe48e321f0121ebabbdd6698f9fa",
                x"23b29f89b45f4af41588dcaf0ca572ada32872a88224f311373917f1b37d08d1",
                x"d3c79b99ac4d265c2f97ac11e3232c07a598b020cf56c6f055472c893c0967ae",
                x"8242640732773249312c47ca7bdb50ca79f15f2ecc32b9c83ceebba44fb74df7",
                x"e36200aaa8dce9453567bba108bdc51f7f1174b97a65e4dc4402fc5de779d41c",
                x"cbdd028cfe32c1c1f2d84bfec71e19f92df509bba7b8ad31ca6c1a134fe09204",
                x"f178fcce82f95c524b53b077e6180bd2d779a9057fdff4255a0af95af918cee0",
                x"de13fc96ea6899acbdc5db3afaa683f62fe35b60ff6eb723dad28a11d2b12f8c"
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
            object::new_named_object(BBNGlobalParams {
                max_version: 1,
        });
        object::add_field(&mut obj, 0, bbn_global_param_0);
        object::add_field(&mut obj, 1, bbn_global_params_1);
        object::to_shared(obj);
        init_bbn_global_param_v2();
    }

    public fun init_for_upgrade(){
        let object_id = object::named_object_id<BBNGlobalParams>();
        assert!(!object::exists_object(object_id), ErrorAlreadyInit);
        genesis_init()
    }

    /// BBN global param version 2 initialization
    entry fun init_bbn_global_param_v2(){
        let version = 2;
        let object_id = object::named_object_id<BBNGlobalParams>();
        let param_obj = object::borrow_mut_object_extend<BBNGlobalParams>(object_id);
        assert!(!object::contains_field(param_obj, version), ErrorAlreadyInit);

        // bb-1 version 2
        let bbn_global_params_2 = BBNGlobalParamV1 {
            version: 2,
            activation_height: 874088,
            cap_height: 875087,
            //bbn1
            tag: x"62626e31",
            //we keep the x-only pubkey in the vector
            covenant_pks: vector[
                x"d45c70d28f169e1f0c7f4a78e2bc73497afe585b70aa897955989068f3350aaa",
                x"4b15848e495a3a62283daaadb3f458a00859fe48e321f0121ebabbdd6698f9fa",
                x"23b29f89b45f4af41588dcaf0ca572ada32872a88224f311373917f1b37d08d1",
                x"d3c79b99ac4d265c2f97ac11e3232c07a598b020cf56c6f055472c893c0967ae",
                x"8242640732773249312c47ca7bdb50ca79f15f2ecc32b9c83ceebba44fb74df7",
                x"e36200aaa8dce9453567bba108bdc51f7f1174b97a65e4dc4402fc5de779d41c",
                x"cbdd028cfe32c1c1f2d84bfec71e19f92df509bba7b8ad31ca6c1a134fe09204",
                x"f178fcce82f95c524b53b077e6180bd2d779a9057fdff4255a0af95af918cee0",
                x"de13fc96ea6899acbdc5db3afaa683f62fe35b60ff6eb723dad28a11d2b12f8c"
            ],
            covenant_quorum: 6,
            unbonding_time: 1008,
            unbonding_fee: 32000,
            max_staking_amount: 500000000000,
            min_staking_amount: 500000,
            min_staking_time: 64000,
            max_staking_time: 64000,
            confirmation_depth: 10
        };
        object::add_field(param_obj, version, bbn_global_params_2);
        object::borrow_mut(param_obj).max_version = version;
    }

    fun new_bbn_stake_seal(
        block_height: u64, txid: address, staking_output_index: u32, tag: vector<u8>, staker_pub_key: vector<u8>,
        finality_provider_pub_key: vector<u8>, staking_time: u16, staking_value: u64
    ): Object<BBNStakeSeal> {
        object::new(BBNStakeSeal {
            block_height,
            txid,
            staking_output_index,
            tag,
            staker_pub_key,
            finality_provider_pub_key,
            staking_time,
            staking_value
        })
    }

    fun drop_bbn_stake_seal(stake_object_id: ObjectID, seal: BBNStakeSeal) {
        let BBNStakeSeal {
            block_height,
            txid,
            staking_output_index,
            tag:_,
            staker_pub_key:_,
            finality_provider_pub_key:_,
            staking_time,
            staking_value
        } = seal;
        event::emit(BBNStakingUnbondingEvent {
            block_height,
            txid,
            staking_output_index,
            staking_time,
            staking_value,
            stake_object_id
        });
    }


    fun get_bbn_param_version(height: u64): Option<u64> {
        if(height < BBN_V1_ACTIVATION_HEIGHT){
            none()
        } else if (height <= BBN_V1_CAP_HEIGHT){
            some(1)
        } else if (height >= BBN_V2_ACTIVATION_HEIGHT && height <= BBN_V2_CAP_HEIGHT){
            some(2)
        }else{
            none()
        }
    }

    fun get_bbn_param(version: u64): &BBNGlobalParamV1 {
        let object_id = object::named_object_id<BBNGlobalParams>();
        let param_obj = object::borrow_object<BBNGlobalParams>(object_id);
        object::borrow_field(param_obj, version)
    }

    fun try_get_bbn_op_return_ouput(tx_output: &vector<TxOut>): Option<BBNOpReturnOutput> {
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
                        op_return_output_idx: (index as u32),
                        op_return_data: option::destroy_some(op_return_data_opt)
                    });
                }
            };
            index = index + 1;
        };
        result
    }

    fun try_get_bbn_op_return_ouput_from_tx_bytes(bytes: vector<u8>): Option<BBNOpReturnOutput> {
        let tx = bcs::from_bytes<Transaction>(bytes);
        try_get_bbn_op_return_ouput(types::tx_output(&tx))
    }

    fun try_get_bbn_staking_output(tx_output: &vector<TxOut>, staking_output_pk_script: &ScriptBuf): Option<u32> {
        let result: Option<u32> = none();
        let index = 0;
        let output_len = length(tx_output);
        while (index < output_len) {
            let output = borrow(tx_output, index);
            let script_pubkey = txout_script_pubkey(output);
            if (script_pubkey == staking_output_pk_script) {
                if (is_some(&result)) {
                    // bbn only allow for one staking output per transaction
                    return none()
                } else {
                    result = some((index as u32))
                }
            };
            index = index + 1;
        };
        result
    }

    fun try_get_bbn_staking_output_from_tx_bytes(bytes: vector<u8>, staking_output_pk_script: vector<u8>): Option<u32> {
        let tx = bcs::from_bytes<Transaction>(bytes);
        try_get_bbn_staking_output(types::tx_output(&tx), &script_buf::new(staking_output_pk_script))
    }

    /// Deprecated function
    /// Use `bbn_updater::is_possible_bbn_tx` instead
    public fun is_possible_bbn_tx(_txid: address): bool {
        abort DeprecatedFunction
    }

    /// Check if the transaction is a possible Babylon transaction
    /// If the transaction contains an OP_RETURN output with the correct tag, it is considered a possible Babylon transaction
    public fun is_possible_bbn_transaction(block_height: u64, tx: &Transaction): bool {
        let version_opt = get_bbn_param_version(block_height);
        if (is_none(&version_opt)) {
            return false
        };
        let version = option::destroy_some(version_opt);
        let param = get_bbn_param(version);
        let output_opt = try_get_bbn_op_return_ouput(types::tx_output(tx));
        if (is_none(&output_opt)) {
            return false
        };
        let output = option::destroy_some(output_opt);
        if (output.op_return_data.tag != param.tag) {
            return false
        };
        true 
    }

    /// Deprecated function
    /// Use `bbn_updater::process_bbn_tx_entry` instead
    public entry fun process_bbn_tx_entry(_txid: address){
        abort DeprecatedFunction
    }

    fun validate_bbn_op_return_data(param: &BBNGlobalParamV1, op_return_data: &BBNV0OpReturnData): Result<bool,String> {
        if (op_return_data.version != 0) {
            return err_str(b"Invalid version")
        };
        if (op_return_data.tag != param.tag) {
            return err_str(b"Invalid tag")
        };
        if (op_return_data.staking_time < param.min_staking_time
            || op_return_data.staking_time > param.max_staking_time) {
            return err_str(b"Invalid staking time")
        };
        ok(true)
    }

    public(friend) fun process_bbn_transaction(block_height: u64, tx: &Transaction) {
        let version_opt = get_bbn_param_version(block_height);
        if (is_none(&version_opt)) {
            return
        };
        let version = option::destroy_some(version_opt);
        let param = get_bbn_param(version);
        let tx_output = types::tx_output(tx);

        let op_return_output_opt = try_get_bbn_op_return_ouput(tx_output);
        if(is_none(&op_return_output_opt)) {
            return
        };
        let op_return_output = option::destroy_some(op_return_output_opt);
        let txid = types::tx_id(tx);
        let process_result = process_parsed_bbn_tx(param, txid, block_height, tx, op_return_output);
        if (is_err(&process_result)) {
            let error = result::unwrap_err(process_result);
            let event = BBNStakingFailedEvent {
                block_height,
                txid,
                error
            };
            event::emit(event);
        }else{
            let stake_object_id = result::unwrap(process_result);
            let event = BBNStakingEvent {
                block_height,
                txid,
                stake_object_id
            };
            event::emit(event);
        }
    }

    fun process_parsed_bbn_tx(param: &BBNGlobalParamV1, txid: address, block_height: u64, tx: &Transaction, op_return_output: BBNOpReturnOutput): Result<ObjectID, String> {
        
        let BBNOpReturnOutput{op_return_output_idx: _, op_return_data} = op_return_output;
        let valid_result = validate_bbn_op_return_data(param, &op_return_data);
        
        if (is_err(&valid_result)) {
            return as_err(valid_result)
        };

        let staking_output_pk_script = build_staking_tx_output_script_pubkey(
            op_return_data.staker_pub_key, vector::singleton(op_return_data.finality_provider_pub_key), param.covenant_pks, param.covenant_quorum, op_return_data.staking_time
        );

        let tx_output = types::tx_output(tx);
        let staking_output_opt = try_get_bbn_staking_output(tx_output, &staking_output_pk_script);
        if (is_none(&staking_output_opt)) {
            return err_str(b"Staking output not found")
        };
    
        let staking_output_idx = option::destroy_some(staking_output_opt);
        let staking_output = borrow(tx_output, (staking_output_idx as u64));

        let seal_protocol = type_info::type_name<BBNStakeSeal>();

        let txout_value = txout_value(staking_output);

        if(txout_value < param.min_staking_amount || txout_value > param.max_staking_amount){
            return err_str(b"Invalid staking amount")
        };

        let out_point = types::new_outpoint(txid, staking_output_idx);
        let utxo_obj = utxo::borrow_mut_utxo(out_point);
        let utxo = object::borrow_mut(utxo_obj);
        
        assert!(!utxo::has_seal_internal(utxo, &seal_protocol), ErrorUTXOAlreadySealed);

        let bbn_stake_seal_obj = new_bbn_stake_seal(
            block_height, txid, staking_output_idx, op_return_data.tag, 
            op_return_data.staker_pub_key, op_return_data.finality_provider_pub_key,
            op_return_data.staking_time, txout_value
        );
        let seal_object_id = object::id(&bbn_stake_seal_obj);
        let staker_address = pubkey_to_rooch_address(&op_return_data.staker_pub_key);
        object::transfer_extend(bbn_stake_seal_obj, staker_address);
        
        let seal = utxo::new_utxo_seal(seal_protocol, seal_object_id);
        utxo::add_seal_internal(utxo, seal);

        return ok(seal_object_id)
    }

    public(friend) fun on_utxo_spend(utxo: &mut UTXO){
        let seal_obj_ids = utxo::remove_seals_internal<BBNStakeSeal>(utxo);
        vector::for_each(seal_obj_ids, |seal_obj_id|{
            remove_bbn_seal(seal_obj_id);
        });
    }

    public(friend) fun remove_bbn_seal(seal_obj_id: ObjectID){
        let stake_seal_obj = object::take_object_extend<BBNStakeSeal>(seal_obj_id);
        drop_temp_area(&mut stake_seal_obj);
        let seal = object::remove(stake_seal_obj);
        drop_bbn_stake_seal(seal_obj_id, seal);
    }

    fun drop_temp_area(seal_obj: &mut Object<BBNStakeSeal>){
        if(object::contains_field(seal_obj, TEMPORARY_AREA)){
            let seal_obj_id = object::id(seal_obj);
            let seal = object::borrow(seal_obj);
            let staking_time = seal.staking_time;
            let staking_value = seal.staking_value;
            let temp_state = object::remove_field(seal_obj, TEMPORARY_AREA);
            let state_type_names = temp_state::remove(temp_state);
            let idx = 0;
            let len = vector::length(&state_type_names);
            while(idx < len){
                let state_type_name = vector::pop_back(&mut state_type_names);
                event_queue::emit(state_type_name, TempStateDropEvent{
                    stake_object_id: seal_obj_id,
                    staking_time,
                    staking_value,
                });
                idx = idx + 1;
            };
        }
    }


    fun pubkey_to_rooch_address(pubkey: &vector<u8>): address {
        to_rooch_address(&derive_bitcoin_taproot_address_from_pubkey(pubkey))
    }

    fun parse_bbn_op_return_data(script_buf: &ScriptBuf): Option<BBNV0OpReturnData>{
        // 1. OP_RETURN opcode - which signalizes that data is provably unspendable
	    // 2. OP_DATA_71 opcode - which pushes 71 bytes of data to the stack
        let script_bytes = script_buf::bytes(script_buf);
        let script_len = vector::length(script_bytes);
        if (script_len != 73 || *vector::borrow(script_bytes, 0) != opcode::op_return() || *vector::borrow(script_bytes, 1) != opcode::op_pushbytes_71()){
            return none() 
        };
        let tag = vector::slice(script_bytes, 2, 6);
        let version = *vector::borrow(script_bytes, 6);
        
        let staker_pub_key = vector::slice(script_bytes, 7, 39);
        let finality_provider_pub_key = vector::slice(script_bytes, 39, 71);
        let staking_time = bytes_to_u16(vector::slice(script_bytes, 71, 73));
        some(BBNV0OpReturnData{
            tag,
            version,
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

    public fun staking_output_index(stake: &BBNStakeSeal): u32 {
        stake.staking_output_index
    }

    public fun outpoint(stake: &BBNStakeSeal): types::OutPoint {
        types::new_outpoint(stake.txid, stake.staking_output_index)
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

    public fun staking_value(stake: &BBNStakeSeal): u64 {
        stake.staking_value
    }

    /// Deprecated function
    /// Use `bbn_updater::is_expired` instead
    public fun is_expired(_stake: &BBNStakeSeal): bool {
        abort DeprecatedFunction
    }

    public fun is_expired_at(stake: &BBNStakeSeal, current_block_height: u64): bool {
        current_block_height > (stake.block_height + (stake.staking_time as u64))
    }

    // ============== BBNScriptPaths ==============

    fun build_staking_tx_output_script_pubkey(
        staker_key: vector<u8>,
        fp_keys: vector<vector<u8>>,
        covenant_keys: vector<vector<u8>>,
        covenant_quorum: u32,
        staking_time: u16,
    ): ScriptBuf {
        let script_paths = build_bbn_script_paths(staker_key, fp_keys, covenant_keys, covenant_quorum, staking_time);
        let tb = assemble_taproot_script_tree(script_paths.time_lock_path_script, script_paths.unbonding_path_script, script_paths.slashing_path_script);
        let taproot_root_result = taproot_builder::finalize(tb);
        let taproot_root = result::assert_ok(taproot_root_result, ErrorFailedToFinalizeTaproot);
        let addr = bitcoin_address::p2tr(&UNSPENDABLEKEYPATHKEY, option::some(taproot_root));
        script_buf::script_pubkey(&addr)
    }

    fun build_bbn_script_paths(
        staker_key: vector<u8>, 
        fp_keys: vector<vector<u8>>, 
        covenant_keys: vector<vector<u8>>, 
        covenant_quorum: u32, lock_time: u16): BBNScriptPaths {

        let time_lock_path_script = build_time_lock_script(staker_key, lock_time);

        let covenant_multisig_script = build_multi_sig_script(covenant_keys, covenant_quorum, false);

        let staker_sig_script = build_single_key_sig_script(staker_key, true);

        let fp_multisig_script = build_multi_sig_script(fp_keys, 1, true);

        let unbonding_path_script = aggregate_scripts(vector[
            staker_sig_script,
            covenant_multisig_script
        ]);

        let slashing_path_script = aggregate_scripts(vector[
            staker_sig_script,
            fp_multisig_script,
            covenant_multisig_script
        ]);

        BBNScriptPaths {
            time_lock_path_script,
            unbonding_path_script,
            slashing_path_script,
        }
    }

    fun build_time_lock_script(pub_key: vector<u8>, lock_time: u16): ScriptBuf {
        let builder = script_buf::empty();
        script_buf::push_x_only_key(&mut builder, pub_key);
        script_buf::push_opcode(&mut builder, opcode::op_checksigverify());
        script_buf::push_int(&mut builder, (lock_time as u64));
        script_buf::push_opcode(&mut builder, opcode::op_csv());
        builder
    }

    fun build_single_key_sig_script(pub_key: vector<u8>, with_verify: bool): ScriptBuf {
        let builder = script_buf::empty();
        script_buf::push_x_only_key(&mut builder, pub_key);
        if (with_verify){
            script_buf::push_opcode(&mut builder, opcode::op_checksigverify());
        }else{
            script_buf::push_opcode(&mut builder, opcode::op_checksig());
        };
        builder
    }

    fun build_multi_sig_script(
        keys: vector<vector<u8>>,
        threshold: u32,
        with_verify: bool
    ): ScriptBuf {
        let sb = script_buf::empty();
        let keys_len = vector::length(&keys);
        let threshold = (threshold as u64);
        assert!(keys_len > 0, ErrorNoKeysProvided);
        assert!(threshold <= keys_len, ErrorInvalidThreshold);
        if (keys_len == 1) {
            return build_single_key_sig_script(*vector::borrow(&keys, 0), with_verify)
        };

        sort::sort(&mut keys);
        
        let i = 0;
        while (i < keys_len) {
            script_buf::push_data(&mut sb, *vector::borrow(&keys, i));
            if (i == 0) {
                script_buf::push_opcode(&mut sb, opcode::op_checksig());
            } else {
                script_buf::push_opcode(&mut sb, opcode::op_checksigadd());
            };
            i = i + 1;
        };
        
        script_buf::push_int(&mut sb, threshold);
        if (with_verify) {
            script_buf::push_opcode(&mut sb, opcode::op_numequalverify());
        } else {
            script_buf::push_opcode(&mut sb, opcode::op_numequal());
        };
        
        sb
    }

    fun aggregate_scripts(scripts: vector<ScriptBuf>): ScriptBuf {
        let final_script = vector::empty();
        vector::for_each(scripts, |sb| {
            vector::append(&mut final_script, script_buf::into_bytes(sb));
        });
        script_buf::new(final_script)
    }

    fun assemble_taproot_script_tree(
        time_lock_script: ScriptBuf,
        unbonding_path_script: ScriptBuf,
        slashing_path_script: ScriptBuf,
    ): TaprootBuilder {
        let builder = taproot_builder::new();
        taproot_builder::add_leaf(&mut builder, 2, time_lock_script);
        taproot_builder::add_leaf(&mut builder, 2, unbonding_path_script);
        taproot_builder::add_leaf(&mut builder, 1, slashing_path_script);
        builder
    }

    #[test]
    fun test_parse_bbn_op_return_data(){
        //https://mempool.space/tx/7d90210b21aad480cd88fd8399aa6d47e6b3f2ecea2f9f9cfdd79598430e3003
        let script_buf = script_buf::new(x"6a4762626e31000b93d2d388a2b89c2ba2ef28e99c0cfc20735b693cb8b2350fe8aceca2d0f393db9160428e401753dc1a9952ffd4fa3386c7609cf8411d2b6d79c42323ca9923fa00");
        let bbn_opreturn_data_opt = parse_bbn_op_return_data(&script_buf);
        assert!(is_some(&bbn_opreturn_data_opt), 1000);
        let bbn_opreturn_data = option::destroy_some(bbn_opreturn_data_opt);
        let BBNV0OpReturnData{tag, version, staker_pub_key, finality_provider_pub_key, staking_time} = bbn_opreturn_data;
        assert!(tag == x"62626e31", 1001);
        assert!(version == 0u8, 1002);
        assert!(vector::length(&staker_pub_key) == 32, 1003);
        assert!(vector::length(&finality_provider_pub_key) == 32, 1004);
        assert!(staking_time == 64000, 1005);
    }

    #[test]
    fun test_build_time_lock_script() {
        let staker_pk = x"0b93d2d388a2b89c2ba2ef28e99c0cfc20735b693cb8b2350fe8aceca2d0f393";
        let sb = build_time_lock_script(staker_pk, 64000);
        let expected_script = x"200b93d2d388a2b89c2ba2ef28e99c0cfc20735b693cb8b2350fe8aceca2d0f393ad0300fa00b2";
        assert!(script_buf::into_bytes(sb) == expected_script, 1006);
    }

    #[test]
    fun test_covenant_multisig_script() {
        let keys = vector[
            x"d45c70d28f169e1f0c7f4a78e2bc73497afe585b70aa897955989068f3350aaa",
            x"4b15848e495a3a62283daaadb3f458a00859fe48e321f0121ebabbdd6698f9fa",
            x"23b29f89b45f4af41588dcaf0ca572ada32872a88224f311373917f1b37d08d1",
            x"d3c79b99ac4d265c2f97ac11e3232c07a598b020cf56c6f055472c893c0967ae",
            x"8242640732773249312c47ca7bdb50ca79f15f2ecc32b9c83ceebba44fb74df7",
            x"e36200aaa8dce9453567bba108bdc51f7f1174b97a65e4dc4402fc5de779d41c",
            x"cbdd028cfe32c1c1f2d84bfec71e19f92df509bba7b8ad31ca6c1a134fe09204",
            x"f178fcce82f95c524b53b077e6180bd2d779a9057fdff4255a0af95af918cee0",
            x"de13fc96ea6899acbdc5db3afaa683f62fe35b60ff6eb723dad28a11d2b12f8c"
        ];
        let covenant_quorum = 6;
        let sb = build_multi_sig_script(keys, covenant_quorum, false);
        let expected_sb = x"2023b29f89b45f4af41588dcaf0ca572ada32872a88224f311373917f1b37d08d1ac204b15848e495a3a62283daaadb3f458a00859fe48e321f0121ebabbdd6698f9faba208242640732773249312c47ca7bdb50ca79f15f2ecc32b9c83ceebba44fb74df7ba20cbdd028cfe32c1c1f2d84bfec71e19f92df509bba7b8ad31ca6c1a134fe09204ba20d3c79b99ac4d265c2f97ac11e3232c07a598b020cf56c6f055472c893c0967aeba20d45c70d28f169e1f0c7f4a78e2bc73497afe585b70aa897955989068f3350aaaba20de13fc96ea6899acbdc5db3afaa683f62fe35b60ff6eb723dad28a11d2b12f8cba20e36200aaa8dce9453567bba108bdc51f7f1174b97a65e4dc4402fc5de779d41cba20f178fcce82f95c524b53b077e6180bd2d779a9057fdff4255a0af95af918cee0ba569c";
        assert!(script_buf::into_bytes(sb) == expected_sb, 1007);
    }

    #[test]
    fun test_build_staking_tx_output_script_pubkey(){
        genesis_init();
        //https://mempool.space/tx/7d90210b21aad480cd88fd8399aa6d47e6b3f2ecea2f9f9cfdd79598430e3003
        let script_buf = script_buf::new(x"6a4762626e31000b93d2d388a2b89c2ba2ef28e99c0cfc20735b693cb8b2350fe8aceca2d0f393db9160428e401753dc1a9952ffd4fa3386c7609cf8411d2b6d79c42323ca9923fa00");
        let bbn_opreturn_data_opt = parse_bbn_op_return_data(&script_buf);
        let bbn_opreturn_data = option::destroy_some(bbn_opreturn_data_opt);
        let BBNV0OpReturnData{tag:_, version:_, staker_pub_key, finality_provider_pub_key, staking_time} = bbn_opreturn_data;
        let param_version = 1;
        let param = get_bbn_param(param_version);
        let sb = build_staking_tx_output_script_pubkey(
            staker_pub_key,
            vector::singleton(finality_provider_pub_key),
            param.covenant_pks,
            param.covenant_quorum,
            staking_time
        );
        let result = script_buf::into_bytes(sb);
        std::debug::print(&result);
        let expected_sb = x"512082f93ece9366a9e680d152dbc0c487e181accbba145b5b72d00c820545064d44";
        assert!(result == expected_sb, 1008);
    }
}

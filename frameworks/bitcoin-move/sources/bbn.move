module bitcoin_move::bbn {

    use std::option;
    use std::option::{is_none, Option, none, is_some, some};
    use std::vector;
    use std::vector::{for_each_ref, length};
    use bitcoin_move::bitcoin;
    use bitcoin_move::utxo::UTXO;
    use bitcoin_move::types;
    use bitcoin_move::utxo;
    use bitcoin_move::script_buf;
    use bitcoin_move::types::{Transaction, tx_id, tx_output, txout_value, tx_lock_time, txout_script_pubkey};
    use bitcoin_move::bitcoin::get_tx_height;
    use rooch_framework::bitcoin_address::{derive_bitcoin_taproot_address_from_pubkey, to_rooch_address};
    use bitcoin_move::script_buf::{unpack_bbn_stake_data};
    use moveos_std::object::{Object, ObjectID};
    use moveos_std::object;

    friend bitcoin_move::genesis;

    struct BBNGlobalParam has key {
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

    const UNSPENDABLEKEYPATHKEY: vector<u8> = b"0250929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0";

    const ErrorNotBabylonUTXO: u64 = 0;
    const ErrorNotTransaction: u64 = 1;
    const ErrorNotBabylonOpReturn: u64 = 2;
    const ErrorTransactionLockTime: u64 = 3;

    public(friend) fun genesis_init() {
        // TODO here just add bbn test-4 version 2
        let bbn_global_params_2 = BBNGlobalParam{
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
        let obj = object::new_named_object(BBNGlobalParams{
            bbn_global_param: vector[bbn_global_params_2]
        });
        object::to_shared(obj);
    }

    fun borrow_bbn_params(): &Object<BBNGlobalParams>{
        let object_id = object::named_object_id<BBNGlobalParams>();
        object::borrow_object(object_id)
    }

    fun borrow_bbn_params_mut(): &mut Object<BBNGlobalParams>{
        let object_id = object::named_object_id<BBNGlobalParams>();
        object::borrow_mut_object_shared(object_id)
    }

    public fun try_get_bbn_op_return_data(transaction: Transaction): (bool, u64, BBNOpReturnData){
        let bbn_op_return_data = BBNOpReturnData{
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
        for_each_ref(tx_output, |output| {
            (bbn_op_return_data.tag, bbn_op_return_data.version, bbn_op_return_data.staker_pub_key, bbn_op_return_data.finality_provider_pub_key, bbn_op_return_data.staking_time) = unpack_bbn_stake_data(txout_script_pubkey(output));
            if (vector::length(&bbn_op_return_data.tag) != 0){
                break
            };
            index = index + 1;
        }) ;
        if (vector::length(&bbn_op_return_data.tag) == 0){
            return (false, 0, bbn_op_return_data)
        };
        let option_tx_height = get_tx_height(tx_id(&transaction));
        if (is_none(&option_tx_height)){
            return (false, 0, bbn_op_return_data)
        };
        let tx_height = option::destroy_some(option_tx_height);
        let bbn_params = object::borrow(borrow_bbn_params());
        for_each_ref(&bbn_params.bbn_global_param, |param| {
            if (bbn_op_return_data.version != param.version || bbn_op_return_data.tag != param.tag || tx_height < param.activation_height || param.covenant_quorum > (length(&param.covenant_pks) as u32)){
                continue
            };
            if (param.cap_height != 0 && tx_height > (param.activation_height + param.cap_height)){
                continue
            };
            if (bbn_op_return_data.staking_time < param.min_staking_time || bbn_op_return_data.staking_time > param.max_staking_time){
                continue
            };
            if (!vector::contains(&param.covenant_pks, &bbn_op_return_data.finality_provider_pub_key)){
                continue
            };

            return (true, index, bbn_op_return_data)
        });
        return (false, 0, bbn_op_return_data)
    }

    public fun try_get_staking_output(transaction: Transaction, staking_output_script: &vector<u8>): (bool, u64, Option<ObjectID>){
        let tx_outputs = tx_output(&transaction);
        let tx_id = tx_id(&transaction);
        if (vector::length(tx_outputs) == 0) {
            return (false, 0 , none())
        };
        let index = 0;

        // should not multiple staking outputs
        for_each_ref(tx_outputs, |tx_output| {
            if (script_buf::bytes(tx_output.txout_script_pubkey()) == staking_output_script) {
                let out_point = types::new_outpoint(tx_id, (txout_value(tx_output) as u32));
                return (true, index, option::some(utxo::derive_utxo_id(out_point)))
            };
            index = index + 1;
        });
        return (false, index , none())
    }

    public fun derive_bbn_utxo(utxo_obj: &Object<UTXO>) {
        // assert!(object::owner(utxo_obj) == @bitcoin_move, ErrorNotBabylonUTXO);
        let utxo = object::borrow(utxo_obj);
        let txid = utxo::txid(utxo);
        let option_tx = bitcoin::get_tx(txid);
        assert!(is_some(&option_tx), ErrorNotTransaction);
        let transaction = option::destroy_some(option_tx);
        let (is_true, op_return_index, op_return_data) = try_get_bbn_op_return_data(transaction);
        assert!(is_true, ErrorNotBabylonOpReturn);
        // TODO here should replace to check staking output
        // try_get_staking_output()

        assert!(tx_lock_time(&transaction) >= (op_return_data.staking_time as u32), ErrorTransactionLockTime);
        let tx_outputs = tx_output(&transaction);
        let index = 0;
        // TODO bbn should not multiple staking outputs, we temporarily support
        for_each_ref(tx_outputs, |tx_output| {
            if (index == op_return_index) {
                continue
            };
            let out_point = types::new_outpoint(txid, (txout_value(tx_output) as u32));
            let borrow_utxo = utxo::borrow_utxo(out_point);
            if (object::owner(borrow_utxo) != @bitcoin_move){
                continue
            };
            let utxo_id = utxo::derive_utxo_id(out_point);
            let utxo_obj = utxo::take(utxo_id);
            utxo::add_temp_state(&mut utxo_obj, op_return_data);
            // TODO here should modify sender to trigger event queue?
            utxo::transfer(utxo_obj, some(@bitcoin_move), pubkey_to_rooch_address(&op_return_data.staker_pub_key));
            index = index + 1;
        });
    }

    // TODO build stake info


    fun pubkey_to_rooch_address(pubkey: &vector<u8>): address {
        to_rooch_address(&derive_bitcoin_taproot_address_from_pubkey(pubkey))
    }

}

module rooch_nursery::btc_script {

    use std::string::String;
    use std::vector;
    use std::vector::{length, for_each};
    use bitcoin_move::script_buf::{script_pubkey, ScriptBuf};
    use bitcoin_move::types::{OutPoint, new_outpoint};
    use bitcoin_move::utxo::{UTXO, value, txid, vout};
    use moveos_std::hex;
    use moveos_std::json;
    use moveos_std::object;
    use moveos_std::object::ObjectID;
    use rooch_framework::bitcoin_address;

    const ErrorInvalidOutputLen: u64 = 1;

    const ENABLE_RBF_NO_LOCKTIME: u32 = 0xFFFFFFFD;
    const TWO: u32 = 2;

    #[data_struct]
    struct Transaction has store, copy, drop {
        /// The protocol version, is currently expected to be 1 or 2 (BIP 68).
        version: u32,
        /// Block height or timestamp. Transaction cannot be included in a block until this height/time.
        ///
        /// ### Relevant BIPs
        ///
        /// * [BIP-65 OP_CHECKLOCKTIMEVERIFY](https://github.com/bitcoin/bips/blob/master/bip-0065.mediawiki)
        /// * [BIP-113 Median time-past as endpoint for lock-time calculations](https://github.com/bitcoin/bips/blob/master/bip-0113.mediawiki)
        lock_time: u32,
        /// List of transaction inputs.
        input: vector<TxIn>,
        /// List of transaction outputs.
        output: vector<TxOut>,
    }

    #[data_struct]
    struct TxIn has store, copy, drop {
        /// The reference to the previous output that is being used as an input.
        previous_output: OutPoint,
        /// The script which pushes values on the stack which will cause
        /// the referenced output's script to be accepted.
        script_sig: vector<u8>,
        /// The sequence number, which suggests to miners which of two
        /// conflicting transactions should be preferred, or 0xFFFFFFFF
        /// to ignore this feature. This is generally never used since
        /// the miner behavior cannot be enforced.
        sequence: u32,
        /// Witness data: an array of byte-arrays.
        /// Note that this field is *not* (de)serialized with the rest of the TxIn in
        /// Encodable/Decodable, as it is (de)serialized at the end of the full
        /// Transaction. It *is* (de)serialized with the rest of the TxIn in other
        /// (de)serialization routines.
        witness: Witness,
    }

    #[data_struct]
    struct Witness has store, copy, drop {
        witness: vector<vector<u8>>,
    }

    #[data_struct]
    struct TxOut has store, copy, drop {
        /// The value of the output, in satoshis.
        value: u64,
        /// The script which must be satisfied for the output to be spent.
        script_pubkey: ScriptBuf,
    }

    public fun send_btc(inputs: vector<ObjectID>, outputs_address: vector<String>, outputs_amount: vector<u64>, change_address: String, fee: u64, lock_time: u32): vector<u8> {
        let outputs_address_len = length(&outputs_address);
        let outputs_amount_len = length(&outputs_amount);
        assert!( outputs_address_len == outputs_amount_len, ErrorInvalidOutputLen);
        let i = 0;
        let tx_output = vector::empty<TxOut>();
        let total_output = 0;
        while (i < outputs_address_len) {
            let recipient = vector::borrow(&outputs_address, i);
            let recipient_addr = bitcoin_address::from_string(recipient);
            let script_pubkey = script_pubkey(&recipient_addr);
            let value = *vector::borrow(&outputs_amount, i);
            vector::push_back(&mut tx_output, TxOut { script_pubkey, value });
            total_output = total_output + value;
            i = i + 1;
        };
        let tx_inputs = vector::empty<TxIn>();
        let total_input = 0;
        for_each(inputs, |input| {
            let utxo_obj = object::borrow_object<UTXO>(input);
            let utxo = object::borrow(utxo_obj);
            total_input = total_input + value(utxo);
            vector::push_back(&mut tx_inputs,
                TxIn{
                    previous_output: new_outpoint(txid(utxo), vout(utxo)),
                    script_sig: vector::empty<u8>(),
                    sequence: ENABLE_RBF_NO_LOCKTIME,
                    witness: Witness { witness: vector::empty<vector<u8>>() }
                });
        });
        let change = total_input - total_output - fee;
        if (change > 0) {
            let change_address = bitcoin_address::from_string(&change_address);
            vector::push_back(&mut tx_output, TxOut { script_pubkey: script_pubkey(&change_address), value: change });
        };
        let tx = Transaction { lock_time, input: tx_inputs, output: tx_output, version: TWO};
        return hex::encode(json::to_json(&tx))
    }

}

// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

#[test_only]
module bitcoin_move::state_reset_tests {
    use std::option;
    use moveos_std::account;
    use moveos_std::object;
    use bitcoin_move::genesis;
    use bitcoin_move::ord;
    use bitcoin_move::types;
    use bitcoin_move::utxo;
    use rooch_framework::onchain_config;

    #[test]
    fun test_reset_utxo_store() {
        genesis::init_for_test();
        let txid = @0x77dfc2fe598419b00641c296181a96cf16943697f573480b023b77cce82ada21;
        let vout = 0;
        let outpoint = types::new_outpoint(txid, vout);
        let utxo_obj = utxo::new_for_testing(txid, vout, 100);
        utxo::transfer_for_testing(utxo_obj, @0x42);
        assert!(utxo::exists_utxo(outpoint), 1);
        assert!(object::field_size(utxo::borrow_utxo_store()) == 1, 2);
        let admin = account::create_account_for_testing(onchain_config::admin());
        utxo::reset_utxo_store(&admin);
        assert!(!utxo::exists_utxo(outpoint), 3);
        assert!(object::field_size(utxo::borrow_utxo_store()) == 0, 4);
    }

    #[test]
    fun test_reset_inscription_store() {
        genesis::init_for_test();
        let txid = @0x77dfc2fe598419b00641c296181a96cf16943697f573480b023b77cce82ada21;
        let inscription_id = ord::new_inscription_id(txid, 0);
        let inscription_obj = ord::new_inscription_object_for_test(
            inscription_id,
            0,
            0,
            vector[],
            option::none(),
            option::none(),
            vector[],
            option::none(),
            vector[],
            option::none(),
        );
        ord::transfer_inscription_for_test(inscription_obj, @0x42);
        assert!(ord::inscription_store_field_size_for_test() == 2, 1);
        assert!(ord::exists_inscription(inscription_id), 2);
        let admin = account::create_account_for_testing(onchain_config::admin());
        ord::reset_inscription_store(&admin);
        assert!(ord::inscription_store_field_size_for_test() == 0, 3);
        assert!(!ord::exists_inscription(inscription_id), 4);
        assert!(ord::get_inscription_next_sequence_number() == 0, 5);
    }
}

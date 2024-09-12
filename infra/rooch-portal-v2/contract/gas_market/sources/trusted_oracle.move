module gas_market::trusted_oracle {
    use std::signer::address_of;
    use std::string::{utf8, String};
    use std::vector;
    use moveos_std::event::emit;
    use rooch_framework::oracle::{SimpleOracle, OracleAdminCap};
    use moveos_std::decimal_value::{DecimalValue, new};
    use moveos_std::tx_context::sender;
    use moveos_std::signer::module_signer;
    use moveos_std::account::{move_resource_to, borrow_resource};
    use moveos_std::object::{to_shared, ObjectID, transfer, Object};
    use moveos_std::object;
    use rooch_framework::oracle;
    use rooch_framework::oracle_meta;
    #[test_only]
    use moveos_std::timestamp;
    #[test_only]
    use rooch_framework::genesis;


    struct Oracle has key{
        ids: vector<ObjectID>
    }

    struct NewOracleEvent has copy, drop {
        name: String,
        oracle_id: ObjectID,
        admin_id: ObjectID
    }

    fun init() {
        let (oracle1, admin_cap1)= oracle::create(utf8(b"pyth"), utf8(b"https://hermes.pyth.network"), utf8(b"Price Data From Pyth"));
        let (oracle2, admin_cap2)= oracle::create(utf8(b"binance"), utf8(b"https://api.binance.com/api/v3/ticker/price"), utf8(b"Price Data From Binance"));
        let (oracle3, admin_cap3)= oracle::create(utf8(b"okex"), utf8(b"https://www.okx.com/api/v5/market/tickers?instType=SPOT"), utf8(b"Price Data From Okex"));
        let signer = module_signer<Oracle>();
        move_resource_to(&signer, Oracle{
            ids: vector[object::id(&oracle1), object::id(&oracle2), object::id(&oracle3)]
        });
        emit(NewOracleEvent{
            name: utf8(b"pyth"),
            oracle_id: object::id(&oracle1),
            admin_id: object::id(&admin_cap1)
        });
        emit(NewOracleEvent{
            name: utf8(b"binance"),
            oracle_id: object::id(&oracle2),
            admin_id: object::id(&admin_cap2)
        });
        emit(NewOracleEvent{
            name: utf8(b"okex"),
            oracle_id: object::id(&oracle3),
            admin_id: object::id(&admin_cap3)
        });

        to_shared(oracle1);
        to_shared(oracle2);
        to_shared(oracle3);

        transfer(admin_cap1, sender());
        transfer(admin_cap2, sender());
        transfer(admin_cap3, sender());
    }

    public fun trusted_price(ticker: String): DecimalValue {
        let meta_oracle = oracle_meta::new<DecimalValue>(2, 60000, ticker);
        let signer = module_signer<Oracle>();
        let oracle = borrow_resource<Oracle>(address_of(&signer));
        let i = 0;
        while (i < vector::length(&oracle.ids)){
            let oracle_id = vector::borrow(&oracle.ids, i);
            oracle_meta::add_simple_oracle(&mut meta_oracle, object::borrow_object(*oracle_id));
            i = i + 1;
        };

        let trusted_data = oracle_meta::median(meta_oracle);
        *oracle_meta::value(&trusted_data)
    }

    public entry fun submit_data(
        oracle_obj: &mut Object<SimpleOracle>,
        ticker: String,
        value: u256,
        decimal: u8,
        identifier: String,
        admin_obj: &mut Object<OracleAdminCap>,
    ){
        let decimal_value = new(value, decimal);
        oracle::submit_data(oracle_obj, ticker, decimal_value, identifier, admin_obj)
    }

    #[test]
    fun test_btc_price() {
        genesis::init_for_test();
        let (oracle1, admin_cap1)= oracle::create(utf8(b"pyth"), utf8(b"https://hermes.pyth.network"), utf8(b"Price Data From Pyth"));
        let (oracle2, admin_cap2)= oracle::create(utf8(b"binance"), utf8(b"https://api.binance.com/api/v3/ticker/price"), utf8(b"Price Data From Binance"));
        let (oracle3, admin_cap3)= oracle::create(utf8(b"okex"), utf8(b"https://www.okx.com/api/v5/market/tickers?instType=SPOT"), utf8(b"Price Data From Okex"));
        let signer = module_signer<Oracle>();
        move_resource_to(&signer, Oracle{
            ids: vector[object::id(&oracle1), object::id(&oracle2), object::id(&oracle3)]
        });
        timestamp::fast_forward_milliseconds_for_test(100000000);
        submit_data(&mut oracle1, utf8(b"BTCUSD"), 5805106000000, 8, utf8(b"1"), &mut admin_cap1);
        submit_data(&mut oracle2, utf8(b"BTCUSD"), 5805206000000, 8, utf8(b"2"), &mut admin_cap2);
        submit_data(&mut oracle3, utf8(b"BTCUSD"), 5805306000000, 8, utf8(b"3"), &mut admin_cap3);
        to_shared(oracle1);
        to_shared(oracle2);
        to_shared(oracle3);

        transfer(admin_cap1, sender());
        transfer(admin_cap2, sender());
        transfer(admin_cap3, sender());
        let price = trusted_price(utf8(b"BTCUSD"));
        assert!(price.value() == 5805206000000, 1);
        assert!(price.decimal() == 8, 1);
    }
}
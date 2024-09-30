module gas_market::trusted_oracle {
    use std::signer::address_of;
    use std::string::{String};
    use std::vector;
    use moveos_std::decimal_value::{DecimalValue}; 
    use moveos_std::signer::module_signer;
    use moveos_std::account::{move_resource_to, borrow_resource, borrow_mut_resource};
    use moveos_std::object::{ObjectID, Object};
    use moveos_std::object;

    use rooch_framework::oracle_meta;
    use rooch_framework::oracle::{SimpleOracle};

    use gas_market::admin::{AdminCap};
 

    const ErrorTrustedOracleNotExists: u64 = 1;
    const ErrorTrustedOracleAlreadyExists: u64 = 2;

    struct Oracle has key {
        ids: vector<ObjectID>
    }

    fun init() {
        let signer = module_signer<Oracle>();
        move_resource_to(
            &signer,
            Oracle {
                ids: vector[]
            }
        );
    }

    public fun trusted_price(ticker: String): DecimalValue {
        let meta_oracle = oracle_meta::new<DecimalValue>(2, 60000, ticker);
        let signer = module_signer<Oracle>();
        let oracle = borrow_resource<Oracle>(address_of(&signer));
        let i = 0;
        while (i < vector::length(&oracle.ids)) {
            let oracle_id = vector::borrow(&oracle.ids, i);
            oracle_meta::add_simple_oracle(
                &mut meta_oracle, object::borrow_object(*oracle_id)
            );
            i = i + 1;
        };

        let trusted_data = oracle_meta::median(meta_oracle);
        *oracle_meta::value(&trusted_data)
    }

    public entry fun add_trusted_oracle(
        oracle_obj: &Object<SimpleOracle>,
        _admin: &mut Object<AdminCap>,
    ) {
        let oracle = borrow_mut_resource<Oracle>(@gas_market);
        let oracle_id = object::id(oracle_obj);
        assert!(
            !vector::contains(&oracle.ids, &oracle_id),
            ErrorTrustedOracleAlreadyExists
        );
        vector::push_back(&mut oracle.ids, oracle_id);
    }

    public entry fun remove_trusted_oracle(
        oracle_obj: &Object<SimpleOracle>,
        _admin: &mut Object<AdminCap>,
    ) {
        let oracle = borrow_mut_resource<Oracle>(@gas_market);
        let removed = vector::remove_value(&mut oracle.ids, &object::id(oracle_obj));
        assert!(vector::length(&removed) > 0, ErrorTrustedOracleNotExists);
    } 

    #[test_only]
    use std::string::utf8;
    #[test_only]
    use moveos_std::object::{to_shared, transfer};
    #[test_only]
    use moveos_std::timestamp;
    #[test_only]
    use moveos_std::decimal_value;
    #[test_only]
    use moveos_std::tx_context::sender;
    #[test_only]
    use rooch_framework::genesis;
    #[test_only]
    use rooch_framework::oracle;
    #[test_only]
    use gas_market::admin;

    #[test]
    fun test_btc_price() {
        genesis::init_for_test();
        let admin_cap = admin::init_for_test();
        init();

        let (oracle1, admin_cap1) =
            oracle::create(
                utf8(b"pyth"),
                utf8(b"https://hermes.pyth.network"),
                utf8(b"Price Data From Pyth")
            );
        let (oracle2, admin_cap2) =
            oracle::create(
                utf8(b"binance"),
                utf8(b"https://api.binance.com/api/v3/ticker/price"),
                utf8(b"Price Data From Binance")
            );
        let (oracle3, admin_cap3) =
            oracle::create(
                utf8(b"okex"),
                utf8(b"https://www.okx.com/api/v5/market/tickers?instType=SPOT"),
                utf8(b"Price Data From Okex")
            );

        add_trusted_oracle(&oracle1, admin_cap);
        add_trusted_oracle(&oracle2, admin_cap);
        add_trusted_oracle(&oracle3, admin_cap);

        timestamp::fast_forward_milliseconds_for_test(100000000);
        oracle::submit_decimal_data(
            &mut oracle1,
            utf8(b"BTCUSD"),
            5805106000000,
            8,
            utf8(b"1"),
            &mut admin_cap1
        );
        oracle::submit_decimal_data(
            &mut oracle2,
            utf8(b"BTCUSD"),
            5805206000000,
            8,
            utf8(b"2"),
            &mut admin_cap2
        );
        oracle::submit_decimal_data(
            &mut oracle3,
            utf8(b"BTCUSD"),
            5805306000000,
            8,
            utf8(b"3"),
            &mut admin_cap3
        );
        to_shared(oracle1);
        to_shared(oracle2);
        to_shared(oracle3);

        transfer(admin_cap1, sender());
        transfer(admin_cap2, sender());
        transfer(admin_cap3, sender());
        let price = trusted_price(utf8(b"BTCUSD"));
        assert!(decimal_value::value(&price) == 5805206000000, 1);
        assert!(decimal_value::decimal(&price) == 8, 1);
    }
}

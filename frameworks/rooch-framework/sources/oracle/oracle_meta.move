// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_framework::oracle_meta {
    use std::option;
    use std::option::Option;
    use std::string::String;
    use std::vector;
    use moveos_std::sort::quick_sort;
    use rooch_framework::oracle;
    use moveos_std::object::Object;

    use rooch_framework::oracle_data::{Self, Data};
    use rooch_framework::oracle::SimpleOracle;
    #[test_only]
    use moveos_std::decimal_value;
    #[test_only]
    use moveos_std::decimal_value::DecimalValue;


    const ErrorValidDataSizeLessThanThreshold: u64 = 0;
    const ErrorUnsupportedDataType: u64 = 1;

    struct MetaOracle<T> {
        oracle_data: vector<Option<Data<T>>>,
        threshold: u64,
        time_window_ms: u64,
        ticker: String,
        max_timestamp: u64,
    }

    public fun new<T: copy + drop>(threshold: u64, time_window_ms: u64, ticker: String): MetaOracle<T> {
        MetaOracle {
            oracle_data: vector::empty(),
            threshold,
            time_window_ms,
            ticker,
            max_timestamp: 0,
        }
    }

    public fun add_simple_oracle<T: copy + drop + store>(
        meta_oracle: &mut MetaOracle<T>,
        oracle: &Object<SimpleOracle>
    ) {
        let oracle_data = oracle::get_latest_data(oracle, meta_oracle.ticker);
        if (option::is_some(&oracle_data)) {
            meta_oracle.max_timestamp = oracle_data::timestamp(option::borrow(&oracle_data));
        };
        vector::push_back(&mut meta_oracle.oracle_data, oracle_data);
    }

    struct TrustedData<T> has copy, drop {
        value: T,
        oracles: vector<address>,
    }

    fun combine<T: copy + drop>(meta_oracle: MetaOracle<T>, ): (vector<T>, vector<address>) {
        let MetaOracle { oracle_data, threshold, time_window_ms, ticker: _, max_timestamp } = meta_oracle;
        let min_timestamp = max_timestamp - time_window_ms;
        let values = vector<T>[];
        let oracles = vector<address>[];
        while (vector::length(&oracle_data) > 0) {
            let oracle_data = vector::remove(&mut oracle_data, 0);
            if (option::is_some(&oracle_data)) {
                let oracle_data = option::destroy_some(oracle_data);
                if (oracle_data::timestamp(&oracle_data) > min_timestamp) {
                    vector::push_back(&mut values, *oracle_data::value(&oracle_data));
                    vector::push_back(&mut oracles, *oracle_data::oracle_address(&oracle_data));
                };
            };
        };
        assert!(vector::length(&values) >= threshold, ErrorValidDataSizeLessThanThreshold);
        (values, oracles)
    }

    /// take the median value
    public fun median<T: copy + drop>(meta_oracle: MetaOracle<T>): TrustedData<T> {
        let (values, oracles) = combine(meta_oracle);
        quick_sort(&mut values);
        let i = vector::length(&values) / 2;
        let value = vector::remove(&mut values, i);
        TrustedData { value, oracles }
    }



    public fun data<T>(meta: &MetaOracle<T>): &vector<Option<Data<T>>> {
        &meta.oracle_data
    }

    public fun threshold<T>(meta: &MetaOracle<T>): u64 {
        meta.threshold
    }

    public fun time_window_ms<T>(meta: &MetaOracle<T>): u64 {
        meta.time_window_ms
    }

    public fun ticker<T>(meta: &MetaOracle<T>): String {
        meta.ticker
    }

    public fun max_timestamp<T>(meta: &MetaOracle<T>): u64 {
        meta.max_timestamp
    }

    public fun value<T>(data: &TrustedData<T>): &T {
        &data.value
    }

    public fun oracles<T>(data: &TrustedData<T>): vector<address> {
        data.oracles
    }

    #[test]
    fun test_quick_sort() {
        let data = vector<u64>[1, 3, 2, 5, 4];
        quick_sort(&mut data);
        assert!(vector::length<u64>(&data) == 5, 0);
        assert!(*vector::borrow(&data, 0) == 1, 0);
        assert!(*vector::borrow(&data, 1) == 2, 0);
        assert!(*vector::borrow(&data, 2) == 3, 0);
        assert!(*vector::borrow(&data, 3) == 4, 0);
        assert!(*vector::borrow(&data, 4) == 5, 0);
    }

    #[test]
    fun test_quick_sort_u128() {
        let data = vector<u128>[1, 3, 2, 5, 4];
        quick_sort(&mut data);
        assert!(vector::length<u128>(&data) == 5, 0);
        assert!(*vector::borrow(&data, 0) == 1, 0);
        assert!(*vector::borrow(&data, 1) == 2, 0);
        assert!(*vector::borrow(&data, 2) == 3, 0);
        assert!(*vector::borrow(&data, 3) == 4, 0);
        assert!(*vector::borrow(&data, 4) == 5, 0);
    }

    #[test]
    fun test_quick_sort_decimal_value() {
        let data = vector<DecimalValue>[
            decimal_value::new(1000000, 6),
            decimal_value::new(3000000, 6),
            decimal_value::new(2000000, 6),
            decimal_value::new(5000000, 6),
            decimal_value::new(4000000, 6)];
        quick_sort(&mut data);
        assert!(vector::length<DecimalValue>(&data) == 5, 0);
        assert!(decimal_value::value(vector::borrow(&data, 0)) == 1000000, 0);
        assert!(decimal_value::value(vector::borrow(&data, 1)) == 2000000, 0);
        assert!(decimal_value::value(vector::borrow(&data, 2)) == 3000000, 0);
        assert!(decimal_value::value(vector::borrow(&data, 3)) == 4000000, 0);
        assert!(decimal_value::value(vector::borrow(&data, 4)) == 5000000, 0);
    }

    #[test]
    fun test_quick_sort_decimal_value_different_decimal() {
        let data = vector<DecimalValue>[
            decimal_value::new(60000, 2),
            decimal_value::new(70000, 2),
            decimal_value::new(1000000, 6),
            decimal_value::new(3000000, 6),
            decimal_value::new(2000000, 6),
            decimal_value::new(5000000, 6),
            decimal_value::new(4000000, 6)];

        quick_sort(&mut data);

        assert!(vector::length<DecimalValue>(&data) == 7, 0);
        assert!(decimal_value::value(vector::borrow(&data, 0)) == 1000000, 0);
        assert!(decimal_value::value(vector::borrow(&data, 1)) == 2000000, 0);
        assert!(decimal_value::value(vector::borrow(&data, 2)) == 3000000, 0);
        assert!(decimal_value::value(vector::borrow(&data, 3)) == 4000000, 0);
        assert!(decimal_value::value(vector::borrow(&data, 4)) == 5000000, 0);
        assert!(decimal_value::value(vector::borrow(&data, 5)) == 60000, 0);
        assert!(decimal_value::value(vector::borrow(&data, 6)) == 70000, 0);
    }
}
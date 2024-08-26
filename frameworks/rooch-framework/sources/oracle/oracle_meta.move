// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_framework::oracle_meta {
    use std::option;
    use std::option::Option;
    use std::string::String;
    use std::type_name;
    use std::u256;
    use std::vector;
    use rooch_framework::oracle;
    use moveos_std::bcs;
    use moveos_std::object::Object;

    use rooch_framework::oracle_data::{Self, Data};
    use rooch_framework::oracle_decimal_value::DecimalValue;
    use rooch_framework::oracle::SimpleOracle;

    #[test_only]
    use rooch_framework::oracle_decimal_value;

    const EValidDataSizeLessThanThreshold: u64 = 0;
    const EUnsupportedDataType: u64 = 1;

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
        assert!(vector::length(&values) >= threshold, EValidDataSizeLessThanThreshold);
        (values, oracles)
    }

    /// take the median value
    public fun median<T: copy + drop>(meta_oracle: MetaOracle<T>): TrustedData<T> {
        let (values, oracles) = combine(meta_oracle);
        let sortedData = quick_sort(values);
        let i = vector::length(&sortedData) / 2;
        let value = vector::remove(&mut sortedData, i);
        TrustedData { value, oracles }
    }

    fun cmp<T: copy + drop>(a: &T, b: &T): u8 {
        let t = type_name::get<T>();
        let a = bcs::new(bcs::to_bytes(a));
        let b = bcs::new(bcs::to_bytes(b));

        if (t == type_name::get<u64>()) {
            let a = bcs::peel_u64(&mut a);
            let b = bcs::peel_u64(&mut b);
            if (a > b) {
                return 1
            } else if (a == b) {
                return 0
            } else {
                return 2
            }
        } else if (t == type_name::get<u128>()) {
            let a = bcs::peel_u128(&mut a);
            let b = bcs::peel_u128(&mut b);
            if (a > b) {
                return 1
            } else if (a == b) {
                return 0
            } else {
                return 2
            }
        }else if (t == type_name::get<u8>()) {
            let a = bcs::peel_u8(&mut a);
            let b = bcs::peel_u8(&mut b);
            if (a > b) {
                return 1
            } else if (a == b) {
                return 0
            } else {
                return 2
            }
        } else if (t == type_name::get<DecimalValue>()) {
            let a_value = bcs::peel_u64(&mut a);
            let a_decimal = bcs::peel_u8(&mut a);
            let b_value = bcs::peel_u64(&mut b);
            let b_decimal = bcs::peel_u8(&mut b);
            // Normalise the decimal values
            let a = (a_value as u256) * u256::pow(10, b_decimal);
            let b = (b_value as u256) * u256::pow(10, a_decimal);

            if (a > b) {
                return 1
            } else if (a == b) {
                return 0
            } else {
                return 2
            }
        }else {
            assert!(false, EUnsupportedDataType)
        };
        0
    }

    public fun quick_sort<T: drop + copy>(data: vector<T>): vector<T> {
        if (vector::length(&data) <= 1) {
            return data
        };

        let pivot = *vector::borrow(&data, 0);
        let less = vector<T>[];
        let equal = vector<T>[];
        let greater = vector<T>[];

        while (vector::length(&data) > 0) {
            let value = vector::remove(&mut data, 0);
            let cmp = cmp(&value, &pivot);
            if (cmp == 2) {
                vector::push_back(&mut less, value);
            } else if (cmp == 0) {
                vector::push_back(&mut equal, value);
            } else {
                vector::push_back(&mut greater, value);
            };
        };

        let sortedData = vector<T>[];
        vector::append(&mut sortedData, quick_sort(less));
        vector::append(&mut sortedData, equal);
        vector::append(&mut sortedData, quick_sort(greater));
        sortedData
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
        let sortedData = quick_sort(data);
        assert!(vector::length<u64>(&sortedData) == 5, 0);
        assert!(*vector::borrow(&sortedData, 0) == 1, 0);
        assert!(*vector::borrow(&sortedData, 1) == 2, 0);
        assert!(*vector::borrow(&sortedData, 2) == 3, 0);
        assert!(*vector::borrow(&sortedData, 3) == 4, 0);
        assert!(*vector::borrow(&sortedData, 4) == 5, 0);
    }

    #[test]
    fun test_quick_sort_u128() {
        let data = vector<u128>[1, 3, 2, 5, 4];
        let sortedData = quick_sort(data);
        assert!(vector::length<u128>(&sortedData) == 5, 0);
        assert!(*vector::borrow(&sortedData, 0) == 1, 0);
        assert!(*vector::borrow(&sortedData, 1) == 2, 0);
        assert!(*vector::borrow(&sortedData, 2) == 3, 0);
        assert!(*vector::borrow(&sortedData, 3) == 4, 0);
        assert!(*vector::borrow(&sortedData, 4) == 5, 0);
    }

    #[test]
    fun test_quick_sort_decimal_value() {
        let data = vector<DecimalValue>[
            oracle_decimal_value::new(1000000, 6),
            oracle_decimal_value::new(3000000, 6),
            oracle_decimal_value::new(2000000, 6),
            oracle_decimal_value::new(5000000, 6),
            oracle_decimal_value::new(4000000, 6)];
        let sortedData = quick_sort(data);
        assert!(vector::length<DecimalValue>(&sortedData) == 5, 0);
        assert!(oracle_decimal_value::value(vector::borrow(&sortedData, 0)) == 1000000, 0);
        assert!(oracle_decimal_value::value(vector::borrow(&sortedData, 1)) == 2000000, 0);
        assert!(oracle_decimal_value::value(vector::borrow(&sortedData, 2)) == 3000000, 0);
        assert!(oracle_decimal_value::value(vector::borrow(&sortedData, 3)) == 4000000, 0);
        assert!(oracle_decimal_value::value(vector::borrow(&sortedData, 4)) == 5000000, 0);
    }

    #[test]
    fun test_quick_sort_decimal_value_different_decimal() {
        let data = vector<DecimalValue>[
            oracle_decimal_value::new(60000, 2),
            oracle_decimal_value::new(70000, 2),
            oracle_decimal_value::new(1000000, 6),
            oracle_decimal_value::new(3000000, 6),
            oracle_decimal_value::new(2000000, 6),
            oracle_decimal_value::new(5000000, 6),
            oracle_decimal_value::new(4000000, 6)];

        let sortedData = quick_sort(data);

        assert!(vector::length<DecimalValue>(&sortedData) == 7, 0);
        assert!(oracle_decimal_value::value(vector::borrow(&sortedData, 0)) == 1000000, 0);
        assert!(oracle_decimal_value::value(vector::borrow(&sortedData, 1)) == 2000000, 0);
        assert!(oracle_decimal_value::value(vector::borrow(&sortedData, 2)) == 3000000, 0);
        assert!(oracle_decimal_value::value(vector::borrow(&sortedData, 3)) == 4000000, 0);
        assert!(oracle_decimal_value::value(vector::borrow(&sortedData, 4)) == 5000000, 0);
        assert!(oracle_decimal_value::value(vector::borrow(&sortedData, 5)) == 60000, 0);
        assert!(oracle_decimal_value::value(vector::borrow(&sortedData, 6)) == 70000, 0);
    }
}
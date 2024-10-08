// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_framework::oracle {
    use std::option;
    use std::option::Option;
    use std::string;
    use std::string::String;
    use moveos_std::timestamp::now_milliseconds;
    use moveos_std::tx_context::sender;
    use moveos_std::object;
    use moveos_std::object::{Object, ObjectID};
    use moveos_std::table::{Self, Table};
    use moveos_std::event;
    use moveos_std::decimal_value;

    use rooch_framework::oracle_data::{Self, Data};

    const ErrorSenderNotOracle: u64 = 0;
    const ErrorTickerNotExists: u64 = 1;

    struct TablePlaceholder has key {
        _placeholder: bool,
    }

    struct SimpleOracle has store, key {
        id: Object<TablePlaceholder>,
        /// The address of the oracle.
        address: address,
        /// The name of the oracle.
        name: String,
        /// The description of the oracle.
        description: String,
        /// The URL of the oracle.
        url: String,
    }

    struct OracleAdminCap has key, store {
        oracle_id: ObjectID
    }

    struct StoredData<T: store> has copy, store, drop {
        value: T,
        sequence_number: u64,
        timestamp: u64,
        /// An identifier for the reading (for example real time of observation, or sequence number of observation on other chain).
        identifier: String,
    }

    struct NewOracleEvent has copy, drop {
        name: String,
        oracle_id: ObjectID,
        admin_id: ObjectID
    }

    public fun get_historical_data<K: copy + drop + store, V: store + copy>(
        oracle_obj: &Object<SimpleOracle>,
        ticker: String,
        archival_key: K
    ): Option<Data<V>> {
        let oracle = object::borrow(oracle_obj);
        string::append(&mut string::utf8(b"[historical] "), ticker);
        let historical_data: &Table<K, StoredData<V>> = object::borrow_field(&oracle.id, ticker);
        let StoredData { value, sequence_number, timestamp, identifier } = *table::borrow(
            historical_data,
            archival_key
        );
        option::some(oracle_data::new(value, ticker, sequence_number, timestamp, oracle.address, identifier))
    }

    public fun get_latest_data<T: store + copy>(oracle_obj: &Object<SimpleOracle>, ticker: String): Option<Data<T>> {
        let oracle = object::borrow(oracle_obj);
        if (!object::contains_field(&oracle.id, ticker)) {
            return option::none()
        };
        let data: &StoredData<T> = object::borrow_field(&oracle.id, ticker);
        let StoredData { value, sequence_number, timestamp, identifier } = *data;
        option::some(oracle_data::new(value, ticker, sequence_number, timestamp, oracle.address, identifier))
    }

    /// Create a new shared SimpleOracle object for publishing data.
    public entry fun create_entry(name: String, url: String, description: String) {
        let (oracle, admin_cap) = create(name, url, description);
        object::to_shared(oracle);
        object::transfer(admin_cap, sender())
    }

    /// Create a new SimpleOracle object for publishing data.
    public fun create(name: String, url: String, description: String): (Object<SimpleOracle>, Object<OracleAdminCap>) {
        let oracle = object::new(
            SimpleOracle {
                id: object::new(TablePlaceholder { _placeholder: false }), address: sender(
                ), name, description, url
            }
        );
        let oracle_id = object::id(&oracle);
        let admin_cap = object::new(OracleAdminCap {
            oracle_id
        });
        event::emit(
            NewOracleEvent {
                name,
                oracle_id,
                admin_id: object::id(&admin_cap)
            }
        );
        (oracle, admin_cap)
    }

    public fun submit_data<T: store + copy + drop>(
        oracle_obj: &mut Object<SimpleOracle>,
        ticker: String,
        value: T,
        identifier: String,
        admin_obj: &mut Object<OracleAdminCap>,
    ) {
        let timestamp = now_milliseconds();
        submit_data_with_timestamp(oracle_obj, ticker, value, identifier, timestamp, admin_obj); 
    }

    /// Submit data with timestamp.
    /// This function is used to submit data with a specific timestamp.
    /// The timestamp is the time from the oracle's data source.
    /// The timestamp is measured in milliseconds.
    public fun submit_data_with_timestamp<T: store + copy + drop>(
        oracle_obj: &mut Object<SimpleOracle>,
        ticker: String,
        value: T,
        identifier: String,
        timestamp: u64,
        admin_obj: &mut Object<OracleAdminCap>,
    ) {
        let oracle_id = object::id(oracle_obj);
        let admin_id = object::borrow(admin_obj).oracle_id;
        assert!(oracle_id == admin_id, ErrorSenderNotOracle);
        let oracle = object::borrow_mut(oracle_obj);

        let sequence_number = if (object::contains_field(&oracle.id, ticker)) {
            let old_data: StoredData<T> = object::remove_field(&mut oracle.id, ticker);
            old_data.sequence_number + 1
        }else {
            0
        };

        let new_data = StoredData {
            value,
            sequence_number,
            timestamp,
            identifier,
        };
        object::add_field(&mut oracle.id, ticker, new_data);
    }

    public entry fun submit_decimal_data(
        oracle_obj: &mut Object<SimpleOracle>,
        ticker: String,
        value: u256,
        decimal: u8,
        identifier: String,
        timestamp: u64,
        admin_obj: &mut Object<OracleAdminCap>
    ) {
        let decimal_value = decimal_value::new(value, decimal);
        submit_data_with_timestamp(oracle_obj, ticker, decimal_value, identifier, timestamp, admin_obj);
    }

    public fun archive_data<K: store + copy + drop, V: store + copy + drop>(
        oracle_obj: &mut Object<SimpleOracle>,
        ticker: String,
        archival_key: K,
        admin_obj: &mut Object<OracleAdminCap>,
    ) {
        let oracle_id = object::id(oracle_obj);
        let admin_id = object::borrow(admin_obj).oracle_id;
        assert!(oracle_id == admin_id, ErrorSenderNotOracle);
        let oracle = object::borrow_mut(oracle_obj);
        assert!(object::contains_field(&oracle.id, ticker), ErrorTickerNotExists);

        let latest_data: StoredData<V> = *object::borrow_mut_field(&mut oracle.id, ticker);

        string::append(&mut string::utf8(b"[historical] "), ticker);
        if (!object::contains_field(&oracle.id, ticker)) {
            let data_source = table::new<K, StoredData<V>>();
            object::add_field(&mut oracle.id, ticker, data_source);
        };
        let historical_data: &mut Table<K, StoredData<V>> = object::borrow_mut_field(&mut oracle.id, ticker);
        // Replace the old data in historical data if any.
        if (table::contains(historical_data, archival_key)) {
            table::remove(historical_data, archival_key);
        };
        table::add(historical_data, archival_key, latest_data);
    }
}
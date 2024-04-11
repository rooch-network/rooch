// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// Copyright (c) RoochDataImportMode
// SPDX-License-Identifier: Apache-2.0

module bitcoin_move::data_import_config{
    use moveos_std::object;

    const ErrorUnknownDataImportMode: u64 = 1;

    friend bitcoin_move::genesis;

    /// Bitcoin data import mode onchain configuration.
    struct DataImportConfig has key{
        data_import_mode: u8
    }

    public(friend) fun genesis_init(data_import_mode: u8){
        let obj = object::new_named_object(DataImportConfig{data_import_mode});
        object::to_shared(obj);
    }

    /// Get the current data import mode from the onchain configuration.
    public fun data_import_mode() : u8 {
        let id = object::named_object_id<DataImportConfig>();
        object::borrow(object::borrow_object<DataImportConfig>(id)).data_import_mode 
    }

    /// Currently, Move does not support enum types, so we use constants to represent the data import mode type.
    /// Bitcoin's none data import mode.
    const DATA_IMPORT_MODE_NONE: u8 = 0;
    public fun data_import_mode_none(): u8 {
        DATA_IMPORT_MODE_NONE
    }

    /// Bitcoin's utxo data import mode.
    const DATA_IMPORT_MODE_UTXO: u8 = 1;
    public fun data_import_mode_utxo(): u8 {
        DATA_IMPORT_MODE_UTXO
    }

    /// Bitcoin's ord data import mode.
    const DATA_IMPORT_MODE_ORD: u8 = 2;
    public fun data_import_mode_ord(): u8 {
        DATA_IMPORT_MODE_ORD
    }

    /// Bitcoin's full data import mode.
    /// All mode will process full data and indexer
    const DATA_IMPORT_MODE_FULL: u8 = 10;
    public fun data_import_mode_full(): u8 {
        DATA_IMPORT_MODE_FULL
    }

    public fun is_data_import_mode(data_import_mode: u8): bool {
        data_import_mode == DATA_IMPORT_MODE_UTXO || data_import_mode == DATA_IMPORT_MODE_ORD
    }

    public fun is_ord_mode(data_import_mode: u8): bool {
        data_import_mode == DATA_IMPORT_MODE_ORD || data_import_mode == DATA_IMPORT_MODE_FULL
    }
}

 module rooch_framework::genesis {
    use rooch_framework::account;
    use rooch_framework::block;
    use rooch_framework::chain_id;
    use rooch_framework::timestamp;
    use rooch_framework::governance;
    use rooch_framework::core_addresses;

    /// Genesis step 1: Initialize rooch genesis account and core modules on chain.
    fun initialize(
        _gas_schedule: vector<u8>,
        chain_id: u8,
        _initial_version: u64,
        epoch_interval_microsecs: u64,
        genesis_timestamp: u64,

        //TODO block reward

        //TODO consensus config
        _consensus_config: vector<u8>,

        //TODO vm config

        //TODO gas constants
    ) {
        // Initialize the rooch genesis account. This is the account where system resources and modules will be
        // deployed to. This will be entirely managed by on-chain governance and no entities have the key or privileges
        // to use this account.
        let (rooch_genesis_account, rooch_genesis_signer_cap) = account::create_framework_reserved_account(core_addresses::genesis_address());

        // transaction_validation::initialize(
        //     &rooch_genesis_account,
        //     b"script_prologue",
        //     b"module_prologue",
        //     b"multi_agent_script_prologue",
        //     b"epilogue",
        // );

        // Give the decentralized on-chain governance control over the core genesis account.
        governance::store_signer_cap(&rooch_genesis_account, core_addresses::genesis_address(), rooch_genesis_signer_cap);

        // consensus_config::initialize(&rooch_genesis_account, consensus_config);
        // version::initialize(&rooch_genesis_account, initial_version);
        // gas_schedule::initialize(&rooch_genesis_account, gas_schedule);

        chain_id::initialize(&rooch_genesis_account, chain_id);
        block::initialize(&rooch_genesis_account, epoch_interval_microsecs);
        timestamp::initialize(&rooch_genesis_account, genesis_timestamp);
    }
}

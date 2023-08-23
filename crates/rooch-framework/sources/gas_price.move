module rooch_framework::gas_price {

    /// Returns the gas price per unit of gas.
    public fun get_gas_price_per_unit(): u64 {
        //TODO we should provide a algorithm to cordanate the gas price based on the network throughput
        return 1
    }
}
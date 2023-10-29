// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module moveos_std::tx_result {

    /// The result of a transaction.
    /// The VM will put this struct in the TxContext after the transaction execution.
    /// We can get the result in the `post_execute` function.
    struct TxResult has copy, store, drop {
        /// The transaction is executed successfully or not.
        executed: bool,
        /// The gas used by the transaction.
        gas_used: u64,
    }

    public fun is_executed(self: &TxResult) : bool {
        self.executed
    }

    public fun gas_used(self: &TxResult) : u64 {
        self.gas_used
    }
}
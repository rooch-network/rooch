// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module bitcoin_move::transaction_validator {
    use bitcoin_move::bitcoin;

    /// The l1 tx already execute
    // const ErrorValidateL1TxAlreadyExist: u64 = 1001;

    /// Just using to get module signer
    struct TransactionValidatorPlaceholder {}

    /// This function is for Rooch to validate the l1 transaction.
    /// If validate fails, abort this function.
    public fun validate_l1_tx(
        tx_hash: address,
        _payload: vector<u8>
    ): bool {
        // If the l1 tx has been executed, then skip the tx.
        // And the validate result will be false instead of abort the function
        if(bitcoin::exist_l1_tx(tx_hash)){
            return false
        };

        // If validate fails, abort this function.

        true
    }
}

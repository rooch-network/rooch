// Origin source https://github.com/MystenLabs/sui/blob/598f106ef5fbdfbe1b644236f0caf46c94f4d1b7/crates/sui-framework/sources/tx_context.move#L24
// And do refactoring

module moveos_std::tx_context {

    friend moveos_std::object;

    /// Number of bytes in an tx hash (which will be the transaction digest)
    const TX_HASH_LENGTH: u64 = 32;

    /// Expected an tx hash of length 32, but found a different length
    const EBadTxHashLength: u64 = 0;


    /// Information about the transaction currently being executed.
    /// This cannot be constructed by a transaction--it is a privileged object created by
    /// the VM and passed in to the entrypoint of the transaction as `&mut TxContext`.
    struct TxContext has drop {
        /// The address of the user that signed the current transaction
        sender: address,
        /// Hash of the current transaction
        tx_hash: vector<u8>,
        /// Counter recording the number of fresh id's created while executing
        /// this transaction. Always 0 at the start of a transaction
        ids_created: u64
    }

    /// Return the address of the user that signed the current
    /// transaction
    public fun sender(self: &TxContext): address {
        self.sender
    } 

      /// Generate a new, globally unique object ID with version 0
    public(friend) fun new_object(ctx: &mut TxContext): address {
        let ids_created = ctx.ids_created;
        //Can we derive_id in Move?
        let id = derive_id(*&ctx.tx_hash, ids_created);
        ctx.ids_created = ids_created + 1;
        id
    }
    
    //TODO should support make signer with TxContext?
    // pub fun sender_signer(self: &TxContext): &signer {
    //     
    //}

    /// Return the hash of the current transaction
    public fun tx_hash(self: &TxContext): vector<u8> {
        self.tx_hash
    }

    /// Return the number of id's created by the current transaction.
    /// Hidden for now, but may expose later
    fun ids_created(self: &TxContext): u64 {
        self.ids_created
    }

    /// Should move this function to Object?
    /// Native function for deriving an ID via hash(tx_hash || ids_created)
    native fun derive_id(tx_hash: vector<u8>, ids_created: u64): address;

}

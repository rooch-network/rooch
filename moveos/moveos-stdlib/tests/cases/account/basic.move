//# init --addresses genesis=0x1 bob=0x42

//TODO create account by faucet

//create account by bob self
//# run --signers genesis
script {
    use rooch_framework::account;
    use moveos_std::storage_context::StorageContext;
    fun main(ctx: &mut StorageContext, _sender: signer) {
        account::create_account_entry(ctx, @bob);
    }
}

//check
//# run --signers bob
script {
    use rooch_framework::account;
    use moveos_std::storage_context::StorageContext;
    fun main(ctx: &mut StorageContext, _sender: signer) {
        assert!(account::exists_at(ctx, @bob), 0);
    }
}

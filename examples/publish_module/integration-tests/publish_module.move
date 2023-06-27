//# init --addresses genesis=0x1

//create account by bob self
//# run --signers genesis
script {
    use rooch_examples::publish_module;
    use moveos_std::storage_context::StorageContext;

    fun main(ctx: &mut StorageContext,  account: &signer) {
        publish_module::publish_modules_entry(ctx,  account);
    }
}
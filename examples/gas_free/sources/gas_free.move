module rooch_examples::gas_payer {
    use moveos_std::storage_context::StorageContext;

    fun gas_validate_function(_storage_ctx: &StorageContext): bool{
        true
    }

    fun gas_charge_post_function(_storage_ctx: &mut StorageContext, _gas_used: u128) {
    }

    #[gas_free(gas_validate=gas_validate_function, gas_charge_post=gas_charge_post_function)]
    public entry fun play(_ctx: &mut StorageContext, _sender: &signer){
    }
}

module rooch_examples::gas_payer {
    use moveos_std::context::Context;

    fun gas_validate_function(_ctx: &Context): bool{
        true
    }

    fun gas_charge_post_function(_ctx: &mut Context, _gas_used: u128) {
    }

    #[gas_free(gas_validate=gas_validate_function, gas_charge_post=gas_charge_post_function)]
    public entry fun play(_ctx: &mut Context, _sender: &signer){
    }
}

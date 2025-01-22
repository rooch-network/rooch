module rooch_dex::admin_util {

    use moveos_std::object::{Object};
    use moveos_std::signer;
    use app_admin::admin::AdminCap;
    use rooch_dex::liquid_xp::LiquidXP;
    use rooch_dex::liquidity_incentive::{Self, FarmingAsset};

    struct Witness{}

    //any account with AdminCap can call this function to create LiquidXP incentive
    public entry fun create_xp_incentive_pool<X: key+store, Y: key+store>(
        release_per_second: u128,
        coin_amount: u256,
        start_time_in_seconds: u64,
        _admin_cap: &mut Object<AdminCap>,
    ){
        let module_signer = signer::module_signer<Witness>();
        liquidity_incentive::create_pool<X, Y, LiquidXP>(&module_signer, release_per_second, coin_amount, start_time_in_seconds);
    }

    public entry fun add_xp_incentive<X: key+store, Y: key+store>(
        farming_asset_obj: &mut Object<FarmingAsset<X, Y, LiquidXP>>,
        amount: u256,
        _admin_cap: &mut Object<AdminCap>,
    ){
        let module_signer = signer::module_signer<Witness>();
        liquidity_incentive::add_incentive(&module_signer, farming_asset_obj, amount);
    }
}
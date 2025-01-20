module rooch_dex::swap {
    use std::signer;
    use std::option::none;
    use std::string;
    use std::u128;
    use app_admin::admin;
    use moveos_std::timestamp;
    use moveos_std::account::borrow_mut_resource;
    use rooch_dex::swap_utils;
    use rooch_framework::account_coin_store;
    use moveos_std::event;
    use moveos_std::type_info;
    use rooch_framework::coin;
    use moveos_std::signer::module_signer;
    use moveos_std::tx_context::sender;
    use moveos_std::object;
    use moveos_std::account;
    use rooch_framework::coin::{CoinInfo, coin_info, symbol_by_type, supply_by_type};
    use moveos_std::object::{Object, ObjectID};
    use rooch_framework::coin_store::{CoinStore, balance, deposit, withdraw};
    use rooch_framework::coin_store;
    #[test_only]
    use moveos_std::object::to_shared;
    #[test_only]
    use rooch_framework::coin::Coin;



    friend rooch_dex::router;

    const RESOURCE_ACCOUNT: address = @rooch_dex;
    const MINIMUM_LIQUIDITY: u128 = 1000;
    const MAX_COIN_NAME_LENGTH: u64 = 32;

    // List of errors
    const ErrorAlreadyExists: u64 = 1;
    const ErrorTokenPairNotOpen: u64 = 2;
    const ErrorInsufficientLiquidityAmount: u64 = 3;
    const ErrorInsufficientTokenAmount: u64 = 4;
    const ErrorInsufficientLiquidity: u64 = 5;
    const ErrorInvalidAmount: u64 = 6;
    const ErrorLiquidityBurned: u64 = 7;
    const ErrorOutputTokenAmount: u64 = 8;
    const ErrorInputTokenAmount: u64 = 9;
    const ErrorKValue: u64 = 10;
    const ErrorWithdrawFee: u64 = 11;

    const PRECISION: u64 = 10000;

    const MAX_U128: u128 = 340282366920938463463374607431768211455;

    struct LPToken<phantom X: key+store, phantom Y: key+store> has key, store {}

    struct TokenPair<phantom X: key+store, phantom Y: key+store> has key {
        creator: address,
        fee: Object<CoinStore<LPToken<X, Y>>>,
        k_last: u128,
        balance_x: Object<CoinStore<X>>,
        balance_y: Object<CoinStore<Y>>,
        is_open: bool
    }

    struct TokenPairReserve<phantom X: key+store, phantom Y: key+store> has key {
        reserve_x: u64,
        reserve_y: u64,
        block_timestamp_last: u64
    }

    struct PairCreatedEvent has drop, store, copy {
        user: address,
        token_x: string::String,
        token_y: string::String
    }

    struct AddLiquidityEvent<phantom X: key+store, phantom Y: key+store> has drop, store, copy {
        user: address,
        amount_x: u64,
        amount_y: u64,
        liquidity: u64,
        fee: u64
    }

    struct RemoveLiquidityEvent<phantom X: key+store, phantom Y: key+store> has drop, store, copy {
        user: address,
        liquidity: u64,
        amount_x: u64,
        amount_y: u64,
        fee: u64
    }

    struct SwapEvent<phantom X: key+store, phantom Y: key+store> has drop, store, copy {
        user: address,
        amount_x_in: u64,
        amount_y_in: u64,
        amount_x_out: u64,
        amount_y_out: u64
    }

    struct RoochDexCap has key {}

    fun init() {
        object::transfer_extend(object::new_named_object(RoochDexCap{}), sender())
    }


    public entry fun create_admin(_admin: &mut Object<admin::AdminCap>, receiver: address){
        let new_admin = object::new(RoochDexCap{});
        object::transfer_extend(new_admin, receiver)
    }

    public entry fun delete_admin(_admin: &mut Object<admin::AdminCap>, admin_id: ObjectID){
        let admin_obj = object::take_object_extend<RoochDexCap>(admin_id);
        let RoochDexCap{} = object::remove(admin_obj);
    }

    /// Create the specified coin pair
    public(friend) fun create_pair<X:key+store, Y:key+store>(
        sender: &signer,
    ) {
        assert!(!is_pair_created<X, Y>(), ErrorAlreadyExists);

        let sender_addr = signer::address_of(sender);
        let resource_signer = module_signer<RoochDexCap>();

        let lp_name: string::String = string::utf8(b"RoochDex-");
        let name_x = symbol_by_type<X>();
        let name_y = symbol_by_type<Y>();
        string::append(&mut lp_name, name_x);
        string::append_utf8(&mut lp_name, b"-");
        string::append(&mut lp_name, name_y);
        string::append_utf8(&mut lp_name, b"-LP");
        if (string::length(&lp_name) > MAX_COIN_NAME_LENGTH) {
            lp_name = string::utf8(b"RoochDex LPs");
        };

        let coin_info = coin::register_extend<LPToken<X, Y>>(
            lp_name,
            string::utf8(b"RDex-LP"),
            none(),
            8,
        );

        account::move_resource_to<TokenPairReserve<X, Y>>(
            &resource_signer,
            TokenPairReserve {
                reserve_x: 0,
                reserve_y: 0,
                block_timestamp_last: 0
            }
        );

        account::move_resource_to<TokenPair<X, Y>>(
            &resource_signer,
            TokenPair {
                creator: sender_addr,
                fee: coin_store::create_coin_store(),
                k_last: 0,
                balance_x: coin_store::create_coin_store(),
                balance_y: coin_store::create_coin_store(),
                is_open: true
            }
        );

        // pair created event
        let token_x = type_info::type_name<X>();
        let token_y = type_info::type_name<Y>();

        event::emit<PairCreatedEvent>(
            PairCreatedEvent {
                user: sender_addr,
                token_x,
                token_y
            }
        );

        object::to_shared(coin_info);
    }


    public fun is_pair_created<X:key+store, Y:key+store>(): bool {
        account::exists_resource<TokenPair<X, Y>>(RESOURCE_ACCOUNT)
    }

    /// Obtain the LP token balance of `addr`.
    /// This method can only be used to check other users' balance.
    public fun lp_balance<X:key+store, Y:key+store>(addr: address): u256 {
        account_coin_store::balance<LPToken<X, Y>>(addr)
    }

    /// Get the total supply of LP Tokens
    public fun total_lp_supply<X:key+store, Y:key+store>(): u128 {
        (supply_by_type<LPToken<X, Y>>() as u128)
    }

    /// Get the current reserves of T0 and T1 with the latest updated timestamp
    public fun token_reserves<X:key+store, Y:key+store>(): (u64, u64, u64) {
        let reserve = account::borrow_resource<TokenPairReserve<X, Y>>(RESOURCE_ACCOUNT);
        (
            reserve.reserve_x,
            reserve.reserve_y,
            reserve.block_timestamp_last
        )
    }

    /// The amount of balance currently in pools of the liquidity pair
    public fun token_balances<X:key+store, Y:key+store>(): (u64, u64) {
        let token_pair =
            account::borrow_resource<TokenPair<X, Y>>(RESOURCE_ACCOUNT);
        (
            (balance(&token_pair.balance_x) as u64),
            (balance(&token_pair.balance_y) as u64)
        )
    }


    // ===================== Update functions ======================
    /// Add more liquidity to token types. This method explicitly assumes the
    /// min of both tokens are 0.
    public(friend) fun add_liquidity<X:key+store, Y:key+store>(
        sender: &signer,
        amount_x: u64,
        amount_y: u64,
        coin_info_id: ObjectID
    ): (u64, u64, u64) {
        let coin_info = object::borrow_mut_object_shared(coin_info_id);
        let (a_x, a_y, coin_lp, fee, coin_left_x, coin_left_y) = add_liquidity_direct(account_coin_store::withdraw<X>(sender,
            (amount_x as u256)
        ), account_coin_store::withdraw<Y>(sender, (amount_y as u256)), coin_info);
        let sender_addr = signer::address_of(sender);
        let lp_amount = (coin::value(&coin_lp) as u64);
        assert!(lp_amount > 0, ErrorInsufficientLiquidity);
        account_coin_store::deposit(sender_addr, coin_lp);
        account_coin_store::deposit(sender_addr, coin_left_x);
        account_coin_store::deposit(sender_addr, coin_left_y);

        event::emit<AddLiquidityEvent<X, Y>>(
            AddLiquidityEvent<X, Y> {
                user: sender_addr,
                amount_x: a_x,
                amount_y: a_y,
                liquidity: lp_amount,
                fee,
            }
        );

        (a_x, a_y, lp_amount)
    }

    public(friend) fun add_swap_event<X:key+store, Y:key+store>(
        sender: &signer,
        amount_x_in: u64,
        amount_y_in: u64,
        amount_x_out: u64,
        amount_y_out: u64
    ) {
        let sender_addr = signer::address_of(sender);
        event::emit<SwapEvent<X, Y>>(
            SwapEvent<X, Y> {
                user: sender_addr,
                amount_x_in,
                amount_y_in,
                amount_x_out,
                amount_y_out
            }
        );
    }

    public(friend) fun add_swap_event_with_address<X:key+store, Y:key+store>(
        sender_addr: address,
        amount_x_in: u64,
        amount_y_in: u64,
        amount_x_out: u64,
        amount_y_out: u64
    ) {
        event::emit<SwapEvent<X, Y>>(
            SwapEvent<X, Y> {
                user: sender_addr,
                amount_x_in,
                amount_y_in,
                amount_x_out,
                amount_y_out
            }
        );
    }

    /// Add more liquidity to token types. This method explicitly assumes the
    /// min of both tokens are 0.
    fun add_liquidity_direct<X:key+store, Y:key+store>(
        x: coin::Coin<X>,
        y: coin::Coin<Y>,
        coin_info: &mut Object<CoinInfo<LPToken<X, Y>>>
    ): (u64, u64, coin::Coin<LPToken<X, Y>>, u64, coin::Coin<X>, coin::Coin<Y>){
        let amount_x = (coin::value(&x) as u64);
        let amount_y = (coin::value(&y) as u64);
        let (reserve_x, reserve_y, _) = token_reserves<X, Y>();
        let (a_x, a_y) = if (reserve_x == 0 && reserve_y == 0) {
            (amount_x, amount_y)
        } else {
            let amount_y_optimal = swap_utils::quote(amount_x, reserve_x, reserve_y);
            if (amount_y_optimal <= amount_y) {
                (amount_x, amount_y_optimal)
            } else {
                let amount_x_optimal = swap_utils::quote(amount_y, reserve_y, reserve_x);
                assert!(amount_x_optimal <= amount_x, ErrorInvalidAmount);
                (amount_x_optimal, amount_y)
            }
        };

        assert!(a_x <= amount_x, ErrorInsufficientTokenAmount);
        assert!(a_y <= amount_y, ErrorInsufficientTokenAmount);

        let left_x = coin::extract(&mut x, (amount_x - a_x as u256));
        let left_y = coin::extract(&mut y, (amount_y - a_y as u256));
        deposit_x<X, Y>(x);
        deposit_y<X, Y>(y);
        let (lp, fee) = mint<X, Y>(coin_info);
        (a_x, a_y, lp, fee, left_x, left_y)
    }

    /// Remove liquidity to token types.
    public(friend) fun remove_liquidity<X:key+store, Y:key+store>(
        sender: &signer,
        liquidity: u64,
        coin_info_id: ObjectID
    ): (u64, u64) {
        let coin_info = object::borrow_mut_object_shared(coin_info_id);
        let coins = account_coin_store::withdraw<LPToken<X, Y>>(sender, (liquidity as u256));
        let (coins_x, coins_y, fee) = remove_liquidity_direct<X, Y>(coins, coin_info);
        let amount_x = (coin::value(&coins_x) as u64);
        let amount_y = (coin::value(&coins_y) as u64);
        let sender_addr = signer::address_of(sender);
        account_coin_store::deposit<X>(sender_addr, coins_x);
        account_coin_store::deposit<Y>(sender_addr, coins_y);
        // event
        event::emit<RemoveLiquidityEvent<X, Y>>(
            RemoveLiquidityEvent<X, Y> {
                user: sender_addr,
                amount_x,
                amount_y,
                liquidity,
                fee
            }
        );
        (amount_x, amount_y)
    }

    /// Remove liquidity to token types.
    fun remove_liquidity_direct<X:key+store, Y:key+store>(
        liquidity: coin::Coin<LPToken<X, Y>>,
        coin_info: &mut Object<CoinInfo<LPToken<X, Y>>>
    ): (coin::Coin<X>, coin::Coin<Y>, u64){
        burn<X, Y>(liquidity, coin_info)
    }

    /// Swap X to Y, X is in and Y is out. This method assumes amount_out_min is 0
    public(friend) fun swap_exact_x_to_y<X:key+store, Y:key+store>(
        sender: &signer,
        amount_in: u64,
        to: address
    ): u64{
        let coins = account_coin_store::withdraw<X>(sender, (amount_in as u256));
        let (coins_x_out, coins_y_out) = swap_exact_x_to_y_direct<X, Y>(coins);
        let amount_out = coin::value(&coins_y_out);
        coin::destroy_zero(coins_x_out); // or others ways to drop `coins_x_out`
        account_coin_store::deposit(to, coins_y_out);
        (amount_out as u64)
    }

    /// Swap X to Y, X is in and Y is out. This method assumes amount_out_min is 0
    public(friend) fun swap_exact_x_to_y_direct<X:key+store, Y:key+store>(
        coins_in: coin::Coin<X>
    ): (coin::Coin<X>, coin::Coin<Y>){
        let amount_in = coin::value<X>(&coins_in);
        deposit_x<X, Y>(coins_in);
        let (rin, rout, _) = token_reserves<X, Y>();
        let amount_out = swap_utils::get_amount_out((amount_in as u64), rin, rout);
        let (coins_x_out, coins_y_out) = swap<X, Y>(0, amount_out);
        assert!(coin::value<X>(&coins_x_out) == 0, ErrorOutputTokenAmount);
        (coins_x_out, coins_y_out)
    }

    public(friend) fun swap_x_to_exact_y<X:key+store, Y:key+store>(
        sender: &signer,
        amount_in: u64,
        amount_out: u64,
        to: address
    ): u64 {
        let coins_in = account_coin_store::withdraw<X>(sender, (amount_in as u256));
        let (coins_x_out, coins_y_out) = swap_x_to_exact_y_direct<X, Y>(coins_in, amount_out);
        coin::destroy_zero(coins_x_out); // or others ways to drop `coins_x_out`
        account_coin_store::deposit(to, coins_y_out);
        amount_in
    }

    public(friend) fun swap_x_to_exact_y_direct<X:key+store, Y:key+store>(
        coins_in: coin::Coin<X>, amount_out: u64
    ): (coin::Coin<X>, coin::Coin<Y>) {
        deposit_x<X, Y>(coins_in);
        let (coins_x_out, coins_y_out) = swap<X, Y>(0, amount_out);
        assert!(coin::value<X>(&coins_x_out) == 0, ErrorOutputTokenAmount);
        (coins_x_out, coins_y_out)
    }

    /// Swap Y to X, Y is in and X is out. This method assumes amount_out_min is 0
    public(friend) fun swap_exact_y_to_x<X:key+store, Y:key+store>(
        sender: &signer,
        amount_in: u64,
        to: address
    ): u64{
        let coins = account_coin_store::withdraw<Y>(sender, (amount_in as u256));
        let (coins_x_out, coins_y_out) = swap_exact_y_to_x_direct<X, Y>(coins);
        let amount_out = coin::value<X>(&coins_x_out);
        account_coin_store::deposit(to, coins_x_out);
        coin::destroy_zero(coins_y_out); // or others ways to drop `coins_y_out`
        (amount_out as u64)
    }

    public(friend) fun swap_y_to_exact_x<X:key+store, Y:key+store>(
        sender: &signer,
        amount_in: u64,
        amount_out: u64,
        to: address
    ): u64 {
        let coins_in = account_coin_store::withdraw<Y>(sender, (amount_in as u256));
        let (coins_x_out, coins_y_out) = swap_y_to_exact_x_direct<X, Y>(coins_in, amount_out);
        account_coin_store::deposit(to, coins_x_out);
        coin::destroy_zero(coins_y_out); // or others ways to drop `coins_y_out`
        amount_in
    }

    public(friend) fun swap_y_to_exact_x_direct<X:key+store, Y:key+store>(
        coins_in: coin::Coin<Y>, amount_out: u64
    ): (coin::Coin<X>, coin::Coin<Y>) {
        deposit_y<X, Y>(coins_in);
        let (coins_x_out, coins_y_out) = swap<X, Y>(amount_out, 0);
        assert!(coin::value<Y>(&coins_y_out) == 0, ErrorOutputTokenAmount);
        (coins_x_out, coins_y_out)
    }

    /// Swap Y to X, Y is in and X is out. This method assumes amount_out_min is 0
    public(friend) fun swap_exact_y_to_x_direct<X:key+store, Y:key+store>(
        coins_in: coin::Coin<Y>
    ): (coin::Coin<X>, coin::Coin<Y>) {
        let amount_in = coin::value<Y>(&coins_in);
        deposit_y<X, Y>(coins_in);
        let (rout, rin, _) = token_reserves<X, Y>();
        let amount_out = swap_utils::get_amount_out((amount_in as u64), rin, rout);
        let (coins_x_out, coins_y_out) = swap<X, Y>(amount_out, 0);
        assert!(coin::value<Y>(&coins_y_out) == 0, ErrorOutputTokenAmount);
        (coins_x_out, coins_y_out)
    }

    fun swap<X:key+store, Y:key+store>(
        amount_x_out: u64,
        amount_y_out: u64
    ): (coin::Coin<X>, coin::Coin<Y>){
        assert!(amount_x_out > 0 || amount_y_out > 0, ErrorOutputTokenAmount);

        let reserves = account::borrow_mut_resource<TokenPairReserve<X, Y>>(RESOURCE_ACCOUNT);
        assert!(amount_x_out < reserves.reserve_x && amount_y_out < reserves.reserve_y, ErrorInsufficientLiquidity);

        let token_pair = account::borrow_mut_resource<TokenPair<X, Y>>(RESOURCE_ACCOUNT);
        assert!(token_pair.is_open, ErrorTokenPairNotOpen);
        let coins_x_out = coin::zero<X>();
        let coins_y_out = coin::zero<Y>();
        if (amount_x_out > 0) coin::merge(&mut coins_x_out, withdraw_x((amount_x_out as u256), token_pair));
        if (amount_y_out > 0) coin::merge(&mut coins_y_out, withdraw_y((amount_y_out as u256), token_pair));
        let (balance_x, balance_y) = token_balances<X, Y>();

        let amount_x_in = if (balance_x > reserves.reserve_x - amount_x_out) {
            balance_x - (reserves.reserve_x - amount_x_out)
        } else { 0 };
        let amount_y_in = if (balance_y > reserves.reserve_y - amount_y_out) {
            balance_y - (reserves.reserve_y - amount_y_out)
        } else { 0 };

        assert!(amount_x_in > 0 || amount_y_in > 0, ErrorInputTokenAmount);

        let prec = (PRECISION as u128);
        let balance_x_adjusted = (balance_x as u128) * prec - (amount_x_in as u128) * 25u128;
        let balance_y_adjusted = (balance_y as u128) * prec - (amount_y_in as u128) * 25u128;
        let reserve_x_adjusted = (reserves.reserve_x as u128) * prec;
        let reserve_y_adjusted = (reserves.reserve_y as u128) * prec;

        let compare_result = if(balance_x_adjusted > 0 && reserve_x_adjusted > 0 && MAX_U128 / balance_x_adjusted > balance_y_adjusted && MAX_U128 / reserve_x_adjusted > reserve_y_adjusted){
            balance_x_adjusted * balance_y_adjusted >= reserve_x_adjusted * reserve_y_adjusted
        }else{
            let p: u256 = (balance_x_adjusted as u256) * (balance_y_adjusted as u256);
            let k: u256 = (reserve_x_adjusted as u256) * (reserve_y_adjusted as u256);
            p >= k
        };
        assert!(compare_result, ErrorKValue);

        update(balance_x, balance_y, reserves);

        (coins_x_out, coins_y_out)
    }

    /// Mint LP Token.
    /// This low-level function should be called from a contract which performs important safety checks
    fun mint<X:key+store, Y:key+store>(
        coin_info: &mut Object<CoinInfo<LPToken<X, Y>>>
    ): (coin::Coin<LPToken<X, Y>>, u64) {
        let token_pair = borrow_mut_resource<TokenPair<X, Y>>(RESOURCE_ACCOUNT);
        assert!(token_pair.is_open, ErrorTokenPairNotOpen);
        let (balance_x, balance_y) = (balance(&token_pair.balance_x), balance(&token_pair.balance_y));
        let reserves = borrow_mut_resource<TokenPairReserve<X, Y>>(RESOURCE_ACCOUNT);
        let amount_x = (balance_x as u128) - (reserves.reserve_x as u128);
        let amount_y = (balance_y as u128) - (reserves.reserve_y as u128);

        let fee = mint_fee<X, Y>(reserves.reserve_x, reserves.reserve_y, token_pair, coin_info);

        //Need to add fee amount which have not been mint.
        let total_supply = total_lp_supply<X, Y>();
        let liquidity = if (total_supply == 0u128) {
            let sqrt = u128::sqrt(amount_x * amount_y);
            assert!(sqrt > MINIMUM_LIQUIDITY, ErrorInsufficientLiquidityAmount);
            let l = sqrt - MINIMUM_LIQUIDITY;
            // permanently lock the first MINIMUM_LIQUIDITY tokens
            mint_lp_to<X, Y>(RESOURCE_ACCOUNT, (MINIMUM_LIQUIDITY as u64), coin_info);
            l
        } else {
            let liquidity = u128::min(amount_x * total_supply / (reserves.reserve_x as u128), amount_y * total_supply / (reserves.reserve_y as u128));
            assert!(liquidity > 0u128, ErrorInsufficientLiquidityAmount);
            liquidity
        };


        let lp = mint_lp<X, Y>((liquidity as u64), coin_info);

        update<X, Y>((balance_x as u64), (balance_y as u64), reserves);

        token_pair.k_last = (reserves.reserve_x as u128) * (reserves.reserve_y as u128);

        (lp, fee)
    }

    fun burn<X:key+store, Y:key+store>(lp_tokens: coin::Coin<LPToken<X, Y>>, coin_info: &mut Object<CoinInfo<LPToken<X, Y>>>): (coin::Coin<X>, coin::Coin<Y>, u64){
        let token_pair = account::borrow_mut_resource<TokenPair<X, Y>>(RESOURCE_ACCOUNT);
        assert!(token_pair.is_open, ErrorTokenPairNotOpen);
        let reserves = account::borrow_mut_resource<TokenPairReserve<X, Y>>(RESOURCE_ACCOUNT);
        let liquidity = coin::value(&lp_tokens);

        let fee = mint_fee<X, Y>(reserves.reserve_x, reserves.reserve_y, token_pair, coin_info);

        //Need to add fee amount which have not been mint.
        let total_lp_supply = total_lp_supply<X, Y>();
        let amount_x = ((coin_store::balance(&token_pair.balance_x) as u128) * (liquidity as u128) / total_lp_supply as u256);
        let amount_y = ((coin_store::balance(&token_pair.balance_x) as u128) * (liquidity as u128) / total_lp_supply as u256);
        assert!(amount_x > 0 && amount_y > 0, ErrorLiquidityBurned);
        coin::burn<LPToken<X, Y>>(coin_info, lp_tokens);

        let w_x = withdraw_x(amount_x, token_pair);
        let w_y = withdraw_y(amount_y, token_pair);

        update((coin_store::balance(&token_pair.balance_x) as u64), (coin_store::balance(&token_pair.balance_y) as u64), reserves);

        token_pair.k_last = (reserves.reserve_x as u128) * (reserves.reserve_y as u128);

        (w_x, w_y, fee)
    }

    fun update<X:key+store, Y:key+store>(balance_x: u64, balance_y: u64, reserve: &mut TokenPairReserve<X, Y>) {
        let block_timestamp = timestamp::now_seconds();

        reserve.reserve_x = balance_x;
        reserve.reserve_y = balance_y;
        reserve.block_timestamp_last = block_timestamp;
    }

    /// Mint LP Tokens to account
    fun mint_lp_to<X:key+store, Y:key+store>(
        to: address,
        amount: u64,
        mint_cap: &mut Object<CoinInfo<LPToken<X, Y>>>,
    ) {
        let coins = coin::mint<LPToken<X, Y>>(mint_cap, (amount as u256));
        account_coin_store::deposit(to, coins);
    }

    /// Mint LP Tokens to account
    fun mint_lp<X:key+store, Y:key+store>(amount: u64, mint_cap: &mut Object<CoinInfo<LPToken<X, Y>>>): coin::Coin<LPToken<X, Y>> {
        coin::mint<LPToken<X, Y>>(mint_cap, (amount as u256))
    }

    fun deposit_x<X:key+store, Y:key+store>(amount: coin::Coin<X>){
        let token_pair =
            borrow_mut_resource<TokenPair<X, Y>>(RESOURCE_ACCOUNT);
        deposit(&mut token_pair.balance_x, amount);
    }

    fun deposit_y<X:key+store, Y:key+store>(amount: coin::Coin<Y>) {
        let token_pair =
            borrow_mut_resource<TokenPair<X, Y>>(RESOURCE_ACCOUNT);
        deposit(&mut token_pair.balance_y, amount);
    }

    fun withdraw_x<X:key+store, Y:key+store>(amount: u256, token_pair: &mut TokenPair<X, Y>): coin::Coin<X> {
        assert!(balance(&token_pair.balance_x) > amount, ErrorInsufficientTokenAmount);
        withdraw(&mut token_pair.balance_x, amount)
    }

    fun withdraw_y<X:key+store, Y:key+store>(amount: u256, token_pair: &mut TokenPair<X, Y>): coin::Coin<Y> {
        assert!(balance(&token_pair.balance_y) > amount, ErrorInsufficientTokenAmount);
        withdraw(&mut token_pair.balance_y, amount)
    }

    fun mint_fee<X:key+store, Y:key+store>(reserve_x: u64, reserve_y: u64, token_pair: &mut TokenPair<X, Y>, coin_info: &mut Object<CoinInfo<LPToken<X, Y>>>): u64 {
        let fee = 0u64;
        if (token_pair.k_last != 0) {
            let root_k = u128::sqrt((reserve_x as u128) * (reserve_y as u128));
            let root_k_last = u128::sqrt(token_pair.k_last);
            if (root_k > root_k_last) {
                let numerator = total_lp_supply<X, Y>() * (root_k - root_k_last) * 8u128;
                let denominator = root_k_last * 17u128 + (root_k * 8u128);
                let liquidity = numerator / denominator;
                fee = (liquidity as u64);
                if (fee > 0) {
                    let coin = mint_lp(fee, coin_info);
                    deposit(&mut token_pair.fee, coin);
                }
            };
        };

        fee
    }

    public entry fun withdraw_fee<X:key+store, Y:key+store>(admin_cap: &mut Object<RoochDexCap>){
        if (swap_utils::sort_token_type<X, Y>()) {
            let token_pair = account::borrow_mut_resource<TokenPair<X, Y>>(RESOURCE_ACCOUNT);
            assert!(balance(&token_pair.fee) > 0, ErrorWithdrawFee);
            let fee = balance(&token_pair.fee);
            let coin = withdraw(&mut token_pair.fee, fee);
            account_coin_store::deposit(object::owner(admin_cap), coin);
        } else {
            let token_pair = account::borrow_mut_resource<TokenPair<Y, X>>(RESOURCE_ACCOUNT);
            assert!(balance(&token_pair.fee) > 0, ErrorWithdrawFee);
            let fee = balance(&token_pair.fee);
            let coin = withdraw(&mut token_pair.fee, fee);
            account_coin_store::deposit(object::owner(admin_cap), coin);
        };
    }

    public entry fun update_token_pair_status<X:key+store, Y:key+store>(_admin_cap: &mut Object<RoochDexCap>, status: bool){
        if (swap_utils::sort_token_type<X, Y>()) {
            let token_pair = account::borrow_mut_resource<TokenPair<X, Y>>(RESOURCE_ACCOUNT);
            token_pair.is_open = status
        } else {
            let token_pair = account::borrow_mut_resource<TokenPair<Y, X>>(RESOURCE_ACCOUNT);
            token_pair.is_open = status
        };
    }

    #[test_only]
    struct TestCoinX has key, store{}
    #[test_only]
    struct TestCoinY has key, store{}

    #[test_only]
    public fun init_lp_for_test(amount: u256) : Coin<LPToken<TestCoinX, TestCoinY>> {
        let coin_info = coin::register_extend<LPToken<TestCoinX, TestCoinY>>(
            string::utf8(b"RoochDex LPs"),
            string::utf8(b"RDex-LP"),
            none(),
            8,
        );
        let lp_coin = coin::mint(&mut coin_info, amount);
        to_shared(coin_info);
        return lp_coin
    }
}

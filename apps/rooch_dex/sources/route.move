module rooch_dex::router {
    use rooch_dex::swap;
    use std::signer;
    use std::signer::address_of;
    use rooch_framework::account_coin_store;
    use moveos_std::object::ObjectID;
    use rooch_framework::coin;
    use rooch_dex::swap_utils;



    const ErrorInsufficientOutputAmount: u64 = 1;
    const ErrorInsufficientInputAmount: u64 = 2;
    const ErrorInsufficientXAmount: u64 = 3;
    const ErrorInsufficientYAmount: u64 = 4;
    const ErrorTokenPairNotExist: u64 = 5;
    const ErrorTokenPairAlreadyExist: u64 = 6;


    public entry fun create_token_pair<X:key+store, Y:key+store>(
        sender: &signer,
        amount_x_desired: u64,
        amount_y_desired: u64,
        amount_x_min: u64,
        amount_y_min: u64,
    ) {
        assert!(!(swap::is_pair_created<X, Y>() || swap::is_pair_created<Y, X>()), ErrorTokenPairAlreadyExist);
        if (swap_utils::sort_token_type<X, Y>()) {
            let coin_info_id = swap::create_pair<X, Y>(sender);
            add_liquidity<X, Y>(sender, amount_x_desired, amount_y_desired, amount_x_min, amount_y_min, coin_info_id);
        } else {
            let coin_info_id = swap::create_pair<Y, X>(sender);
            add_liquidity<Y, X>(sender, amount_x_desired, amount_y_desired, amount_x_min, amount_y_min, coin_info_id);
        };

    }


    public entry fun add_liquidity<X:key+store, Y:key+store>(
        sender: &signer,
        amount_x_desired: u64,
        amount_y_desired: u64,
        amount_x_min: u64,
        amount_y_min: u64,
        coin_info: ObjectID,
    ) {

        let amount_x;
        let amount_y;
        let _lp_amount;
        if (swap_utils::sort_token_type<X, Y>()) {
            (amount_x, amount_y, _lp_amount) = swap::add_liquidity<X, Y>(sender, amount_x_desired, amount_y_desired, coin_info);
            assert!(amount_x >= amount_x_min, ErrorInsufficientXAmount);
            assert!(amount_y >= amount_y_min, ErrorInsufficientYAmount);
        } else {
            (amount_y, amount_x, _lp_amount) = swap::add_liquidity<Y, X>(sender, amount_y_desired, amount_x_desired, coin_info);
            assert!(amount_x >= amount_x_min, ErrorInsufficientXAmount);
            assert!(amount_y >= amount_y_min, ErrorInsufficientYAmount);
        };
    }

    fun assert_token_pair_created<X:key+store, Y:key+store>(){
        assert!(swap::is_pair_created<X, Y>() || swap::is_pair_created<Y, X>(), ErrorTokenPairNotExist);
    }

    /// Remove Liquidity
    public entry fun remove_liquidity<X:key+store, Y:key+store>(
        sender: &signer,
        liquidity: u64,
        amount_x_min: u64,
        amount_y_min: u64,
        coin_info: ObjectID
    ) {
        assert_token_pair_created<X, Y>();
        let amount_x;
        let amount_y;
        if (swap_utils::sort_token_type<X, Y>()) {
            (amount_x, amount_y) = swap::remove_liquidity<X, Y>(sender, liquidity, coin_info);
            assert!(amount_x >= amount_x_min, ErrorInsufficientXAmount);
            assert!(amount_y >= amount_y_min, ErrorInsufficientYAmount);
        } else {
            (amount_y, amount_x) = swap::remove_liquidity<Y, X>(sender, liquidity, coin_info);
            assert!(amount_x >= amount_x_min, ErrorInsufficientXAmount);
            assert!(amount_y >= amount_y_min, ErrorInsufficientYAmount);
        }
    }

    fun swap_event<X:key+store, Y:key+store>(
        sender_addr: address,
        amount_x_in: u64,
        amount_y_in: u64,
        amount_x_out: u64,
        amount_y_out: u64
    ) {
        if (swap_utils::sort_token_type<X, Y>()){
            swap::add_swap_event_with_address<X, Y>(sender_addr, amount_x_in, amount_y_in, amount_x_out, amount_y_out);
        } else {
            swap::add_swap_event_with_address<Y, X>(sender_addr, amount_y_in, amount_x_in, amount_y_out, amount_x_out);
        }
    }

    /// Swap exact input amount of X to maxiumin possible amount of Y
    public entry fun swap_with_exact_input<X:key+store, Y:key+store>(
        sender: &signer,
        x_in: u64,
        y_min_out: u64,
    ) {
        assert_token_pair_created<X, Y>();
        let y_out = if (swap_utils::sort_token_type<X, Y>()) {
            swap::swap_exact_x_to_y<X, Y>(sender, x_in, signer::address_of(sender))
        } else {
            swap::swap_exact_y_to_x<Y, X>(sender, x_in, signer::address_of(sender))
        };
        assert!(y_out >= y_min_out, ErrorInsufficientOutputAmount);
        swap_event<X, Y>(address_of(sender), x_in, 0, 0, y_out);
    }

    /// Swap miniumn possible amount of X to exact output amount of Y
    public entry fun swap_with_exact_output<X:key+store, Y:key+store>(
        sender: &signer,
        y_out: u64,
        x_max_in: u64,
    ) {
        assert_token_pair_created<X, Y>();
        let x_in = if (swap_utils::sort_token_type<X, Y>()) {
            let (rin, rout, _) = swap::token_reserves<X, Y>();
            let amount_in = swap_utils::get_amount_in(y_out, rin, rout);
            swap::swap_x_to_exact_y<X, Y>(sender, amount_in, y_out, signer::address_of(sender))
        } else {
            let (rout, rin, _) = swap::token_reserves<Y, X>();
            let amount_in = swap_utils::get_amount_in(y_out, rin, rout);
            swap::swap_y_to_exact_x<Y, X>(sender, amount_in, y_out, signer::address_of(sender))
        };
        assert!(x_in <= x_max_in, ErrorInsufficientInputAmount);
        swap_event<X, Y>(address_of(sender), x_in, 0, 0, y_out);
    }

    fun get_output_coin<X:key+store, Y:key+store>(is_x_to_y: bool, x_in: coin::Coin<X>): coin::Coin<Y> {
        if (is_x_to_y) {
            let (x_out, y_out) = swap::swap_exact_x_to_y_direct<X, Y>(x_in);
            coin::destroy_zero(x_out);
            y_out
        }
        else {
            let (y_out, x_out) = swap::swap_exact_y_to_x_direct<Y, X>(x_in);
            coin::destroy_zero(x_out);
            y_out
        }
    }

    public fun swap_exact_x_to_y_direct_external<X:key+store, Y:key+store>(x_in: coin::Coin<X>): coin::Coin<Y> {
        assert_token_pair_created<X, Y>();
        let x_in_amount = coin::value(&x_in);
        let is_x_to_y = swap_utils::sort_token_type<X, Y>();
        let y_out = get_output_coin<X, Y>(is_x_to_y, x_in);
        let y_out_amount = coin::value(&y_out);
        swap_event<X, Y>(@0x0, (x_in_amount as u64), 0, 0, (y_out_amount as u64));
        y_out
    }

    fun get_intermediate_output_x_to_exact_y<X:key+store, Y:key+store>(is_x_to_y: bool, x_in: coin::Coin<X>, amount_out: u64): coin::Coin<Y> {
        if (is_x_to_y) {
            let (x_out, y_out) = swap::swap_x_to_exact_y_direct<X, Y>(x_in, amount_out);
            coin::destroy_zero(x_out);
            y_out
        }
        else {
            let (y_out, x_out) = swap::swap_y_to_exact_x_direct<Y, X>(x_in, amount_out);
            coin::destroy_zero(x_out);
            y_out
        }
    }

    fun get_amount_in_internal<X:key+store, Y:key+store>(is_x_to_y:bool, y_out_amount: u64): u64 {
        if (is_x_to_y) {
            let (rin, rout, _) = swap::token_reserves<X, Y>();
            swap_utils::get_amount_in(y_out_amount, rin, rout)
        } else {
            let (rout, rin, _) = swap::token_reserves<Y, X>();
            swap_utils::get_amount_in(y_out_amount, rin, rout)
        }
    }

    fun get_amount_out_internal<X:key+store, Y:key+store>(is_x_to_y:bool, x_in_amount: u64): u64 {
        if (is_x_to_y) {
            let (rin, rout, _) = swap::token_reserves<X, Y>();
            swap_utils::get_amount_out(x_in_amount, rin, rout)
        } else {
            let (rout, rin, _) = swap::token_reserves<Y, X>();
            swap_utils::get_amount_out(x_in_amount, rin, rout)
        }
    }

    public fun get_amount_in<X:key+store, Y:key+store>(y_out_amount: u64): u64 {
        assert_token_pair_created<X, Y>();
        let is_x_to_y = swap_utils::sort_token_type<X, Y>();
        get_amount_in_internal<X, Y>(is_x_to_y, y_out_amount)
    }

    public fun get_amount_out<X:key+store, Y:key+store>(x_in_amount: u64): u64 {
        assert_token_pair_created<X, Y>();
        let is_x_to_y = swap_utils::sort_token_type<X, Y>();
        get_amount_out_internal<X, Y>(is_x_to_y, x_in_amount)
    }

    public fun swap_x_to_exact_y_direct_external<X:key+store, Y:key+store>(x_in: coin::Coin<X>, y_out_amount:u64): (coin::Coin<X>, coin::Coin<Y>) {
        assert_token_pair_created<X, Y>();
        let is_x_to_y = swap_utils::sort_token_type<X, Y>();
        let x_in_withdraw_amount = get_amount_in_internal<X, Y>(is_x_to_y, y_out_amount);
        let x_in_amount = coin::value(&x_in);
        assert!(x_in_amount >= (x_in_withdraw_amount as u256), ErrorInsufficientXAmount);
        let x_in_left = coin::extract(&mut x_in, x_in_amount - (x_in_withdraw_amount as u256));
        let y_out = get_intermediate_output_x_to_exact_y<X, Y>(is_x_to_y, x_in, y_out_amount);
        swap_event<X, Y>(@0x0, x_in_withdraw_amount, 0, 0, y_out_amount);
        (x_in_left, y_out)
    }

    fun swap_exact_input_double_internal<X:key+store, Y:key+store, Z:key+store>(
        sender: &signer,
        first_is_x_to_y: bool,
        second_is_y_to_z: bool,
        x_in: u64,
        z_min_out: u64,
    ): u64 {
        let coin_x = account_coin_store::withdraw<X>(sender, (x_in as u256));
        let coin_y = get_output_coin<X, Y>(first_is_x_to_y, coin_x);
        let coins_y_out = (coin::value(&coin_y) as u64);
        let coin_z = get_output_coin<Y, Z>(second_is_y_to_z, coin_y);

        let coin_z_amt = (coin::value(&coin_z) as u64);

        assert!(coin_z_amt >= z_min_out, ErrorInsufficientOutputAmount);
        let sender_addr = signer::address_of(sender);
        account_coin_store::deposit(sender_addr, coin_z);

        swap_event<X, Y>(address_of(sender), x_in, 0, 0, coins_y_out);
        swap_event<Y, Z>(address_of(sender), coins_y_out, 0, 0, coin_z_amt);
        coin_z_amt
    }

    /// Same as `swap_exact_input` with specify path: X -> Y -> Z
    public entry fun swap_exact_input_doublehop<X:key+store, Y:key+store, Z:key+store>(
        sender: &signer,
        x_in: u64,
        z_min_out: u64,
    ) {
        assert_token_pair_created<X, Y>();
        assert_token_pair_created<Y, Z>();
        let first_is_x_to_y: bool = swap_utils::sort_token_type<X, Y>();

        let second_is_y_to_z: bool = swap_utils::sort_token_type<Y, Z>();

        swap_exact_input_double_internal<X, Y, Z>(sender, first_is_x_to_y, second_is_y_to_z, x_in, z_min_out);
    }

    fun swap_exact_output_double_internal<X:key+store, Y:key+store, Z:key+store>(
        sender: &signer,
        first_is_x_to_y: bool,
        second_is_y_to_z: bool,
        x_max_in: u64,
        z_out: u64,
    ): u64 {
        let rin;
        let rout;
        let y_out = if (second_is_y_to_z) {
            (rin, rout, _) = swap::token_reserves<Y, Z>();
            swap_utils::get_amount_in(z_out, rin, rout)
        }else {
            (rout, rin, _) = swap::token_reserves<Z, Y>();
            swap_utils::get_amount_in(z_out, rin, rout)
        };
        let x_in = if (first_is_x_to_y) {
            (rin, rout, _) = swap::token_reserves<X, Y>();
            swap_utils::get_amount_in(y_out, rin, rout)
        }else {
            (rout, rin, _) = swap::token_reserves<Y, X>();
            swap_utils::get_amount_in(y_out, rin, rout)
        };

        assert!(x_in <= x_max_in, ErrorInsufficientInputAmount);

        let coin_x = account_coin_store::withdraw<X>(sender, (x_in as u256));
        let coin_y = get_intermediate_output_x_to_exact_y<X, Y>(first_is_x_to_y, coin_x, y_out);
        let coin_z = get_intermediate_output_x_to_exact_y<Y, Z>(second_is_y_to_z, coin_y, z_out);

        let coin_z_amt = (coin::value(&coin_z) as u64);
        let sender_addr = signer::address_of(sender);
        account_coin_store::deposit(sender_addr, coin_z);

        swap_event<X, Y>(address_of(sender), x_in, 0, 0, y_out);
        swap_event<Y, Z>(address_of(sender), y_out, 0, 0, coin_z_amt);
        coin_z_amt
    }

    /// Same as `swap_exact_output` with specify path: X -> Y -> Z
    public entry fun swap_exact_output_doublehop<X:key+store, Y:key+store, Z:key+store>(
        sender: &signer,
        z_out: u64,
        x_max_in: u64,
    ) {
        assert_token_pair_created<X, Y>();
        assert_token_pair_created<Y, Z>();
        let first_is_x_to_y: bool = swap_utils::sort_token_type<X, Y>();

        let second_is_y_to_z: bool = swap_utils::sort_token_type<Y, Z>();

        swap_exact_output_double_internal<X, Y, Z>(sender, first_is_x_to_y, second_is_y_to_z, x_max_in, z_out);
    }

    fun swap_exact_input_triple_internal<X:key+store, Y:key+store, Z:key+store, A:key+store>(
        sender: &signer,
        first_is_x_to_y: bool,
        second_is_y_to_z: bool,
        third_is_z_to_a: bool,
        x_in: u64,
        a_min_out: u64,
    ): u64 {
        let coin_x = account_coin_store::withdraw<X>(sender, (x_in as u256));
        let coin_y = get_output_coin<X, Y>(first_is_x_to_y, coin_x);
        let coins_y_out = (coin::value(&coin_y) as u64);

        let coin_z = get_output_coin<Y, Z>(second_is_y_to_z, coin_y);
        let coins_z_out = (coin::value(&coin_z) as u64);

        let coin_a = get_output_coin<Z, A>(third_is_z_to_a, coin_z);

        let coin_a_amt = (coin::value(&coin_a) as u64);

        assert!(coin_a_amt >= a_min_out, ErrorInsufficientOutputAmount);
        let sender_addr = signer::address_of(sender);
        account_coin_store::deposit(sender_addr, coin_a);

        swap_event<X, Y>(address_of(sender), x_in, 0, 0, coins_y_out);
        swap_event<Y, Z>(address_of(sender), coins_y_out, 0, 0, coins_z_out);
        swap_event<Z, A>(address_of(sender), coins_z_out, 0, 0, coin_a_amt);
        coin_a_amt
    }

    /// Same as `swap_exact_input` with specify path: X -> Y -> Z -> A
    public entry fun swap_exact_input_triplehop<X:key+store, Y:key+store, Z:key+store, A:key+store>(
        sender: &signer,
        x_in: u64,
        a_min_out: u64,
    ) {
        assert_token_pair_created<X, Y>();
        assert_token_pair_created<Y, Z>();
        assert_token_pair_created<Z, A>();
        let first_is_x_to_y: bool = swap_utils::sort_token_type<X, Y>();

        let second_is_y_to_z: bool = swap_utils::sort_token_type<Y, Z>();

        let third_is_z_to_a: bool = swap_utils::sort_token_type<Z, A>();

        swap_exact_input_triple_internal<X, Y, Z, A>(sender, first_is_x_to_y, second_is_y_to_z, third_is_z_to_a, x_in, a_min_out);
    }

    fun swap_exact_output_triple_internal<X:key+store, Y:key+store, Z:key+store, A:key+store>(
        sender: &signer,
        first_is_x_to_y: bool,
        second_is_y_to_z: bool,
        third_is_z_to_a: bool,
        x_max_in: u64,
        a_out: u64,
    ): u64 {
        let rin;
        let rout;
        let z_out = if (third_is_z_to_a) {
            (rin, rout, _) = swap::token_reserves<Z, A>();
            swap_utils::get_amount_in(a_out, rin, rout)
        }else {
            (rout, rin, _) = swap::token_reserves<A, Z>();
            swap_utils::get_amount_in(a_out, rin, rout)
        };

        let y_out = if (second_is_y_to_z) {
            (rin, rout, _) = swap::token_reserves<Y, Z>();
            swap_utils::get_amount_in(z_out, rin, rout)
        }else {
            (rout, rin, _) = swap::token_reserves<Z, Y>();
            swap_utils::get_amount_in(z_out, rin, rout)
        };
        let x_in = if (first_is_x_to_y) {
            (rin, rout, _) = swap::token_reserves<X, Y>();
            swap_utils::get_amount_in(y_out, rin, rout)
        }else {
            (rout, rin, _) = swap::token_reserves<Y, X>();
            swap_utils::get_amount_in(y_out, rin, rout)
        };

        assert!(x_in <= x_max_in, ErrorInsufficientInputAmount);

        let coin_x = account_coin_store::withdraw<X>(sender, (x_in as u256));
        let coin_y = get_intermediate_output_x_to_exact_y<X, Y>(first_is_x_to_y, coin_x, y_out);
        let coin_z = get_intermediate_output_x_to_exact_y<Y, Z>(second_is_y_to_z, coin_y, z_out);
        let coin_a = get_intermediate_output_x_to_exact_y<Z, A>(third_is_z_to_a, coin_z, a_out);

        let coin_a_amt = (coin::value(&coin_a) as u64);
        let sender_addr = signer::address_of(sender);
        account_coin_store::deposit(sender_addr, coin_a);

        swap_event<X, Y>(address_of(sender), x_in, 0, 0, y_out);
        swap_event<Y, Z>(address_of(sender), y_out, 0, 0, z_out);
        swap_event<Z, A>(address_of(sender), z_out, 0, 0, coin_a_amt);
        coin_a_amt
    }

    /// Same as `swap_exact_output` with specify path: X -> Y -> Z -> A
    public entry fun swap_exact_output_triplehop<X:key+store, Y:key+store, Z:key+store, A:key+store>(
        sender: &signer,
        a_out: u64,
        x_max_in: u64,
    ) {
        assert_token_pair_created<X, Y>();
        assert_token_pair_created<Y, Z>();
        assert_token_pair_created<Z, A>();
        let first_is_x_to_y: bool = swap_utils::sort_token_type<X, Y>();

        let second_is_y_to_z: bool = swap_utils::sort_token_type<Y, Z>();

        let third_is_z_to_a: bool = swap_utils::sort_token_type<Z, A>();

        swap_exact_output_triple_internal<X, Y, Z, A>(sender, first_is_x_to_y, second_is_y_to_z, third_is_z_to_a, x_max_in, a_out);
    }


    fun swap_exact_input_quadruple_internal<X:key+store, Y:key+store, Z:key+store, A:key+store, B:key+store>(
        sender: &signer,
        first_is_x_to_y: bool,
        second_is_y_to_z: bool,
        third_is_z_to_a: bool,
        fourth_is_a_to_b: bool,
        x_in: u64,
        b_min_out: u64,
    ): u64 {
        let coin_x = account_coin_store::withdraw<X>(sender, (x_in as u256));
        let coin_y = get_output_coin<X, Y>(first_is_x_to_y, coin_x);
        let coins_y_out = (coin::value(&coin_y) as u64);

        let coin_z = get_output_coin<Y, Z>(second_is_y_to_z, coin_y);
        let coins_z_out = (coin::value(&coin_z) as u64);

        let coin_a = get_output_coin<Z, A>(third_is_z_to_a, coin_z);
        let coin_a_out = (coin::value(&coin_a) as u64);

        let coin_b = get_output_coin<A, B>(fourth_is_a_to_b, coin_a);
        let coin_b_amt = (coin::value(&coin_b) as u64);

        assert!(coin_b_amt >= b_min_out, ErrorInsufficientOutputAmount);
        let sender_addr = signer::address_of(sender);
        account_coin_store::deposit(sender_addr, coin_b);

        swap_event<X, Y>(address_of(sender), x_in, 0, 0, coins_y_out);
        swap_event<Y, Z>(address_of(sender), coins_y_out, 0, 0, coins_z_out);
        swap_event<Z, A>(address_of(sender), coins_z_out, 0, 0, coin_a_out);
        swap_event<A, B>(address_of(sender), coin_a_out, 0, 0, coin_b_amt);
        coin_b_amt
    }

    /// Same as `swap_exact_input` with specify path: X -> Y -> Z -> A -> B
    public entry fun swap_exact_input_quadruplehop<X:key+store, Y:key+store, Z:key+store, A:key+store, B:key+store>(
        sender: &signer,
        x_in: u64,
        b_min_out: u64,
    ) {
        assert_token_pair_created<X, Y>();
        assert_token_pair_created<Y, Z>();
        assert_token_pair_created<Z, A>();
        assert_token_pair_created<A, B>();
        let first_is_x_to_y: bool = swap_utils::sort_token_type<X, Y>();

        let second_is_y_to_z: bool = swap_utils::sort_token_type<Y, Z>();

        let third_is_z_to_a: bool = swap_utils::sort_token_type<Z, A>();

        let fourth_is_a_to_b: bool = swap_utils::sort_token_type<A, B>();

        swap_exact_input_quadruple_internal<X, Y, Z, A, B>(sender, first_is_x_to_y, second_is_y_to_z, third_is_z_to_a, fourth_is_a_to_b, x_in, b_min_out);
    }

    fun swap_exact_output_quadruple_internal<X:key+store, Y:key+store, Z:key+store, A:key+store, B:key+store>(
        sender: &signer,
        first_is_x_to_y: bool,
        second_is_y_to_z: bool,
        third_is_z_to_a: bool,
        fourth_is_a_to_b: bool,
        x_max_in: u64,
        b_out: u64,
    ): u64 {
        let rin;
        let rout;

        let a_out = if (fourth_is_a_to_b) {
            (rin, rout, _) = swap::token_reserves<A, B>();
            swap_utils::get_amount_in(b_out, rin, rout)
        }else {
            (rout, rin, _) = swap::token_reserves<B, A>();
            swap_utils::get_amount_in(b_out, rin, rout)
        };

        let z_out = if (third_is_z_to_a) {
            (rin, rout, _) = swap::token_reserves<Z, A>();
            swap_utils::get_amount_in(a_out, rin, rout)
        }else {
            (rout, rin, _) = swap::token_reserves<A, Z>();
            swap_utils::get_amount_in(a_out, rin, rout)
        };

        let y_out = if (second_is_y_to_z) {
            (rin, rout, _) = swap::token_reserves<Y, Z>();
            swap_utils::get_amount_in(z_out, rin, rout)
        }else {
            (rout, rin, _) = swap::token_reserves<Z, Y>();
            swap_utils::get_amount_in(z_out, rin, rout)
        };
        let x_in = if (first_is_x_to_y) {
            (rin, rout, _) = swap::token_reserves<X, Y>();
            swap_utils::get_amount_in(y_out, rin, rout)
        }else {
            (rout, rin, _) = swap::token_reserves<Y, X>();
            swap_utils::get_amount_in(y_out, rin, rout)
        };

        assert!(x_in <= x_max_in, ErrorInsufficientInputAmount);

        let coin_x = account_coin_store::withdraw<X>(sender, (x_in as u256));
        let coin_y = get_intermediate_output_x_to_exact_y<X, Y>(first_is_x_to_y, coin_x, y_out);
        let coin_z = get_intermediate_output_x_to_exact_y<Y, Z>(second_is_y_to_z, coin_y, z_out);
        let coin_a = get_intermediate_output_x_to_exact_y<Z, A>(third_is_z_to_a, coin_z, a_out);
        let coin_b = get_intermediate_output_x_to_exact_y<A, B>(fourth_is_a_to_b, coin_a, b_out);

        let coin_b_amt = (coin::value(&coin_b) as u64);
        let sender_addr = signer::address_of(sender);
        account_coin_store::deposit(sender_addr, coin_b);

        swap_event<X, Y>(address_of(sender), x_in, 0, 0, y_out);
        swap_event<Y, Z>(address_of(sender), y_out, 0, 0, z_out);
        swap_event<Z, A>(address_of(sender), z_out, 0, 0, a_out);
        swap_event<A, B>(address_of(sender), a_out, 0, 0, coin_b_amt);
        coin_b_amt
    }

    /// Same as `swap_exact_output` with specify path: X -> Y -> Z -> A -> B
    public entry fun swap_exact_output_quadruplehop<X:key+store, Y:key+store, Z:key+store, A:key+store, B:key+store>(
        sender: &signer,
        b_out: u64,
        x_max_in: u64,
    ) {
        assert_token_pair_created<X, Y>();
        assert_token_pair_created<Y, Z>();
        assert_token_pair_created<Z, A>();
        assert_token_pair_created<A, B>();
        let first_is_x_to_y: bool = swap_utils::sort_token_type<X, Y>();

        let second_is_y_to_z: bool = swap_utils::sort_token_type<Y, Z>();

        let third_is_z_to_a: bool = swap_utils::sort_token_type<Z, A>();

        let fourth_is_a_to_b = swap_utils::sort_token_type<A, B>();

        swap_exact_output_quadruple_internal<X, Y, Z, A, B>(sender, first_is_x_to_y, second_is_y_to_z, third_is_z_to_a, fourth_is_a_to_b, x_max_in, b_out);
    }
}

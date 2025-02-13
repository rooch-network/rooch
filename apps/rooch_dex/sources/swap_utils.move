/// Uniswap v2 like token swap program
module rooch_dex::swap_utils {
    use std::string;
    use moveos_std::type_info;
    use moveos_std::compare::compare_vector_u8;


    const EQUAL: u8 = 0;
    const SMALLER: u8 = 1;
    const GREATER: u8 = 2;

    const ErrorInputTokenAmount: u64 = 1;
    const ErrorInsufficientLiquidity: u64 = 2;
    const ErrorInsufficientXAmount: u64 = 3;
    const ErrorOutputTokenAmount: u64 = 4;
    const ErrorTokenPairAleardyExist: u64 = 5;

    public fun get_amount_out(
        amount_in: u64,
        reserve_in: u64,
        reserve_out: u64,
        fee_rate: u64,
    ): u64 {
        assert!(amount_in > 0, ErrorInputTokenAmount);
        assert!(reserve_in > 0 && reserve_out > 0, ErrorInsufficientLiquidity);

        let amount_in_with_fee = (amount_in as u128) * (fee_rate as u128);
        let numerator = amount_in_with_fee * (reserve_out as u128);
        let denominator = (reserve_in as u128) * 10000u128 + amount_in_with_fee;
        ((numerator / denominator) as u64)
    }

    public fun get_amount_in(
        amount_out: u64,
        reserve_in: u64,
        reserve_out: u64,
        fee_rate: u64
    ): u64 {
        assert!(amount_out > 0, ErrorOutputTokenAmount);
        assert!(reserve_in > 0 && reserve_out > 0, ErrorInsufficientLiquidity);

        let numerator = (reserve_in as u128) * (amount_out as u128) * 10000u128;
        let denominator = ((reserve_out as u128) - (amount_out as u128)) * (fee_rate as u128);
        (((numerator / denominator) as u64) + 1u64)
    }

    public fun quote(amount_x: u64, reserve_x: u64, reserve_y: u64): u64 {
        assert!(amount_x > 0, ErrorInsufficientXAmount);
        assert!(reserve_x > 0 && reserve_y > 0, ErrorInsufficientLiquidity);
        (((amount_x as u128) * (reserve_y as u128) / (reserve_x as u128)) as u64)
    }

    public fun get_token_info<T>(): vector<u8> {
        let type_name = type_info::type_name<T>();
        string::into_bytes(type_name)
    }

    fun compare_struct<X, Y>(): u8 {
        let struct_x_bytes: vector<u8> = get_token_info<X>();
        let struct_y_bytes: vector<u8> = get_token_info<Y>();
        compare_vector_u8(&struct_x_bytes, &struct_y_bytes)
    }

    public fun get_smaller_enum(): u8 {
        SMALLER
    }

    public fun get_greater_enum(): u8 {
        GREATER
    }

    public fun get_equal_enum(): u8 {
        EQUAL
    }

    public fun sort_token_type<X, Y>(): bool {
        let compare_x_y: u8 = compare_struct<X, Y>();
        assert!(compare_x_y != get_equal_enum(), ErrorTokenPairAleardyExist);
        (compare_x_y == get_smaller_enum())
    }

    #[test_only]
    struct TokenABC {}
    #[test_only]
    struct TokenCBA {}
    #[test_only]
    struct TokenACB {}

    #[test_only]
    const FEE_RATE: u64 = 9975; // 0.25% fee = 9975/10000
    
    #[test]
    fun test_sort_token_type() {
        assert!(sort_token_type<TokenABC, TokenCBA>(), 1);
        // assert!(!sort_token_type<TokenB, TokenA>(), 2);
        // assert!(sort_token_type<rooch_framework::gas_coin::RGas, TokenA>(), 3);
        // assert!(sort_token_type<rooch_framework::gas_coin::RGas, TokenB>(), 4);
    }

    // #[test]
    // #[expected_failure(abort_code = ErrorTokenPairAleardyExist, location = rooch_dex::swap_utils)]
    // fun test_sort_token_type_same() {
    //     sort_token_type<TokenA, TokenA>();
    // }

    #[test]
    fun test_get_amount_out() {
        let amount_out = get_amount_out(100, 1000, 1000, FEE_RATE);
        assert!(amount_out > 0, 0);
        let expected = 90;
        assert!(amount_out == expected, 1);
    }

    #[test]
    #[expected_failure(abort_code = ErrorInputTokenAmount, location = rooch_dex::swap_utils)]
    fun test_get_amount_out_zero_input() {
        get_amount_out(0, 1000, 1000, FEE_RATE);
    }

    #[test]
    #[expected_failure(abort_code = ErrorInsufficientLiquidity, location = rooch_dex::swap_utils)]
    fun test_get_amount_out_zero_reserves() {
        get_amount_out(100, 0, 0, FEE_RATE);
    }
}

// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module pump_rooch::bonding_curve {

    use std::string::String;
    use std::u8;
    use moveos_std::object::Object;
    use moveos_std::object;
    use moveos_std::timestamp::now_milliseconds;
    use pump_rooch::legato_math;
    use pump_rooch::math_fixed64;
    use pump_rooch::fixed_point64;
    use pump_rooch::fixed_point64::FixedPoint64;
    use moveos_std::decimal_value::{DecimalValue, value, decimal, new};

    const B: u128 = 5;

    const A: u128 = 1866;

    const ErrorInvalidDecimal: u64 = 0;

    struct BondingCurve has key {
        name: String,
        a: u128,
        b: u128,
        total_supply: u256,
        decimal: u8,
        reserve: u256,
        timestamp: u64
    }


    public fun create_coin(name: String, total_supply: u256, decimal: u8) {
        let bc = BondingCurve {
            name,
            a: A,
            b: B,
            total_supply,
            decimal,
            reserve: 0,
            timestamp: now_milliseconds()
        };
        object::to_shared(object::new(bc))
    }


    public fun get_value_with_precision(num: FixedPoint64, precision: u8): u128 {
        let raw = fixed_point64::get_raw_value(num);
        let scale = 1u128;
        let i = 0u8;
        while (i < precision) {
            scale = scale * 10;
            i = i + 1;
        };
        let integer = raw >> 64;
        let frac = raw & ((1u128 << 64) - 1);
        let scaled_frac = (frac * scale) >> 64;
        integer * scale + scaled_frac
    }


    public fun decimal_value_2_fixed_point(decimal_value: &DecimalValue): FixedPoint64 {
        fixed_point64::create_from_rational((value(decimal_value) as u128),
            (u8::pow(10, decimal(decimal_value)) as u128)
        )
    }

    /// Calculate buy amount
    /// delta_y: Amount of funds invested
    /// x_0: Current token amount in pool
    public fun calculate_buy(delta_y: &DecimalValue, bc_obj: &Object<BondingCurve>): u64 {
        let bc = object::borrow(bc_obj);
        let x_0 =  &new(bc.reserve, bc.decimal) ;
        assert!(decimal(delta_y) == bc.decimal, ErrorInvalidDecimal);
        // Convert input to FixedPoint64
        let x0_fixed = decimal_value_2_fixed_point(x_0);
        let dy_fixed = decimal_value_2_fixed_point(delta_y);
        let b_fixed = fixed_point64::create_from_rational(B, (u8::pow(10, bc.decimal) as u128));
        let a_fixed = fixed_point64::create_from_rational(A, (u8::pow(10, bc.decimal) as u128));

        // Calculate exp(b*x0)
        let b_x0 = math_fixed64::mul_div(
            b_fixed,
            x0_fixed,
            fixed_point64::create_from_u128(1)
        );
        let exp_b_x0 = math_fixed64::exp(b_x0);

        // Calculate dy*b/a
        let dy_b = math_fixed64::mul_div(dy_fixed, b_fixed, a_fixed);

        // Calculate exp(b*x0) + (dy*b/a)
        let exp_b_x1 = fixed_point64::add(exp_b_x0, dy_b);

        // Calculate ln(exp_b_x1)
        let ln_exp_b_x1 = legato_math::ln(exp_b_x1);

        // Calculate ln(exp_b_x1)/b
        let result = math_fixed64::mul_div(
            ln_exp_b_x1,
            fixed_point64::create_from_u128(1),
            b_fixed
        );

        // Calculate ln(exp_b_x1)/b - x0
        let delta_x = fixed_point64::sub(result, x0_fixed);

        (get_value_with_precision(delta_x, 9) as u64)
    }

    /// Calculate return amount when selling
    /// delta_x: Amount of tokens to sell
    /// x_0: Current token amount in pool
    public fun calculate_sell(delta_x: &DecimalValue, bc_obj: &Object<BondingCurve>): u64 {
        let bc = object::borrow(bc_obj);
        let x_0 =  &new(bc.reserve, bc.decimal) ;
        assert!(decimal(delta_x) == bc.decimal, ErrorInvalidDecimal);
        // Convert input to FixedPoint64
        let x0_fixed = decimal_value_2_fixed_point(x_0);
        let dx_fixed = decimal_value_2_fixed_point(delta_x);
        let b_fixed = fixed_point64::create_from_rational(B, (u8::pow(10, bc.decimal) as u128));
        let a_fixed = fixed_point64::create_from_rational(A, (u8::pow(10, bc.decimal) as u128));

        // Calculate exp(b*x0)
        let b_x0 = math_fixed64::mul_div(
            b_fixed,
            x0_fixed,
            fixed_point64::create_from_u128(1)
        );
        let exp_b_x0 = math_fixed64::exp(b_x0);

        // Calculate exp(b*(x0-dx))
        let x1_fixed = fixed_point64::sub(x0_fixed, dx_fixed);
        let b_x1 = math_fixed64::mul_div(
            b_fixed,
            x1_fixed,
            fixed_point64::create_from_u128(1)
        );
        let exp_b_x1 = math_fixed64::exp(b_x1);

        // Calculate exp(b*x0) - exp(b*x1)
        let delta_exp = fixed_point64::sub(exp_b_x0, exp_b_x1);

        // Calculate (a/b)*(exp(b*x0) - exp(b*x1))
        let result = math_fixed64::mul_div(a_fixed, delta_exp, b_fixed);


        (get_value_with_precision(result, 9) as u64)
    }

}
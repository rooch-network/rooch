// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module pump_rooch::legato_math {

    use pump_rooch::fixed_point64::{Self, FixedPoint64};
    use pump_rooch::math_fixed64;

    const LOG_2_E: u128 = 26613026195707766742;

    // Maximum values for u64 and u128
    const MAX_U64: u128 = 18446744073709551615;
    const MAX_U128: u256 = 340282366920938463463374607431768211455;

    // Helper function to calculate the power of a FixedPoint64 number to a FixedPoint64 exponent
    // - When `n` is > 1, it uses the formula `exp(y * ln(x))` instead of `x^y`.
    // - When `n` is < 1, it employs the Newton-Raphson method.
    public fun power(n: FixedPoint64, e: FixedPoint64) : FixedPoint64 {
        // Check if the exponent is 0, return 1 if it is
        if (fixed_point64::equal(e, fixed_point64::create_from_u128(0)) ) {
            fixed_point64::create_from_u128(1)
        } else if (fixed_point64::equal(e, fixed_point64::create_from_u128(1))) {
            // If the exponent is 1, return the base value n
            n
        } else if (fixed_point64::less(n, fixed_point64::create_from_u128(1))) {

            // Split the exponent into integer and fractional parts
            let integerPart = fixed_point64::floor( e );
            let fractionalPart = fixed_point64::sub(e, fixed_point64::create_from_u128(integerPart));

            // Calculate the integer power using math_fixed64 power function
            let result = math_fixed64::pow( n, (integerPart as u64) );

            if ( fixed_point64::equal( fractionalPart, fixed_point64::create_from_u128(0) ) ) {
                // If the fractional part is zero, return the integer result
                result
            } else {
                // Calculate the fractional using internal nth root function
                let nth = math_fixed64::mul_div( fixed_point64::create_from_u128(1), fixed_point64::create_from_u128(1), fractionalPart );

                let nth_rounded = fixed_point64::round(nth);

                let fractionalResult =  nth_root( n , (nth_rounded as u64) );

                // Combine the integer and fractional powers using multiplication
                math_fixed64::mul_div( result, fractionalResult,  fixed_point64::create_from_u128(1)  )
            }

        } else {

            // Calculate ln(n) times e
            let ln_x_times_y = math_fixed64::mul_div(  e , ln(n), fixed_point64::create_from_u128(1) );
            // Compute exp(ln(x) * y) to get the result of x^y
            math_fixed64::exp(ln_x_times_y)
        }

    }

    // Helper function to approximate the n-th root of a number using the Newton-Raphson method when x < 1.
    public fun nth_root( x: FixedPoint64, n: u64): FixedPoint64 {
        if ( n == 0 ) {
            fixed_point64::create_from_u128(1)
        } else {

            // Initialize guess
            let guess = fixed_point64::create_from_rational(1, 2);

            // Define the epsilon value for determining convergence
            let epsilon = fixed_point64::create_from_rational( 1, 1000 );

            let delta = fixed_point64::create_from_rational( MAX_U64, 1 );

            // Perform Newton-Raphson iterations until convergence
            while ( fixed_point64::greater( delta ,  epsilon )) {

                let xn = pow_raw( guess,  n);
                let derivative = math_fixed64::mul_div( fixed_point64::create_from_u128( (n as u128)), pow_raw( guess,  n-1), fixed_point64::create_from_u128(1) );

                if (fixed_point64::greater_or_equal(xn, x)) {
                    delta = math_fixed64::mul_div( fixed_point64::sub(xn, x) , fixed_point64::create_from_u128(1), derivative);
                    guess = fixed_point64::sub(guess, delta);
                } else {
                    delta = math_fixed64::mul_div( fixed_point64::sub(x, xn) , fixed_point64::create_from_u128(1), derivative);
                    guess = fixed_point64::add(guess, delta);
                };

            };
            // Return the final approximation of the n-th root
            guess
        }
    }

    // Function to calculate the power of a FixedPoint64 number
    public fun pow_raw(x: FixedPoint64, n: u64): FixedPoint64 {
        // Get the raw value of x as a 256-bit unsigned integer
        let raw_value = (fixed_point64::get_raw_value(x) as u256);

        let res: u256 = 1 << 64;

        // Perform exponentiation using bitwise operations
        while (n != 0) {
            if (n & 1 != 0) {
                res = (res * raw_value) >> 64;
            };
            n = n >> 1;
            if ( raw_value <= MAX_U128 ) {
                raw_value = (raw_value * raw_value) >> 64;
            } else {
                raw_value = (raw_value >> 32) * (raw_value >> 32);
            };
        };

        fixed_point64::create_from_raw_value((res as u128))
    }

    // Calculate the natural logarithm of the input using FixedPoint64
    public fun ln(input : FixedPoint64) : FixedPoint64 {
        // Define the constant log_2(e)
        let log_2_e = fixed_point64::create_from_raw_value( LOG_2_E );

        // Calculate the base-2 logarithm of the input
        let after_log2 = (math_fixed64::log2_plus_64( input ));

        let fixed_2 = fixed_point64::create_from_u128(64);

        // Subtract 64 to adjust the result back
        let (after_subtracted, _) = absolute( after_log2, fixed_2 );
        math_fixed64::mul_div( after_subtracted, fixed_point64::create_from_u128(1) , log_2_e)
    }

    public fun absolute( a: FixedPoint64, b:  FixedPoint64 ) : (FixedPoint64, bool) {
        if (fixed_point64::greater_or_equal(a, b)) {
            (fixed_point64::sub(a, b), false)
        } else {
            (fixed_point64::sub(b, a), true)
        }
    }

    #[test]
    public fun test_ln() {

        let output_1 = ln( fixed_point64::create_from_u128(10) );
        assert!( fixed_point64::almost_equal( output_1, fixed_point64::create_from_rational( 230258509299, 100000000000  ), fixed_point64::create_from_u128(1)) , 0 ); // 2.30258509299

        let output_2 = ln( fixed_point64::create_from_u128(100) );
        assert!( fixed_point64::almost_equal( output_2, fixed_point64::create_from_rational( 460517018599 , 100000000000  ), fixed_point64::create_from_u128(1)) , 1 ); // 4.60517018599

        let output_3 = ln( fixed_point64::create_from_u128(500) );
        assert!( fixed_point64::almost_equal( output_3, fixed_point64::create_from_rational( 621460809842 , 100000000000  ), fixed_point64::create_from_u128(1)) , 2 ); // 6.21460809842

        // return absolute value when input < 1
        let output_4 = ln( fixed_point64::create_from_rational(1, 2) );
        assert!( fixed_point64::almost_equal( output_4, fixed_point64::create_from_rational( 693147181 , 1000000000  ), fixed_point64::create_from_u128(1)) , 2 ); // 0.693147181

    }

    #[test]
    public fun test_power() {

        // Asserts that 2^3 = 8
        let output_1 = power(  fixed_point64::create_from_u128(2), fixed_point64::create_from_u128(3) );
        assert!( fixed_point64::round(output_1) == 8, 0 );

        // Asserts that 200^3 = 8000000
        let output_2 = power(  fixed_point64::create_from_u128(200), fixed_point64::create_from_u128(3) );
        assert!( fixed_point64::round(output_2) == 8000000, 1 );

        // Asserts that 30^5 = 24300000
        let output_3 = power(  fixed_point64::create_from_u128(30), fixed_point64::create_from_u128(5) );
        assert!( fixed_point64::round(output_3) == 24300000, 2 ); // 30^5 = 24300000

        // Asserts that the square root of 16 is approximately 4.
        let n_output_1 = power(  fixed_point64::create_from_u128(16), fixed_point64::create_from_rational(1, 2 )  );
        assert!( fixed_point64::almost_equal( n_output_1, fixed_point64::create_from_rational( 4, 1  ), fixed_point64::create_from_u128(1)) , 3 );
        // Asserts that the fifth root of 625 is approximately 3.623.
        let n_output_2 = power(  fixed_point64::create_from_u128(625), fixed_point64::create_from_rational(1, 5 )  );
        assert!( fixed_point64::almost_equal( n_output_2, fixed_point64::create_from_rational( 3623, 1000 ), fixed_point64::create_from_u128(1)) , 4 );
        // Asserts that the cube root of 1000 is approximately 9.999999977.
        let n_output_3 = power(  fixed_point64::create_from_u128(1000), fixed_point64::create_from_rational(1, 3 )  );
        assert!( fixed_point64::almost_equal( n_output_3, fixed_point64::create_from_rational( 9999, 1000 ), fixed_point64::create_from_u128(1)) , 5 );
        // Asserts that the cube root of 729 is approximately 8.99999998.
        let n_output_4 = power(  fixed_point64::create_from_u128(729), fixed_point64::create_from_rational(1, 3 )  );
        assert!( fixed_point64::almost_equal( n_output_4, fixed_point64::create_from_rational( 8999, 1000 ), fixed_point64::create_from_u128(1)) , 6 );

        // Asserts that the fourth root of 9/16 is approximately 0.866025404.
        let n_output_5 = power(  fixed_point64::create_from_rational( 9, 16 ), fixed_point64::create_from_rational( 1, 4 )  );
        assert!( fixed_point64::almost_equal( n_output_5, fixed_point64::create_from_rational( 866025404, 1000000000 ), fixed_point64::create_from_u128(1)) , 7 ); // 0.866025404

        // Asserts that the tenth root of 1/2 is approximately 0.420448208.
        let n_output_6 = power(  fixed_point64::create_from_rational( 1, 2 ), fixed_point64::create_from_rational( 10, 8 )  );
        assert!( fixed_point64::almost_equal( n_output_6, fixed_point64::create_from_rational( 420448208, 1000000000 ), fixed_point64::create_from_u128(1)) , 8 ); // 0.420448208

        // Asserts that the fifth root of 2/5 is approximately 0.01024.
        let n_output_7 = power(  fixed_point64::create_from_rational( 2, 5 ), fixed_point64::create_from_rational( 5, 1 )  );
        assert!( fixed_point64::almost_equal( n_output_7, fixed_point64::create_from_rational( 1024, 100000 ), fixed_point64::create_from_u128(1)) , 9 ); // 0.01024

        // Asserts that the ninth root of 3/5 is approximately 0.566896603.
        let n_output_8 = power(  fixed_point64::create_from_rational( 3, 5 ), fixed_point64::create_from_rational( 10, 9 )  );
        assert!( fixed_point64::almost_equal( n_output_8, fixed_point64::create_from_rational( 566896603, 1000000000 ), fixed_point64::create_from_u128(1)) , 10 ); // 0.566896603

    }

}
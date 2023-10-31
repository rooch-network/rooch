
<a name="0x1_fixed_point32"></a>

# Module `0x1::fixed_point32`

Defines a fixed-point numeric type with a 32-bit integer part and
a 32-bit fractional part.


-  [Struct `FixedPoint32`](#0x1_fixed_point32_FixedPoint32)
-  [Constants](#@Constants_0)
-  [Function `multiply_u64`](#0x1_fixed_point32_multiply_u64)
-  [Function `divide_u64`](#0x1_fixed_point32_divide_u64)
-  [Function `create_from_rational`](#0x1_fixed_point32_create_from_rational)
-  [Function `create_from_raw_value`](#0x1_fixed_point32_create_from_raw_value)
-  [Function `get_raw_value`](#0x1_fixed_point32_get_raw_value)
-  [Function `is_zero`](#0x1_fixed_point32_is_zero)
-  [Function `min`](#0x1_fixed_point32_min)
-  [Function `max`](#0x1_fixed_point32_max)
-  [Function `create_from_u64`](#0x1_fixed_point32_create_from_u64)
-  [Function `floor`](#0x1_fixed_point32_floor)
-  [Function `ceil`](#0x1_fixed_point32_ceil)
-  [Function `round`](#0x1_fixed_point32_round)
-  [Module Specification](#@Module_Specification_1)


<pre><code></code></pre>



<a name="0x1_fixed_point32_FixedPoint32"></a>

## Struct `FixedPoint32`

Define a fixed-point numeric type with 32 fractional bits.
This is just a u64 integer but it is wrapped in a struct to
make a unique type. This is a binary representation, so decimal
values may not be exactly representable, but it provides more
than 9 decimal digits of precision both before and after the
decimal point (18 digits total). For comparison, double precision
floating-point has less than 16 decimal digits of precision, so
be careful about using floating-point to convert these values to
decimal.


<pre><code><b>struct</b> <a href="fixed_point32.md#0x1_fixed_point32_FixedPoint32">FixedPoint32</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x1_fixed_point32_MAX_U64"></a>

> TODO: This is a basic constant and should be provided somewhere centrally in the framework.


<pre><code><b>const</b> <a href="fixed_point32.md#0x1_fixed_point32_MAX_U64">MAX_U64</a>: u128 = 18446744073709551615;
</code></pre>



<a name="0x1_fixed_point32_EDENOMINATOR"></a>

The denominator provided was zero


<pre><code><b>const</b> <a href="fixed_point32.md#0x1_fixed_point32_EDENOMINATOR">EDENOMINATOR</a>: u64 = 65537;
</code></pre>



<a name="0x1_fixed_point32_EDIVISION"></a>

The quotient value would be too large to be held in a <code>u64</code>


<pre><code><b>const</b> <a href="fixed_point32.md#0x1_fixed_point32_EDIVISION">EDIVISION</a>: u64 = 131074;
</code></pre>



<a name="0x1_fixed_point32_EDIVISION_BY_ZERO"></a>

A division by zero was encountered


<pre><code><b>const</b> <a href="fixed_point32.md#0x1_fixed_point32_EDIVISION_BY_ZERO">EDIVISION_BY_ZERO</a>: u64 = 65540;
</code></pre>



<a name="0x1_fixed_point32_EMULTIPLICATION"></a>

The multiplied value would be too large to be held in a <code>u64</code>


<pre><code><b>const</b> <a href="fixed_point32.md#0x1_fixed_point32_EMULTIPLICATION">EMULTIPLICATION</a>: u64 = 131075;
</code></pre>



<a name="0x1_fixed_point32_ERATIO_OUT_OF_RANGE"></a>

The computed ratio when converting to a <code><a href="fixed_point32.md#0x1_fixed_point32_FixedPoint32">FixedPoint32</a></code> would be unrepresentable


<pre><code><b>const</b> <a href="fixed_point32.md#0x1_fixed_point32_ERATIO_OUT_OF_RANGE">ERATIO_OUT_OF_RANGE</a>: u64 = 131077;
</code></pre>



<a name="0x1_fixed_point32_multiply_u64"></a>

## Function `multiply_u64`

Multiply a u64 integer by a fixed-point number, truncating any
fractional part of the product. This will abort if the product
overflows.


<pre><code><b>public</b> <b>fun</b> <a href="fixed_point32.md#0x1_fixed_point32_multiply_u64">multiply_u64</a>(val: u64, multiplier: <a href="fixed_point32.md#0x1_fixed_point32_FixedPoint32">fixed_point32::FixedPoint32</a>): u64
</code></pre>



<a name="0x1_fixed_point32_divide_u64"></a>

## Function `divide_u64`

Divide a u64 integer by a fixed-point number, truncating any
fractional part of the quotient. This will abort if the divisor
is zero or if the quotient overflows.


<pre><code><b>public</b> <b>fun</b> <a href="fixed_point32.md#0x1_fixed_point32_divide_u64">divide_u64</a>(val: u64, divisor: <a href="fixed_point32.md#0x1_fixed_point32_FixedPoint32">fixed_point32::FixedPoint32</a>): u64
</code></pre>



<a name="0x1_fixed_point32_create_from_rational"></a>

## Function `create_from_rational`

Create a fixed-point value from a rational number specified by its
numerator and denominator. Calling this function should be preferred
for using <code><a href="fixed_point32.md#0x1_fixed_point32_create_from_raw_value">Self::create_from_raw_value</a></code> which is also available.
This will abort if the denominator is zero. It will also
abort if the numerator is nonzero and the ratio is not in the range
2^-32 .. 2^32-1. When specifying decimal fractions, be careful about
rounding errors: if you round to display N digits after the decimal
point, you can use a denominator of 10^N to avoid numbers where the
very small imprecision in the binary representation could change the
rounding, e.g., 0.0125 will round down to 0.012 instead of up to 0.013.


<pre><code><b>public</b> <b>fun</b> <a href="fixed_point32.md#0x1_fixed_point32_create_from_rational">create_from_rational</a>(numerator: u64, denominator: u64): <a href="fixed_point32.md#0x1_fixed_point32_FixedPoint32">fixed_point32::FixedPoint32</a>
</code></pre>



<a name="0x1_fixed_point32_create_from_raw_value"></a>

## Function `create_from_raw_value`

Create a fixedpoint value from a raw value.


<pre><code><b>public</b> <b>fun</b> <a href="fixed_point32.md#0x1_fixed_point32_create_from_raw_value">create_from_raw_value</a>(value: u64): <a href="fixed_point32.md#0x1_fixed_point32_FixedPoint32">fixed_point32::FixedPoint32</a>
</code></pre>



<a name="0x1_fixed_point32_get_raw_value"></a>

## Function `get_raw_value`

Accessor for the raw u64 value. Other less common operations, such as
adding or subtracting FixedPoint32 values, can be done using the raw
values directly.


<pre><code><b>public</b> <b>fun</b> <a href="fixed_point32.md#0x1_fixed_point32_get_raw_value">get_raw_value</a>(num: <a href="fixed_point32.md#0x1_fixed_point32_FixedPoint32">fixed_point32::FixedPoint32</a>): u64
</code></pre>



<a name="0x1_fixed_point32_is_zero"></a>

## Function `is_zero`

Returns true if the ratio is zero.


<pre><code><b>public</b> <b>fun</b> <a href="fixed_point32.md#0x1_fixed_point32_is_zero">is_zero</a>(num: <a href="fixed_point32.md#0x1_fixed_point32_FixedPoint32">fixed_point32::FixedPoint32</a>): bool
</code></pre>



<a name="0x1_fixed_point32_min"></a>

## Function `min`

Returns the smaller of the two FixedPoint32 numbers.


<pre><code><b>public</b> <b>fun</b> <b>min</b>(num1: <a href="fixed_point32.md#0x1_fixed_point32_FixedPoint32">fixed_point32::FixedPoint32</a>, num2: <a href="fixed_point32.md#0x1_fixed_point32_FixedPoint32">fixed_point32::FixedPoint32</a>): <a href="fixed_point32.md#0x1_fixed_point32_FixedPoint32">fixed_point32::FixedPoint32</a>
</code></pre>



<a name="0x1_fixed_point32_max"></a>

## Function `max`

Returns the larger of the two FixedPoint32 numbers.


<pre><code><b>public</b> <b>fun</b> <a href="fixed_point32.md#0x1_fixed_point32_max">max</a>(num1: <a href="fixed_point32.md#0x1_fixed_point32_FixedPoint32">fixed_point32::FixedPoint32</a>, num2: <a href="fixed_point32.md#0x1_fixed_point32_FixedPoint32">fixed_point32::FixedPoint32</a>): <a href="fixed_point32.md#0x1_fixed_point32_FixedPoint32">fixed_point32::FixedPoint32</a>
</code></pre>



<a name="0x1_fixed_point32_create_from_u64"></a>

## Function `create_from_u64`

Create a fixedpoint value from a u64 value.


<pre><code><b>public</b> <b>fun</b> <a href="fixed_point32.md#0x1_fixed_point32_create_from_u64">create_from_u64</a>(val: u64): <a href="fixed_point32.md#0x1_fixed_point32_FixedPoint32">fixed_point32::FixedPoint32</a>
</code></pre>



<a name="0x1_fixed_point32_floor"></a>

## Function `floor`

Returns the largest integer less than or equal to a given number.


<pre><code><b>public</b> <b>fun</b> <a href="fixed_point32.md#0x1_fixed_point32_floor">floor</a>(num: <a href="fixed_point32.md#0x1_fixed_point32_FixedPoint32">fixed_point32::FixedPoint32</a>): u64
</code></pre>



<a name="0x1_fixed_point32_ceil"></a>

## Function `ceil`

Rounds up the given FixedPoint32 to the next largest integer.


<pre><code><b>public</b> <b>fun</b> <a href="fixed_point32.md#0x1_fixed_point32_ceil">ceil</a>(num: <a href="fixed_point32.md#0x1_fixed_point32_FixedPoint32">fixed_point32::FixedPoint32</a>): u64
</code></pre>



<a name="0x1_fixed_point32_round"></a>

## Function `round`

Returns the value of a FixedPoint32 to the nearest integer.


<pre><code><b>public</b> <b>fun</b> <a href="fixed_point32.md#0x1_fixed_point32_round">round</a>(num: <a href="fixed_point32.md#0x1_fixed_point32_FixedPoint32">fixed_point32::FixedPoint32</a>): u64
</code></pre>



<a name="@Module_Specification_1"></a>

## Module Specification

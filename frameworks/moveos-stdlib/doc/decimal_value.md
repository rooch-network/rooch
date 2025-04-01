
<a id="0x2_decimal_value"></a>

# Module `0x2::decimal_value`



-  [Struct `DecimalValue`](#0x2_decimal_value_DecimalValue)
-  [Constants](#@Constants_0)
-  [Function `new`](#0x2_decimal_value_new)
-  [Function `value`](#0x2_decimal_value_value)
-  [Function `decimal`](#0x2_decimal_value_decimal)
-  [Function `with_precision`](#0x2_decimal_value_with_precision)
-  [Function `is_equal`](#0x2_decimal_value_is_equal)
-  [Function `add`](#0x2_decimal_value_add)
-  [Function `sub`](#0x2_decimal_value_sub)
-  [Function `mul`](#0x2_decimal_value_mul)
-  [Function `div`](#0x2_decimal_value_div)
-  [Function `mul_u256`](#0x2_decimal_value_mul_u256)
-  [Function `div_u256`](#0x2_decimal_value_div_u256)
-  [Function `as_integer_decimal`](#0x2_decimal_value_as_integer_decimal)
-  [Function `to_integer`](#0x2_decimal_value_to_integer)
-  [Function `round`](#0x2_decimal_value_round)
-  [Function `from_string`](#0x2_decimal_value_from_string)
-  [Function `to_string`](#0x2_decimal_value_to_string)


<pre><code><b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="">0x1::string</a>;
<b>use</b> <a href="">0x1::u256</a>;
</code></pre>



<a id="0x2_decimal_value_DecimalValue"></a>

## Struct `DecimalValue`



<pre><code>#[data_struct]
<b>struct</b> <a href="decimal_value.md#0x2_decimal_value_DecimalValue">DecimalValue</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a id="@Constants_0"></a>

## Constants


<a id="0x2_decimal_value_ErrorDecimalPartTooLong"></a>



<pre><code><b>const</b> <a href="decimal_value.md#0x2_decimal_value_ErrorDecimalPartTooLong">ErrorDecimalPartTooLong</a>: u64 = 6;
</code></pre>



<a id="0x2_decimal_value_ErrorDivisionByZero"></a>



<pre><code><b>const</b> <a href="decimal_value.md#0x2_decimal_value_ErrorDivisionByZero">ErrorDivisionByZero</a>: u64 = 2;
</code></pre>



<a id="0x2_decimal_value_ErrorInvalidDecimalString"></a>



<pre><code><b>const</b> <a href="decimal_value.md#0x2_decimal_value_ErrorInvalidDecimalString">ErrorInvalidDecimalString</a>: u64 = 5;
</code></pre>



<a id="0x2_decimal_value_ErrorInvalidPrecision"></a>



<pre><code><b>const</b> <a href="decimal_value.md#0x2_decimal_value_ErrorInvalidPrecision">ErrorInvalidPrecision</a>: u64 = 3;
</code></pre>



<a id="0x2_decimal_value_ErrorOverflow"></a>



<pre><code><b>const</b> <a href="decimal_value.md#0x2_decimal_value_ErrorOverflow">ErrorOverflow</a>: u64 = 4;
</code></pre>



<a id="0x2_decimal_value_ErrorUnderflow"></a>



<pre><code><b>const</b> <a href="decimal_value.md#0x2_decimal_value_ErrorUnderflow">ErrorUnderflow</a>: u64 = 1;
</code></pre>



<a id="0x2_decimal_value_new"></a>

## Function `new`



<pre><code><b>public</b> <b>fun</b> <a href="decimal_value.md#0x2_decimal_value_new">new</a>(value: <a href="">u256</a>, decimal: u8): <a href="decimal_value.md#0x2_decimal_value_DecimalValue">decimal_value::DecimalValue</a>
</code></pre>



<a id="0x2_decimal_value_value"></a>

## Function `value`



<pre><code><b>public</b> <b>fun</b> <a href="decimal_value.md#0x2_decimal_value_value">value</a>(self: &<a href="decimal_value.md#0x2_decimal_value_DecimalValue">decimal_value::DecimalValue</a>): <a href="">u256</a>
</code></pre>



<a id="0x2_decimal_value_decimal"></a>

## Function `decimal`



<pre><code><b>public</b> <b>fun</b> <a href="decimal_value.md#0x2_decimal_value_decimal">decimal</a>(self: &<a href="decimal_value.md#0x2_decimal_value_DecimalValue">decimal_value::DecimalValue</a>): u8
</code></pre>



<a id="0x2_decimal_value_with_precision"></a>

## Function `with_precision`

Create a new DecimalValue with the given decimal precision
For example, convert 1.234 (value=1234, decimal=3) to 1.23400000 (value=123400000, decimal=8)


<pre><code><b>public</b> <b>fun</b> <a href="decimal_value.md#0x2_decimal_value_with_precision">with_precision</a>(self: &<a href="decimal_value.md#0x2_decimal_value_DecimalValue">decimal_value::DecimalValue</a>, new_decimal: u8): <a href="decimal_value.md#0x2_decimal_value_DecimalValue">decimal_value::DecimalValue</a>
</code></pre>



<a id="0x2_decimal_value_is_equal"></a>

## Function `is_equal`

Check if two DecimalValue instances represent the same numerical value


<pre><code><b>public</b> <b>fun</b> <a href="decimal_value.md#0x2_decimal_value_is_equal">is_equal</a>(a: &<a href="decimal_value.md#0x2_decimal_value_DecimalValue">decimal_value::DecimalValue</a>, b: &<a href="decimal_value.md#0x2_decimal_value_DecimalValue">decimal_value::DecimalValue</a>): bool
</code></pre>



<a id="0x2_decimal_value_add"></a>

## Function `add`

Add two DecimalValue instances


<pre><code><b>public</b> <b>fun</b> <a href="decimal_value.md#0x2_decimal_value_add">add</a>(a: &<a href="decimal_value.md#0x2_decimal_value_DecimalValue">decimal_value::DecimalValue</a>, b: &<a href="decimal_value.md#0x2_decimal_value_DecimalValue">decimal_value::DecimalValue</a>): <a href="decimal_value.md#0x2_decimal_value_DecimalValue">decimal_value::DecimalValue</a>
</code></pre>



<a id="0x2_decimal_value_sub"></a>

## Function `sub`

Subtract b from a


<pre><code><b>public</b> <b>fun</b> <a href="decimal_value.md#0x2_decimal_value_sub">sub</a>(a: &<a href="decimal_value.md#0x2_decimal_value_DecimalValue">decimal_value::DecimalValue</a>, b: &<a href="decimal_value.md#0x2_decimal_value_DecimalValue">decimal_value::DecimalValue</a>): <a href="decimal_value.md#0x2_decimal_value_DecimalValue">decimal_value::DecimalValue</a>
</code></pre>



<a id="0x2_decimal_value_mul"></a>

## Function `mul`

Multiply two DecimalValue instances


<pre><code><b>public</b> <b>fun</b> <a href="decimal_value.md#0x2_decimal_value_mul">mul</a>(a: &<a href="decimal_value.md#0x2_decimal_value_DecimalValue">decimal_value::DecimalValue</a>, b: &<a href="decimal_value.md#0x2_decimal_value_DecimalValue">decimal_value::DecimalValue</a>): <a href="decimal_value.md#0x2_decimal_value_DecimalValue">decimal_value::DecimalValue</a>
</code></pre>



<a id="0x2_decimal_value_div"></a>

## Function `div`

Divide a by b


<pre><code><b>public</b> <b>fun</b> <a href="decimal_value.md#0x2_decimal_value_div">div</a>(a: &<a href="decimal_value.md#0x2_decimal_value_DecimalValue">decimal_value::DecimalValue</a>, b: &<a href="decimal_value.md#0x2_decimal_value_DecimalValue">decimal_value::DecimalValue</a>, precision: u8): <a href="decimal_value.md#0x2_decimal_value_DecimalValue">decimal_value::DecimalValue</a>
</code></pre>



<a id="0x2_decimal_value_mul_u256"></a>

## Function `mul_u256`

Multiply by an integer


<pre><code><b>public</b> <b>fun</b> <a href="decimal_value.md#0x2_decimal_value_mul_u256">mul_u256</a>(a: &<a href="decimal_value.md#0x2_decimal_value_DecimalValue">decimal_value::DecimalValue</a>, b: <a href="">u256</a>): <a href="decimal_value.md#0x2_decimal_value_DecimalValue">decimal_value::DecimalValue</a>
</code></pre>



<a id="0x2_decimal_value_div_u256"></a>

## Function `div_u256`

Divide by an integer


<pre><code><b>public</b> <b>fun</b> <a href="decimal_value.md#0x2_decimal_value_div_u256">div_u256</a>(a: &<a href="decimal_value.md#0x2_decimal_value_DecimalValue">decimal_value::DecimalValue</a>, b: <a href="">u256</a>): <a href="decimal_value.md#0x2_decimal_value_DecimalValue">decimal_value::DecimalValue</a>
</code></pre>



<a id="0x2_decimal_value_as_integer_decimal"></a>

## Function `as_integer_decimal`

Convert to integer part represented as a DecimalValue with decimal=0


<pre><code><b>public</b> <b>fun</b> <a href="decimal_value.md#0x2_decimal_value_as_integer_decimal">as_integer_decimal</a>(self: &<a href="decimal_value.md#0x2_decimal_value_DecimalValue">decimal_value::DecimalValue</a>): <a href="decimal_value.md#0x2_decimal_value_DecimalValue">decimal_value::DecimalValue</a>
</code></pre>



<a id="0x2_decimal_value_to_integer"></a>

## Function `to_integer`

Convert to integer by truncating decimal part and returning raw u256


<pre><code><b>public</b> <b>fun</b> <a href="decimal_value.md#0x2_decimal_value_to_integer">to_integer</a>(self: &<a href="decimal_value.md#0x2_decimal_value_DecimalValue">decimal_value::DecimalValue</a>): <a href="">u256</a>
</code></pre>



<a id="0x2_decimal_value_round"></a>

## Function `round`

Round the decimal value to the specified number of decimal places


<pre><code><b>public</b> <b>fun</b> <a href="decimal_value.md#0x2_decimal_value_round">round</a>(self: &<a href="decimal_value.md#0x2_decimal_value_DecimalValue">decimal_value::DecimalValue</a>, new_decimal: u8): <a href="decimal_value.md#0x2_decimal_value_DecimalValue">decimal_value::DecimalValue</a>
</code></pre>



<a id="0x2_decimal_value_from_string"></a>

## Function `from_string`

Parse a string representation of a decimal number into a DecimalValue
Accepts strings like "123", "123.456", "0.123"


<pre><code><b>public</b> <b>fun</b> <a href="decimal_value.md#0x2_decimal_value_from_string">from_string</a>(s: &<a href="_String">string::String</a>): <a href="decimal_value.md#0x2_decimal_value_DecimalValue">decimal_value::DecimalValue</a>
</code></pre>



<a id="0x2_decimal_value_to_string"></a>

## Function `to_string`

Convert a DecimalValue to its string representation
Returns strings like "123", "123.456", "0.123"


<pre><code><b>public</b> <b>fun</b> <a href="decimal_value.md#0x2_decimal_value_to_string">to_string</a>(d: &<a href="decimal_value.md#0x2_decimal_value_DecimalValue">decimal_value::DecimalValue</a>): <a href="_String">string::String</a>
</code></pre>

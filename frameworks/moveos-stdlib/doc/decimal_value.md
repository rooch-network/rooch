
<a name="0x2_decimal_value"></a>

# Module `0x2::decimal_value`



-  [Struct `DecimalValue`](#0x2_decimal_value_DecimalValue)
-  [Function `new`](#0x2_decimal_value_new)
-  [Function `value`](#0x2_decimal_value_value)
-  [Function `decimal`](#0x2_decimal_value_decimal)
-  [Function `is_equal`](#0x2_decimal_value_is_equal)


<pre><code><b>use</b> <a href="">0x1::u256</a>;
</code></pre>



<a name="0x2_decimal_value_DecimalValue"></a>

## Struct `DecimalValue`



<pre><code>#[data_struct]
<b>struct</b> <a href="decimal_value.md#0x2_decimal_value_DecimalValue">DecimalValue</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x2_decimal_value_new"></a>

## Function `new`



<pre><code><b>public</b> <b>fun</b> <a href="decimal_value.md#0x2_decimal_value_new">new</a>(value: <a href="">u256</a>, decimal: u8): <a href="decimal_value.md#0x2_decimal_value_DecimalValue">decimal_value::DecimalValue</a>
</code></pre>



<a name="0x2_decimal_value_value"></a>

## Function `value`



<pre><code><b>public</b> <b>fun</b> <a href="decimal_value.md#0x2_decimal_value_value">value</a>(self: &<a href="decimal_value.md#0x2_decimal_value_DecimalValue">decimal_value::DecimalValue</a>): <a href="">u256</a>
</code></pre>



<a name="0x2_decimal_value_decimal"></a>

## Function `decimal`



<pre><code><b>public</b> <b>fun</b> <a href="decimal_value.md#0x2_decimal_value_decimal">decimal</a>(self: &<a href="decimal_value.md#0x2_decimal_value_DecimalValue">decimal_value::DecimalValue</a>): u8
</code></pre>



<a name="0x2_decimal_value_is_equal"></a>

## Function `is_equal`

Check if two DecimalValue instances represent the same numerical value


<pre><code><b>public</b> <b>fun</b> <a href="decimal_value.md#0x2_decimal_value_is_equal">is_equal</a>(a: &<a href="decimal_value.md#0x2_decimal_value_DecimalValue">decimal_value::DecimalValue</a>, b: &<a href="decimal_value.md#0x2_decimal_value_DecimalValue">decimal_value::DecimalValue</a>): bool
</code></pre>

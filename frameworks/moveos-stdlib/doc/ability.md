
<a name="0x2_ability"></a>

# Module `0x2::ability`



-  [Constants](#@Constants_0)
-  [Function `ability_copy`](#0x2_ability_ability_copy)
-  [Function `ability_drop`](#0x2_ability_ability_drop)
-  [Function `ability_store`](#0x2_ability_ability_store)
-  [Function `ability_key`](#0x2_ability_ability_key)
-  [Function `has_ability`](#0x2_ability_has_ability)
-  [Function `has_copy`](#0x2_ability_has_copy)
-  [Function `has_drop`](#0x2_ability_has_drop)
-  [Function `has_store`](#0x2_ability_has_store)
-  [Function `has_key`](#0x2_ability_has_key)
-  [Function `native_get_abilities`](#0x2_ability_native_get_abilities)


<pre><code><b>use</b> <a href="">0x1::string</a>;
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x2_ability_ABILITY_COPY"></a>



<pre><code><b>const</b> <a href="ability.md#0x2_ability_ABILITY_COPY">ABILITY_COPY</a>: u8 = 1;
</code></pre>



<a name="0x2_ability_ABILITY_DROP"></a>



<pre><code><b>const</b> <a href="ability.md#0x2_ability_ABILITY_DROP">ABILITY_DROP</a>: u8 = 2;
</code></pre>



<a name="0x2_ability_ABILITY_KEY"></a>



<pre><code><b>const</b> <a href="ability.md#0x2_ability_ABILITY_KEY">ABILITY_KEY</a>: u8 = 8;
</code></pre>



<a name="0x2_ability_ABILITY_STORE"></a>



<pre><code><b>const</b> <a href="ability.md#0x2_ability_ABILITY_STORE">ABILITY_STORE</a>: u8 = 4;
</code></pre>



<a name="0x2_ability_ability_copy"></a>

## Function `ability_copy`



<pre><code><b>public</b> <b>fun</b> <a href="ability.md#0x2_ability_ability_copy">ability_copy</a>(): u8
</code></pre>



<a name="0x2_ability_ability_drop"></a>

## Function `ability_drop`



<pre><code><b>public</b> <b>fun</b> <a href="ability.md#0x2_ability_ability_drop">ability_drop</a>(): u8
</code></pre>



<a name="0x2_ability_ability_store"></a>

## Function `ability_store`



<pre><code><b>public</b> <b>fun</b> <a href="ability.md#0x2_ability_ability_store">ability_store</a>(): u8
</code></pre>



<a name="0x2_ability_ability_key"></a>

## Function `ability_key`



<pre><code><b>public</b> <b>fun</b> <a href="ability.md#0x2_ability_ability_key">ability_key</a>(): u8
</code></pre>



<a name="0x2_ability_has_ability"></a>

## Function `has_ability`



<pre><code><b>public</b> <b>fun</b> <a href="ability.md#0x2_ability_has_ability">has_ability</a>(abilities: u8, <a href="ability.md#0x2_ability">ability</a>: u8): bool
</code></pre>



<a name="0x2_ability_has_copy"></a>

## Function `has_copy`



<pre><code><b>public</b> <b>fun</b> <a href="ability.md#0x2_ability_has_copy">has_copy</a>(abilities: u8): bool
</code></pre>



<a name="0x2_ability_has_drop"></a>

## Function `has_drop`



<pre><code><b>public</b> <b>fun</b> <a href="ability.md#0x2_ability_has_drop">has_drop</a>(abilities: u8): bool
</code></pre>



<a name="0x2_ability_has_store"></a>

## Function `has_store`



<pre><code><b>public</b> <b>fun</b> <a href="ability.md#0x2_ability_has_store">has_store</a>(abilities: u8): bool
</code></pre>



<a name="0x2_ability_has_key"></a>

## Function `has_key`



<pre><code><b>public</b> <b>fun</b> <a href="ability.md#0x2_ability_has_key">has_key</a>(abilities: u8): bool
</code></pre>



<a name="0x2_ability_native_get_abilities"></a>

## Function `native_get_abilities`



<pre><code><b>public</b> <b>fun</b> <a href="ability.md#0x2_ability_native_get_abilities">native_get_abilities</a>(type: <a href="_String">string::String</a>): u8
</code></pre>

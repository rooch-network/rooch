
<a name="0x1_ascii"></a>

# Module `0x1::ascii`

The <code>ASCII</code> module defines basic string and char newtypes in Move that verify
that characters are valid ASCII, and that strings consist of only valid ASCII characters.


-  [Struct `String`](#0x1_ascii_String)
-  [Struct `Char`](#0x1_ascii_Char)
-  [Constants](#@Constants_0)
-  [Function `char`](#0x1_ascii_char)
-  [Function `string`](#0x1_ascii_string)
-  [Function `try_string`](#0x1_ascii_try_string)
-  [Function `all_characters_printable`](#0x1_ascii_all_characters_printable)
-  [Function `push_char`](#0x1_ascii_push_char)
-  [Function `pop_char`](#0x1_ascii_pop_char)
-  [Function `length`](#0x1_ascii_length)
-  [Function `as_bytes`](#0x1_ascii_as_bytes)
-  [Function `into_bytes`](#0x1_ascii_into_bytes)
-  [Function `byte`](#0x1_ascii_byte)
-  [Function `is_valid_char`](#0x1_ascii_is_valid_char)
-  [Function `is_printable_char`](#0x1_ascii_is_printable_char)


<pre><code><b>use</b> <a href="option.md#0x1_option">0x1::option</a>;
</code></pre>



<a name="0x1_ascii_String"></a>

## Struct `String`

The <code><a href="ascii.md#0x1_ascii_String">String</a></code> struct holds a vector of bytes that all represent
valid ASCII characters. Note that these ASCII characters may not all
be printable. To determine if a <code><a href="ascii.md#0x1_ascii_String">String</a></code> contains only "printable"
characters you should use the <code>all_characters_printable</code> predicate
defined in this module.


<pre><code><b>struct</b> <a href="ascii.md#0x1_ascii_String">String</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x1_ascii_Char"></a>

## Struct `Char`

An ASCII character.


<pre><code><b>struct</b> <a href="ascii.md#0x1_ascii_Char">Char</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x1_ascii_EINVALID_ASCII_CHARACTER"></a>

An invalid ASCII character was encountered when creating an ASCII string.


<pre><code><b>const</b> <a href="ascii.md#0x1_ascii_EINVALID_ASCII_CHARACTER">EINVALID_ASCII_CHARACTER</a>: u64 = 65536;
</code></pre>



<a name="0x1_ascii_char"></a>

## Function `char`

Convert a <code>byte</code> into a <code><a href="ascii.md#0x1_ascii_Char">Char</a></code> that is checked to make sure it is valid ASCII.


<pre><code><b>public</b> <b>fun</b> <a href="ascii.md#0x1_ascii_char">char</a>(byte: u8): <a href="ascii.md#0x1_ascii_Char">ascii::Char</a>
</code></pre>



<a name="0x1_ascii_string"></a>

## Function `string`

Convert a vector of bytes <code>bytes</code> into an <code><a href="ascii.md#0x1_ascii_String">String</a></code>. Aborts if
<code>bytes</code> contains non-ASCII characters.


<pre><code><b>public</b> <b>fun</b> <a href="string.md#0x1_string">string</a>(bytes: <a href="vector.md#0x1_vector">vector</a>&lt;u8&gt;): <a href="ascii.md#0x1_ascii_String">ascii::String</a>
</code></pre>



<a name="0x1_ascii_try_string"></a>

## Function `try_string`

Convert a vector of bytes <code>bytes</code> into an <code><a href="ascii.md#0x1_ascii_String">String</a></code>. Returns
<code>Some(&lt;ascii_string&gt;)</code> if the <code>bytes</code> contains all valid ASCII
characters. Otherwise returns <code>None</code>.


<pre><code><b>public</b> <b>fun</b> <a href="ascii.md#0x1_ascii_try_string">try_string</a>(bytes: <a href="vector.md#0x1_vector">vector</a>&lt;u8&gt;): <a href="option.md#0x1_option_Option">option::Option</a>&lt;<a href="ascii.md#0x1_ascii_String">ascii::String</a>&gt;
</code></pre>



<a name="0x1_ascii_all_characters_printable"></a>

## Function `all_characters_printable`

Returns <code><b>true</b></code> if all characters in <code><a href="string.md#0x1_string">string</a></code> are printable characters
Returns <code><b>false</b></code> otherwise. Not all <code><a href="ascii.md#0x1_ascii_String">String</a></code>s are printable strings.


<pre><code><b>public</b> <b>fun</b> <a href="ascii.md#0x1_ascii_all_characters_printable">all_characters_printable</a>(<a href="string.md#0x1_string">string</a>: &<a href="ascii.md#0x1_ascii_String">ascii::String</a>): bool
</code></pre>



<a name="0x1_ascii_push_char"></a>

## Function `push_char`



<pre><code><b>public</b> <b>fun</b> <a href="ascii.md#0x1_ascii_push_char">push_char</a>(<a href="string.md#0x1_string">string</a>: &<b>mut</b> <a href="ascii.md#0x1_ascii_String">ascii::String</a>, char: <a href="ascii.md#0x1_ascii_Char">ascii::Char</a>)
</code></pre>



<a name="0x1_ascii_pop_char"></a>

## Function `pop_char`



<pre><code><b>public</b> <b>fun</b> <a href="ascii.md#0x1_ascii_pop_char">pop_char</a>(<a href="string.md#0x1_string">string</a>: &<b>mut</b> <a href="ascii.md#0x1_ascii_String">ascii::String</a>): <a href="ascii.md#0x1_ascii_Char">ascii::Char</a>
</code></pre>



<a name="0x1_ascii_length"></a>

## Function `length`



<pre><code><b>public</b> <b>fun</b> <a href="ascii.md#0x1_ascii_length">length</a>(<a href="string.md#0x1_string">string</a>: &<a href="ascii.md#0x1_ascii_String">ascii::String</a>): u64
</code></pre>



<a name="0x1_ascii_as_bytes"></a>

## Function `as_bytes`

Get the inner bytes of the <code><a href="string.md#0x1_string">string</a></code> as a reference


<pre><code><b>public</b> <b>fun</b> <a href="ascii.md#0x1_ascii_as_bytes">as_bytes</a>(<a href="string.md#0x1_string">string</a>: &<a href="ascii.md#0x1_ascii_String">ascii::String</a>): &<a href="vector.md#0x1_vector">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x1_ascii_into_bytes"></a>

## Function `into_bytes`

Unpack the <code><a href="string.md#0x1_string">string</a></code> to get its backing bytes


<pre><code><b>public</b> <b>fun</b> <a href="ascii.md#0x1_ascii_into_bytes">into_bytes</a>(<a href="string.md#0x1_string">string</a>: <a href="ascii.md#0x1_ascii_String">ascii::String</a>): <a href="vector.md#0x1_vector">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x1_ascii_byte"></a>

## Function `byte`

Unpack the <code>char</code> into its underlying byte.


<pre><code><b>public</b> <b>fun</b> <a href="ascii.md#0x1_ascii_byte">byte</a>(char: <a href="ascii.md#0x1_ascii_Char">ascii::Char</a>): u8
</code></pre>



<a name="0x1_ascii_is_valid_char"></a>

## Function `is_valid_char`

Returns <code><b>true</b></code> if <code>b</code> is a valid ASCII character. Returns <code><b>false</b></code> otherwise.


<pre><code><b>public</b> <b>fun</b> <a href="ascii.md#0x1_ascii_is_valid_char">is_valid_char</a>(b: u8): bool
</code></pre>



<a name="0x1_ascii_is_printable_char"></a>

## Function `is_printable_char`

Returns <code><b>true</b></code> if <code>byte</code> is an printable ASCII character. Returns <code><b>false</b></code> otherwise.


<pre><code><b>public</b> <b>fun</b> <a href="ascii.md#0x1_ascii_is_printable_char">is_printable_char</a>(byte: u8): bool
</code></pre>

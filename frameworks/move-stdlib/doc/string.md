
<a name="0x1_string"></a>

# Module `0x1::string`

The <code><a href="string.md#0x1_string">string</a></code> module defines the <code><a href="string.md#0x1_string_String">String</a></code> type which represents UTF8 encoded strings.


-  [Struct `String`](#0x1_string_String)
-  [Constants](#@Constants_0)
-  [Function `utf8`](#0x1_string_utf8)
-  [Function `from_ascii`](#0x1_string_from_ascii)
-  [Function `to_ascii`](#0x1_string_to_ascii)
-  [Function `try_utf8`](#0x1_string_try_utf8)
-  [Function `bytes`](#0x1_string_bytes)
-  [Function `into_bytes`](#0x1_string_into_bytes)
-  [Function `is_empty`](#0x1_string_is_empty)
-  [Function `length`](#0x1_string_length)
-  [Function `append`](#0x1_string_append)
-  [Function `append_utf8`](#0x1_string_append_utf8)
-  [Function `insert`](#0x1_string_insert)
-  [Function `sub_string`](#0x1_string_sub_string)
-  [Function `index_of`](#0x1_string_index_of)
-  [Function `internal_check_utf8`](#0x1_string_internal_check_utf8)


<pre><code><b>use</b> <a href="ascii.md#0x1_ascii">0x1::ascii</a>;
<b>use</b> <a href="option.md#0x1_option">0x1::option</a>;
<b>use</b> <a href="vector.md#0x1_vector">0x1::vector</a>;
</code></pre>



<a name="0x1_string_String"></a>

## Struct `String`

A <code><a href="string.md#0x1_string_String">String</a></code> holds a sequence of bytes which is guaranteed to be in utf8 format.


<pre><code><b>struct</b> <a href="string.md#0x1_string_String">String</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x1_string_EINVALID_INDEX"></a>

Index out of range.


<pre><code><b>const</b> <a href="string.md#0x1_string_EINVALID_INDEX">EINVALID_INDEX</a>: u64 = 2;
</code></pre>



<a name="0x1_string_EINVALID_UTF8"></a>

An invalid UTF8 encoding.


<pre><code><b>const</b> <a href="string.md#0x1_string_EINVALID_UTF8">EINVALID_UTF8</a>: u64 = 1;
</code></pre>



<a name="0x1_string_utf8"></a>

## Function `utf8`

Creates a new string from a sequence of bytes. Aborts if the bytes do not represent valid utf8.


<pre><code><b>public</b> <b>fun</b> <a href="string.md#0x1_string_utf8">utf8</a>(bytes: <a href="vector.md#0x1_vector">vector</a>&lt;u8&gt;): <a href="string.md#0x1_string_String">string::String</a>
</code></pre>



<a name="0x1_string_from_ascii"></a>

## Function `from_ascii`

Convert an ASCII string to a UTF8 string


<pre><code><b>public</b> <b>fun</b> <a href="string.md#0x1_string_from_ascii">from_ascii</a>(s: <a href="ascii.md#0x1_ascii_String">ascii::String</a>): <a href="string.md#0x1_string_String">string::String</a>
</code></pre>



<a name="0x1_string_to_ascii"></a>

## Function `to_ascii`

Convert an UTF8 string to an ASCII string.
Aborts if <code>s</code> is not valid ASCII


<pre><code><b>public</b> <b>fun</b> <a href="string.md#0x1_string_to_ascii">to_ascii</a>(s: <a href="string.md#0x1_string_String">string::String</a>): <a href="ascii.md#0x1_ascii_String">ascii::String</a>
</code></pre>



<a name="0x1_string_try_utf8"></a>

## Function `try_utf8`

Tries to create a new string from a sequence of bytes.


<pre><code><b>public</b> <b>fun</b> <a href="string.md#0x1_string_try_utf8">try_utf8</a>(bytes: <a href="vector.md#0x1_vector">vector</a>&lt;u8&gt;): <a href="option.md#0x1_option_Option">option::Option</a>&lt;<a href="string.md#0x1_string_String">string::String</a>&gt;
</code></pre>



<a name="0x1_string_bytes"></a>

## Function `bytes`

Returns a reference to the underlying byte vector.


<pre><code><b>public</b> <b>fun</b> <a href="string.md#0x1_string_bytes">bytes</a>(s: &<a href="string.md#0x1_string_String">string::String</a>): &<a href="vector.md#0x1_vector">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x1_string_into_bytes"></a>

## Function `into_bytes`

Unpack the <code><a href="string.md#0x1_string">string</a></code> to get its backing bytes


<pre><code><b>public</b> <b>fun</b> <a href="string.md#0x1_string_into_bytes">into_bytes</a>(<a href="string.md#0x1_string">string</a>: <a href="string.md#0x1_string_String">string::String</a>): <a href="vector.md#0x1_vector">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x1_string_is_empty"></a>

## Function `is_empty`

Checks whether this string is empty.


<pre><code><b>public</b> <b>fun</b> <a href="string.md#0x1_string_is_empty">is_empty</a>(s: &<a href="string.md#0x1_string_String">string::String</a>): bool
</code></pre>



<a name="0x1_string_length"></a>

## Function `length`

Returns the length of this string, in bytes.


<pre><code><b>public</b> <b>fun</b> <a href="string.md#0x1_string_length">length</a>(s: &<a href="string.md#0x1_string_String">string::String</a>): u64
</code></pre>



<a name="0x1_string_append"></a>

## Function `append`

Appends a string.


<pre><code><b>public</b> <b>fun</b> <a href="string.md#0x1_string_append">append</a>(s: &<b>mut</b> <a href="string.md#0x1_string_String">string::String</a>, r: <a href="string.md#0x1_string_String">string::String</a>)
</code></pre>



<a name="0x1_string_append_utf8"></a>

## Function `append_utf8`

Appends bytes which must be in valid utf8 format.


<pre><code><b>public</b> <b>fun</b> <a href="string.md#0x1_string_append_utf8">append_utf8</a>(s: &<b>mut</b> <a href="string.md#0x1_string_String">string::String</a>, bytes: <a href="vector.md#0x1_vector">vector</a>&lt;u8&gt;)
</code></pre>



<a name="0x1_string_insert"></a>

## Function `insert`

Insert the other string at the byte index in given string. The index must be at a valid utf8 char
boundary.


<pre><code><b>public</b> <b>fun</b> <a href="string.md#0x1_string_insert">insert</a>(s: &<b>mut</b> <a href="string.md#0x1_string_String">string::String</a>, at: u64, o: <a href="string.md#0x1_string_String">string::String</a>)
</code></pre>



<a name="0x1_string_sub_string"></a>

## Function `sub_string`

Returns a sub-string using the given byte indices, where <code>i</code> is the first byte position and <code>j</code> is the start
of the first byte not included (or the length of the string). The indices must be at valid utf8 char boundaries,
guaranteeing that the result is valid utf8.


<pre><code><b>public</b> <b>fun</b> <a href="string.md#0x1_string_sub_string">sub_string</a>(s: &<a href="string.md#0x1_string_String">string::String</a>, i: u64, j: u64): <a href="string.md#0x1_string_String">string::String</a>
</code></pre>



<a name="0x1_string_index_of"></a>

## Function `index_of`

Computes the index of the first occurrence of a string. Returns <code><a href="string.md#0x1_string_length">length</a>(s)</code> if no occurrence found.


<pre><code><b>public</b> <b>fun</b> <a href="string.md#0x1_string_index_of">index_of</a>(s: &<a href="string.md#0x1_string_String">string::String</a>, r: &<a href="string.md#0x1_string_String">string::String</a>): u64
</code></pre>



<a name="0x1_string_internal_check_utf8"></a>

## Function `internal_check_utf8`



<pre><code><b>public</b> <b>fun</b> <a href="string.md#0x1_string_internal_check_utf8">internal_check_utf8</a>(v: &<a href="vector.md#0x1_vector">vector</a>&lt;u8&gt;): bool
</code></pre>

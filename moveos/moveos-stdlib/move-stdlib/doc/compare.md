
<a name="0x1_compare"></a>

# Module `0x1::compare`

Utilities for comparing Move values based on their representation in BCS.


-  [Constants](#@Constants_0)
-  [Function `cmp_bcs_bytes`](#0x1_compare_cmp_bcs_bytes)


<pre><code></code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x1_compare_EQUAL"></a>



<pre><code><b>const</b> <a href="compare.md#0x1_compare_EQUAL">EQUAL</a>: u8 = 0;
</code></pre>



<a name="0x1_compare_GREATER_THAN"></a>



<pre><code><b>const</b> <a href="compare.md#0x1_compare_GREATER_THAN">GREATER_THAN</a>: u8 = 2;
</code></pre>



<a name="0x1_compare_LESS_THAN"></a>



<pre><code><b>const</b> <a href="compare.md#0x1_compare_LESS_THAN">LESS_THAN</a>: u8 = 1;
</code></pre>



<a name="0x1_compare_cmp_bcs_bytes"></a>

## Function `cmp_bcs_bytes`

compare vectors <code>v1</code> and <code>v2</code> using (1) vector contents from right to left and then
(2) vector length to break ties.
Returns either <code><a href="compare.md#0x1_compare_EQUAL">EQUAL</a></code> (0u8), <code><a href="compare.md#0x1_compare_LESS_THAN">LESS_THAN</a></code> (1u8), or <code><a href="compare.md#0x1_compare_GREATER_THAN">GREATER_THAN</a></code> (2u8).

This function is designed to compare BCS (Binary Canonical Serialization)-encoded values
(i.e., vectors produced by <code><a href="bcs.md#0x1_bcs_to_bytes">bcs::to_bytes</a></code>). A typical client will call
<code><a href="compare.md#0x1_compare_cmp_bcs_bytes">compare::cmp_bcs_bytes</a>(<a href="bcs.md#0x1_bcs_to_bytes">bcs::to_bytes</a>(&t1), <a href="bcs.md#0x1_bcs_to_bytes">bcs::to_bytes</a>(&t2))</code>. The comparison provides the
following guarantees w.r.t the original values t1 and t2:
- <code><a href="compare.md#0x1_compare_cmp_bcs_bytes">cmp_bcs_bytes</a>(<a href="bcs.md#0x1_bcs">bcs</a>(t1), <a href="bcs.md#0x1_bcs">bcs</a>(t2)) == <a href="compare.md#0x1_compare_LESS_THAN">LESS_THAN</a></code> iff <code><a href="compare.md#0x1_compare_cmp_bcs_bytes">cmp_bcs_bytes</a>(t2, t1) == <a href="compare.md#0x1_compare_GREATER_THAN">GREATER_THAN</a></code>
- <code>compare::cmp&lt;T&gt;(t1, t2) == <a href="compare.md#0x1_compare_EQUAL">EQUAL</a></code> iff <code>t1 == t2</code> and (similarly)
<code>compare::cmp&lt;T&gt;(t1, t2) != <a href="compare.md#0x1_compare_EQUAL">EQUAL</a></code> iff <code>t1 != t2</code>, where <code>==</code> and <code>!=</code> denote the Move
bytecode operations for polymorphic equality.
- for all primitive types <code>T</code> with <code>&lt;</code> and <code>&gt;</code> comparison operators exposed in Move bytecode
(<code>u8</code>, <code>u16</code>, <code>u32</code>, <code>u64</code>, <code>u128</code>, <code>u256</code>), we have
<code>compare_bcs_bytes(<a href="bcs.md#0x1_bcs">bcs</a>(t1), <a href="bcs.md#0x1_bcs">bcs</a>(t2)) == <a href="compare.md#0x1_compare_LESS_THAN">LESS_THAN</a></code> iff <code>t1 &lt; t2</code> and (similarly)
<code>compare_bcs_bytes(<a href="bcs.md#0x1_bcs">bcs</a>(t1), <a href="bcs.md#0x1_bcs">bcs</a>(t2)) == <a href="compare.md#0x1_compare_LESS_THAN">LESS_THAN</a></code> iff <code>t1 &gt; t2</code>.

For all other types, the order is whatever the BCS encoding of the type and the comparison
strategy above gives you. One case where the order might be surprising is the <code><b>address</b></code>
type.
CoreAddresses are 16 byte hex values that BCS encodes with the identity function. The right
to left, byte-by-byte comparison means that (for example)
<code>compare_bcs_bytes(<a href="bcs.md#0x1_bcs">bcs</a>(0x01), <a href="bcs.md#0x1_bcs">bcs</a>(0x10)) == <a href="compare.md#0x1_compare_LESS_THAN">LESS_THAN</a></code> (as you'd expect), but
<code>compare_bcs_bytes(<a href="bcs.md#0x1_bcs">bcs</a>(0x100), <a href="bcs.md#0x1_bcs">bcs</a>(0x001)) == <a href="compare.md#0x1_compare_LESS_THAN">LESS_THAN</a></code> (as you probably wouldn't expect).
Keep this in mind when using this function to compare addresses.

> TODO: there is currently no specification for this function, which causes no problem because it is not yet
> used in the Diem framework. However, should this functionality be needed in specification, a customized
> native abstraction is needed in the prover framework.


<pre><code><b>public</b> <b>fun</b> <a href="compare.md#0x1_compare_cmp_bcs_bytes">cmp_bcs_bytes</a>(v1: &<a href="vector.md#0x1_vector">vector</a>&lt;u8&gt;, v2: &<a href="vector.md#0x1_vector">vector</a>&lt;u8&gt;): u8
</code></pre>

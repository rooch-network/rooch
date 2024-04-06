
<a name="0x2_rlp"></a>

# Module `0x2::rlp`

Utility for converting a Move value to its binary representation in RLP(Recursive Length Prefix)
https://ethereum.org/nl/developers/docs/data-structures-and-encoding/rlp/


-  [Constants](#@Constants_0)
-  [Function `to_bytes`](#0x2_rlp_to_bytes)
-  [Function `from_bytes`](#0x2_rlp_from_bytes)


<pre><code></code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x2_rlp_ErrorRLPDeserializationFailure"></a>



<pre><code><b>const</b> <a href="rlp.md#0x2_rlp_ErrorRLPDeserializationFailure">ErrorRLPDeserializationFailure</a>: u64 = 2;
</code></pre>



<a name="0x2_rlp_ErrorRLPSerializationFailure"></a>



<pre><code><b>const</b> <a href="rlp.md#0x2_rlp_ErrorRLPSerializationFailure">ErrorRLPSerializationFailure</a>: u64 = 1;
</code></pre>



<a name="0x2_rlp_to_bytes"></a>

## Function `to_bytes`



<pre><code><b>public</b> <b>fun</b> <a href="rlp.md#0x2_rlp_to_bytes">to_bytes</a>&lt;MoveValue&gt;(value: &MoveValue): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a name="0x2_rlp_from_bytes"></a>

## Function `from_bytes`



<pre><code>#[data_struct(#[MoveValue])]
<b>public</b> <b>fun</b> <a href="rlp.md#0x2_rlp_from_bytes">from_bytes</a>&lt;MoveValue&gt;(bytes: <a href="">vector</a>&lt;u8&gt;): MoveValue
</code></pre>

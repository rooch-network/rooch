
<a name="0x2_object_id"></a>

# Module `0x2::object_id`

Move object identifiers


-  [Struct `ObjectID`](#0x2_object_id_ObjectID)
-  [Function `address_to_object_id`](#0x2_object_id_address_to_object_id)


<pre><code></code></pre>



<a name="0x2_object_id_ObjectID"></a>

## Struct `ObjectID`



<pre><code><b>struct</b> <a href="object_id.md#0x2_object_id_ObjectID">ObjectID</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>id: <b>address</b></code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x2_object_id_address_to_object_id"></a>

## Function `address_to_object_id`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object_id.md#0x2_object_id_address_to_object_id">address_to_object_id</a>(<b>address</b>: <b>address</b>): <a href="object_id.md#0x2_object_id_ObjectID">object_id::ObjectID</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="object_id.md#0x2_object_id_address_to_object_id">address_to_object_id</a>(<b>address</b>: <b>address</b>): <a href="object_id.md#0x2_object_id_ObjectID">ObjectID</a> {
    <a href="object_id.md#0x2_object_id_ObjectID">ObjectID</a>{id: <b>address</b>}
}
</code></pre>



</details>


<a name="0x3_nft"></a>

# Module `0x3::nft`



-  [Resource `NFT`](#0x3_nft_NFT)
-  [Resource `MutatorRef`](#0x3_nft_MutatorRef)
-  [Resource `BurnerRef`](#0x3_nft_BurnerRef)
-  [Constants](#@Constants_0)
-  [Function `mint`](#0x3_nft_mint)
-  [Function `burn`](#0x3_nft_burn)
-  [Function `generate_mutator_ref`](#0x3_nft_generate_mutator_ref)
-  [Function `destroy_mutator_ref`](#0x3_nft_destroy_mutator_ref)
-  [Function `generate_burner_ref`](#0x3_nft_generate_burner_ref)
-  [Function `destroy_burner_ref`](#0x3_nft_destroy_burner_ref)
-  [Function `assert_nft_exist_of_id`](#0x3_nft_assert_nft_exist_of_id)
-  [Function `assert_nft_exist_of_ref`](#0x3_nft_assert_nft_exist_of_ref)
-  [Function `assert_mutator_exist_of_ref`](#0x3_nft_assert_mutator_exist_of_ref)
-  [Function `assert_mutator_exist_of_id`](#0x3_nft_assert_mutator_exist_of_id)
-  [Function `assert_burner_exist_of_ref`](#0x3_nft_assert_burner_exist_of_ref)
-  [Function `assert_burner_exist_of_id`](#0x3_nft_assert_burner_exist_of_id)
-  [Function `add_display`](#0x3_nft_add_display)
-  [Function `borrow_display`](#0x3_nft_borrow_display)
-  [Function `borrow_mut_display`](#0x3_nft_borrow_mut_display)
-  [Function `remove_display`](#0x3_nft_remove_display)
-  [Function `contains_display`](#0x3_nft_contains_display)
-  [Function `add_extend`](#0x3_nft_add_extend)
-  [Function `borrow_extend`](#0x3_nft_borrow_extend)
-  [Function `borrow_mut_extend`](#0x3_nft_borrow_mut_extend)
-  [Function `remove_extend`](#0x3_nft_remove_extend)
-  [Function `contains_extend`](#0x3_nft_contains_extend)
-  [Function `get_name`](#0x3_nft_get_name)
-  [Function `get_uri`](#0x3_nft_get_uri)
-  [Function `get_collection`](#0x3_nft_get_collection)
-  [Function `get_creator`](#0x3_nft_get_creator)


<pre><code><b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="">0x1::string</a>;
<b>use</b> <a href="">0x2::context</a>;
<b>use</b> <a href="">0x2::object</a>;
<b>use</b> <a href="">0x2::object_ref</a>;
<b>use</b> <a href="">0x2::type_table</a>;
<b>use</b> <a href="collection.md#0x3_collection">0x3::collection</a>;
<b>use</b> <a href="display.md#0x3_display">0x3::display</a>;
</code></pre>



<a name="0x3_nft_NFT"></a>

## Resource `NFT`



<pre><code><b>struct</b> <a href="nft.md#0x3_nft_NFT">NFT</a> <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>name: <a href="_String">string::String</a></code>
</dt>
<dd>

</dd>
<dt>
<code>uri: <a href="_String">string::String</a></code>
</dt>
<dd>

</dd>
<dt>
<code><a href="collection.md#0x3_collection">collection</a>: <a href="_ObjectID">object::ObjectID</a></code>
</dt>
<dd>

</dd>
<dt>
<code>creator: <b>address</b></code>
</dt>
<dd>

</dd>
<dt>
<code>extend: <a href="_TypeTable">type_table::TypeTable</a></code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x3_nft_MutatorRef"></a>

## Resource `MutatorRef`



<pre><code><b>struct</b> <a href="nft.md#0x3_nft_MutatorRef">MutatorRef</a> <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code><a href="nft.md#0x3_nft">nft</a>: <a href="_ObjectID">object::ObjectID</a></code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x3_nft_BurnerRef"></a>

## Resource `BurnerRef`



<pre><code><b>struct</b> <a href="nft.md#0x3_nft_BurnerRef">BurnerRef</a> <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code><a href="nft.md#0x3_nft">nft</a>: <a href="_ObjectID">object::ObjectID</a></code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="@Constants_0"></a>

## Constants


<a name="0x3_nft_EMutatorNotExist"></a>



<pre><code><b>const</b> <a href="nft.md#0x3_nft_EMutatorNotExist">EMutatorNotExist</a>: u64 = 101;
</code></pre>



<a name="0x3_nft_ENftNotExist"></a>



<pre><code><b>const</b> <a href="nft.md#0x3_nft_ENftNotExist">ENftNotExist</a>: u64 = 100;
</code></pre>



<a name="0x3_nft_mint"></a>

## Function `mint`



<pre><code><b>public</b> <b>fun</b> <a href="nft.md#0x3_nft_mint">mint</a>(name: <a href="_String">string::String</a>, uri: <a href="_String">string::String</a>, mutator_ref: &<a href="_ObjectRef">object_ref::ObjectRef</a>&lt;<a href="collection.md#0x3_collection_MutatorRef">collection::MutatorRef</a>&gt;, creator: <b>address</b>, ctx: &<b>mut</b> <a href="_Context">context::Context</a>): <a href="_ObjectRef">object_ref::ObjectRef</a>&lt;<a href="nft.md#0x3_nft_NFT">nft::NFT</a>&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="nft.md#0x3_nft_mint">mint</a>(
    name: String,
    uri: String,
    mutator_ref: &ObjectRef&lt;<a href="collection.md#0x3_collection_MutatorRef">collection::MutatorRef</a>&gt;,
    creator: <b>address</b>,
    ctx: &<b>mut</b> Context
): ObjectRef&lt;<a href="nft.md#0x3_nft_NFT">NFT</a>&gt; {
    <a href="collection.md#0x3_collection_assert_mutator_exist_of_ref">collection::assert_mutator_exist_of_ref</a>(mutator_ref);
    <b>let</b> <a href="nft.md#0x3_nft">nft</a> = <a href="nft.md#0x3_nft_NFT">NFT</a> {
        name,
        uri,
        <a href="collection.md#0x3_collection">collection</a>: <a href="collection.md#0x3_collection_get_collection_id">collection::get_collection_id</a>(mutator_ref),
        creator,
        extend: <a href="_new">type_table::new</a>(ctx)
    };

    <a href="collection.md#0x3_collection_increment_supply">collection::increment_supply</a>(mutator_ref, ctx);

    <b>let</b> <a href="">object_ref</a> = <a href="_new_object_with_owner">context::new_object_with_owner</a>(
        ctx,
        creator,
        <a href="nft.md#0x3_nft">nft</a>
    );

    <a href="">object_ref</a>
}
</code></pre>



</details>

<a name="0x3_nft_burn"></a>

## Function `burn`



<pre><code><b>public</b> <b>fun</b> <a href="nft.md#0x3_nft_burn">burn</a>(burn_ref: &<a href="_ObjectRef">object_ref::ObjectRef</a>&lt;<a href="nft.md#0x3_nft_BurnerRef">nft::BurnerRef</a>&gt;, mutator_ref: &<a href="_ObjectRef">object_ref::ObjectRef</a>&lt;<a href="collection.md#0x3_collection_MutatorRef">collection::MutatorRef</a>&gt;, ctx: &<b>mut</b> <a href="_Context">context::Context</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="nft.md#0x3_nft_burn">burn</a> (
    burn_ref: &ObjectRef&lt;<a href="nft.md#0x3_nft_BurnerRef">BurnerRef</a>&gt;,
    mutator_ref: &ObjectRef&lt;<a href="collection.md#0x3_collection_MutatorRef">collection::MutatorRef</a>&gt;,
    ctx: &<b>mut</b> Context
) {
    <a href="nft.md#0x3_nft_assert_burner_exist_of_ref">assert_burner_exist_of_ref</a>(burn_ref);
    <b>let</b> burner_object_ref = <a href="_borrow">object_ref::borrow</a>(burn_ref);
    <a href="nft.md#0x3_nft_assert_nft_exist_of_id">assert_nft_exist_of_id</a>(burner_object_ref.<a href="nft.md#0x3_nft">nft</a>, ctx);
    <a href="collection.md#0x3_collection_decrement_supply">collection::decrement_supply</a>(mutator_ref, ctx);
    <b>let</b> (
        _,
        _,
        <a href="nft.md#0x3_nft_NFT">NFT</a> {
            name:_,
            uri:_,
            <a href="collection.md#0x3_collection">collection</a>:_,
            creator:_,
            extend
        }
    ) = <a href="_remove_object">context::remove_object</a>&lt;<a href="nft.md#0x3_nft_NFT">NFT</a>&gt;(ctx, burner_object_ref.<a href="nft.md#0x3_nft">nft</a>);
    <b>if</b>(<a href="_contains">type_table::contains</a>&lt;Display&gt;( &extend )){
       <a href="_remove">type_table::remove</a>&lt;Display&gt;( &<b>mut</b> extend);
    };
    <a href="_destroy_empty">type_table::destroy_empty</a>(extend)
}
</code></pre>



</details>

<a name="0x3_nft_generate_mutator_ref"></a>

## Function `generate_mutator_ref`



<pre><code><b>public</b> <b>fun</b> <a href="nft.md#0x3_nft_generate_mutator_ref">generate_mutator_ref</a>(nft_object_ref: &<a href="_ObjectRef">object_ref::ObjectRef</a>&lt;<a href="nft.md#0x3_nft_NFT">nft::NFT</a>&gt;, ctx: &<b>mut</b> <a href="_Context">context::Context</a>): <a href="_ObjectRef">object_ref::ObjectRef</a>&lt;<a href="nft.md#0x3_nft_MutatorRef">nft::MutatorRef</a>&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="nft.md#0x3_nft_generate_mutator_ref">generate_mutator_ref</a>(nft_object_ref: &ObjectRef&lt;<a href="nft.md#0x3_nft_NFT">NFT</a>&gt;, ctx: &<b>mut</b> Context):ObjectRef&lt;<a href="nft.md#0x3_nft_MutatorRef">MutatorRef</a>&gt;{
    <b>let</b> mutator_ref = <a href="_new_object_with_owner">context::new_object_with_owner</a>(
        ctx,
        <a href="_owner">object_ref::owner</a>(nft_object_ref),
        <a href="nft.md#0x3_nft_MutatorRef">MutatorRef</a> {
            <a href="nft.md#0x3_nft">nft</a>: <a href="_id">object_ref::id</a>(nft_object_ref),
        }
    );
    mutator_ref
}
</code></pre>



</details>

<a name="0x3_nft_destroy_mutator_ref"></a>

## Function `destroy_mutator_ref`



<pre><code><b>public</b> <b>fun</b> <a href="nft.md#0x3_nft_destroy_mutator_ref">destroy_mutator_ref</a>(mutator_ref: <a href="_ObjectRef">object_ref::ObjectRef</a>&lt;<a href="nft.md#0x3_nft_MutatorRef">nft::MutatorRef</a>&gt;): <a href="_ObjectID">object::ObjectID</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="nft.md#0x3_nft_destroy_mutator_ref">destroy_mutator_ref</a>(mutator_ref :ObjectRef&lt;<a href="nft.md#0x3_nft_MutatorRef">MutatorRef</a>&gt;):ObjectID{
    <a href="nft.md#0x3_nft_assert_mutator_exist_of_ref">assert_mutator_exist_of_ref</a>(&mutator_ref);
    <b>let</b> <a href="nft.md#0x3_nft_MutatorRef">MutatorRef</a> {
        <a href="nft.md#0x3_nft">nft</a>
    } = <a href="_remove">object_ref::remove</a>&lt;<a href="nft.md#0x3_nft_MutatorRef">MutatorRef</a>&gt;(mutator_ref);
    <a href="nft.md#0x3_nft">nft</a>
}
</code></pre>



</details>

<a name="0x3_nft_generate_burner_ref"></a>

## Function `generate_burner_ref`



<pre><code><b>public</b> <b>fun</b> <a href="nft.md#0x3_nft_generate_burner_ref">generate_burner_ref</a>(nft_object_ref: &<a href="_ObjectRef">object_ref::ObjectRef</a>&lt;<a href="nft.md#0x3_nft_NFT">nft::NFT</a>&gt;, ctx: &<b>mut</b> <a href="_Context">context::Context</a>): <a href="_ObjectRef">object_ref::ObjectRef</a>&lt;<a href="nft.md#0x3_nft_BurnerRef">nft::BurnerRef</a>&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="nft.md#0x3_nft_generate_burner_ref">generate_burner_ref</a>(nft_object_ref: &ObjectRef&lt;<a href="nft.md#0x3_nft_NFT">NFT</a>&gt;, ctx: &<b>mut</b> Context):ObjectRef&lt;<a href="nft.md#0x3_nft_BurnerRef">BurnerRef</a>&gt;{
    <b>let</b> burner_ref = <a href="_new_object_with_owner">context::new_object_with_owner</a>(
        ctx,
        <a href="_owner">object_ref::owner</a>(nft_object_ref),
        <a href="nft.md#0x3_nft_BurnerRef">BurnerRef</a> {
            <a href="nft.md#0x3_nft">nft</a>: <a href="_id">object_ref::id</a>(nft_object_ref),
        }
    );
    burner_ref
}
</code></pre>



</details>

<a name="0x3_nft_destroy_burner_ref"></a>

## Function `destroy_burner_ref`



<pre><code><b>public</b> <b>fun</b> <a href="nft.md#0x3_nft_destroy_burner_ref">destroy_burner_ref</a>(burner_ref: <a href="_ObjectRef">object_ref::ObjectRef</a>&lt;<a href="nft.md#0x3_nft_BurnerRef">nft::BurnerRef</a>&gt;): <a href="_ObjectID">object::ObjectID</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="nft.md#0x3_nft_destroy_burner_ref">destroy_burner_ref</a>(burner_ref :ObjectRef&lt;<a href="nft.md#0x3_nft_BurnerRef">BurnerRef</a>&gt;):ObjectID{
    <a href="nft.md#0x3_nft_assert_burner_exist_of_ref">assert_burner_exist_of_ref</a>(&burner_ref);
    <b>let</b> <a href="nft.md#0x3_nft_BurnerRef">BurnerRef</a> {
        <a href="nft.md#0x3_nft">nft</a>
    } = <a href="_remove">object_ref::remove</a>&lt;<a href="nft.md#0x3_nft_BurnerRef">BurnerRef</a>&gt;(burner_ref);
    <a href="nft.md#0x3_nft">nft</a>
}
</code></pre>



</details>

<a name="0x3_nft_assert_nft_exist_of_id"></a>

## Function `assert_nft_exist_of_id`



<pre><code><b>public</b> <b>fun</b> <a href="nft.md#0x3_nft_assert_nft_exist_of_id">assert_nft_exist_of_id</a>(objectId: <a href="_ObjectID">object::ObjectID</a>, ctx: &<a href="_Context">context::Context</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="nft.md#0x3_nft_assert_nft_exist_of_id">assert_nft_exist_of_id</a>(objectId: ObjectID, ctx: &Context) {
    <b>assert</b>!(<a href="_exist_object">context::exist_object</a>(ctx, objectId), <a href="nft.md#0x3_nft_ENftNotExist">ENftNotExist</a>);
    <a href="_borrow_object">context::borrow_object</a>&lt;<a href="nft.md#0x3_nft_NFT">NFT</a>&gt;(ctx, objectId);
}
</code></pre>



</details>

<a name="0x3_nft_assert_nft_exist_of_ref"></a>

## Function `assert_nft_exist_of_ref`



<pre><code><b>public</b> <b>fun</b> <a href="nft.md#0x3_nft_assert_nft_exist_of_ref">assert_nft_exist_of_ref</a>(nft_object_ref: &<a href="_ObjectRef">object_ref::ObjectRef</a>&lt;<a href="nft.md#0x3_nft_NFT">nft::NFT</a>&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="nft.md#0x3_nft_assert_nft_exist_of_ref">assert_nft_exist_of_ref</a>(nft_object_ref: &ObjectRef&lt;<a href="nft.md#0x3_nft_NFT">NFT</a>&gt;) {
    <b>assert</b>!(<a href="_exist_object">object_ref::exist_object</a>(nft_object_ref), <a href="nft.md#0x3_nft_ENftNotExist">ENftNotExist</a>);
}
</code></pre>



</details>

<a name="0x3_nft_assert_mutator_exist_of_ref"></a>

## Function `assert_mutator_exist_of_ref`



<pre><code><b>public</b> <b>fun</b> <a href="nft.md#0x3_nft_assert_mutator_exist_of_ref">assert_mutator_exist_of_ref</a>(mutator_ref: &<a href="_ObjectRef">object_ref::ObjectRef</a>&lt;<a href="nft.md#0x3_nft_MutatorRef">nft::MutatorRef</a>&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="nft.md#0x3_nft_assert_mutator_exist_of_ref">assert_mutator_exist_of_ref</a>(mutator_ref: &ObjectRef&lt;<a href="nft.md#0x3_nft_MutatorRef">MutatorRef</a>&gt;) {
    <b>assert</b>!(<a href="_exist_object">object_ref::exist_object</a>(mutator_ref), <a href="nft.md#0x3_nft_EMutatorNotExist">EMutatorNotExist</a>);
}
</code></pre>



</details>

<a name="0x3_nft_assert_mutator_exist_of_id"></a>

## Function `assert_mutator_exist_of_id`



<pre><code><b>public</b> <b>fun</b> <a href="nft.md#0x3_nft_assert_mutator_exist_of_id">assert_mutator_exist_of_id</a>(objectId: <a href="_ObjectID">object::ObjectID</a>, ctx: &<a href="_Context">context::Context</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="nft.md#0x3_nft_assert_mutator_exist_of_id">assert_mutator_exist_of_id</a>(objectId: ObjectID, ctx: &Context) {
    <b>assert</b>!(<a href="_exist_object">context::exist_object</a>(ctx, objectId), <a href="nft.md#0x3_nft_EMutatorNotExist">EMutatorNotExist</a>);
    <a href="_borrow_object">context::borrow_object</a>&lt;<a href="nft.md#0x3_nft_MutatorRef">MutatorRef</a>&gt;(ctx, objectId);
}
</code></pre>



</details>

<a name="0x3_nft_assert_burner_exist_of_ref"></a>

## Function `assert_burner_exist_of_ref`



<pre><code><b>public</b> <b>fun</b> <a href="nft.md#0x3_nft_assert_burner_exist_of_ref">assert_burner_exist_of_ref</a>(burner_ref: &<a href="_ObjectRef">object_ref::ObjectRef</a>&lt;<a href="nft.md#0x3_nft_BurnerRef">nft::BurnerRef</a>&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="nft.md#0x3_nft_assert_burner_exist_of_ref">assert_burner_exist_of_ref</a>(burner_ref: &ObjectRef&lt;<a href="nft.md#0x3_nft_BurnerRef">BurnerRef</a>&gt;) {
    <b>assert</b>!(<a href="_exist_object">object_ref::exist_object</a>(burner_ref), <a href="nft.md#0x3_nft_EMutatorNotExist">EMutatorNotExist</a>);
}
</code></pre>



</details>

<a name="0x3_nft_assert_burner_exist_of_id"></a>

## Function `assert_burner_exist_of_id`



<pre><code><b>public</b> <b>fun</b> <a href="nft.md#0x3_nft_assert_burner_exist_of_id">assert_burner_exist_of_id</a>(objectId: <a href="_ObjectID">object::ObjectID</a>, ctx: &<a href="_Context">context::Context</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="nft.md#0x3_nft_assert_burner_exist_of_id">assert_burner_exist_of_id</a>(objectId: ObjectID, ctx: &Context) {
    <b>assert</b>!(<a href="_exist_object">context::exist_object</a>(ctx, objectId), <a href="nft.md#0x3_nft_EMutatorNotExist">EMutatorNotExist</a>);
    <a href="_borrow_object">context::borrow_object</a>&lt;<a href="nft.md#0x3_nft_BurnerRef">BurnerRef</a>&gt;(ctx, objectId);
}
</code></pre>



</details>

<a name="0x3_nft_add_display"></a>

## Function `add_display`



<pre><code><b>public</b> <b>fun</b> <a href="nft.md#0x3_nft_add_display">add_display</a>(mutator: &<a href="_ObjectRef">object_ref::ObjectRef</a>&lt;<a href="nft.md#0x3_nft_MutatorRef">nft::MutatorRef</a>&gt;, <a href="display.md#0x3_display">display</a>: <a href="display.md#0x3_display_Display">display::Display</a>, ctx: &<b>mut</b> <a href="_Context">context::Context</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="nft.md#0x3_nft_add_display">add_display</a>(mutator: &ObjectRef&lt;<a href="nft.md#0x3_nft_MutatorRef">MutatorRef</a>&gt;, <a href="display.md#0x3_display">display</a>: Display, ctx: &<b>mut</b> Context){
    <a href="nft.md#0x3_nft_add_extend_internal">add_extend_internal</a>(mutator, <a href="display.md#0x3_display">display</a>, ctx);
}
</code></pre>



</details>

<a name="0x3_nft_borrow_display"></a>

## Function `borrow_display`



<pre><code><b>public</b> <b>fun</b> <a href="nft.md#0x3_nft_borrow_display">borrow_display</a>(mutator: &<a href="_ObjectRef">object_ref::ObjectRef</a>&lt;<a href="nft.md#0x3_nft_MutatorRef">nft::MutatorRef</a>&gt;, ctx: &<b>mut</b> <a href="_Context">context::Context</a>): &<a href="display.md#0x3_display_Display">display::Display</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="nft.md#0x3_nft_borrow_display">borrow_display</a>(mutator: &ObjectRef&lt;<a href="nft.md#0x3_nft_MutatorRef">MutatorRef</a>&gt;, ctx: &<b>mut</b> Context):&Display{
    <a href="nft.md#0x3_nft_borrow_extend_internal">borrow_extend_internal</a>(mutator, ctx)
}
</code></pre>



</details>

<a name="0x3_nft_borrow_mut_display"></a>

## Function `borrow_mut_display`



<pre><code><b>public</b> <b>fun</b> <a href="nft.md#0x3_nft_borrow_mut_display">borrow_mut_display</a>(mutator: &<a href="_ObjectRef">object_ref::ObjectRef</a>&lt;<a href="nft.md#0x3_nft_MutatorRef">nft::MutatorRef</a>&gt;, ctx: &<b>mut</b> <a href="_Context">context::Context</a>): &<b>mut</b> <a href="display.md#0x3_display_Display">display::Display</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="nft.md#0x3_nft_borrow_mut_display">borrow_mut_display</a>(mutator: &ObjectRef&lt;<a href="nft.md#0x3_nft_MutatorRef">MutatorRef</a>&gt;, ctx: &<b>mut</b> Context):&<b>mut</b> Display{
    <a href="nft.md#0x3_nft_borrow_mut_extend_internal">borrow_mut_extend_internal</a>(mutator, ctx)
}
</code></pre>



</details>

<a name="0x3_nft_remove_display"></a>

## Function `remove_display`



<pre><code><b>public</b> <b>fun</b> <a href="nft.md#0x3_nft_remove_display">remove_display</a>(mutator: &<a href="_ObjectRef">object_ref::ObjectRef</a>&lt;<a href="nft.md#0x3_nft_MutatorRef">nft::MutatorRef</a>&gt;, ctx: &<b>mut</b> <a href="_Context">context::Context</a>): <a href="display.md#0x3_display_Display">display::Display</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="nft.md#0x3_nft_remove_display">remove_display</a>(mutator: &ObjectRef&lt;<a href="nft.md#0x3_nft_MutatorRef">MutatorRef</a>&gt;, ctx: &<b>mut</b> Context):Display{
    <a href="nft.md#0x3_nft_remove_extend_internal">remove_extend_internal</a>(mutator, ctx)
}
</code></pre>



</details>

<a name="0x3_nft_contains_display"></a>

## Function `contains_display`



<pre><code><b>public</b> <b>fun</b> <a href="nft.md#0x3_nft_contains_display">contains_display</a>(mutator: &<a href="_ObjectRef">object_ref::ObjectRef</a>&lt;<a href="nft.md#0x3_nft_MutatorRef">nft::MutatorRef</a>&gt;, ctx: &<b>mut</b> <a href="_Context">context::Context</a>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="nft.md#0x3_nft_contains_display">contains_display</a>(mutator: &ObjectRef&lt;<a href="nft.md#0x3_nft_MutatorRef">MutatorRef</a>&gt;, ctx: &<b>mut</b> Context): bool{
    <a href="nft.md#0x3_nft_contains_extend_internal">contains_extend_internal</a>&lt;Display&gt;(mutator, ctx)
}
</code></pre>



</details>

<a name="0x3_nft_add_extend"></a>

## Function `add_extend`



<pre><code><b>public</b> <b>fun</b> <a href="nft.md#0x3_nft_add_extend">add_extend</a>&lt;V: key&gt;(mutator: &<a href="_ObjectRef">object_ref::ObjectRef</a>&lt;<a href="nft.md#0x3_nft_MutatorRef">nft::MutatorRef</a>&gt;, val: V, ctx: &<b>mut</b> <a href="_Context">context::Context</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="nft.md#0x3_nft_add_extend">add_extend</a>&lt;V: key&gt;(mutator: &ObjectRef&lt;<a href="nft.md#0x3_nft_MutatorRef">MutatorRef</a>&gt;, val: V, ctx: &<b>mut</b> Context){
    <a href="nft.md#0x3_nft_add_extend_internal">add_extend_internal</a>(mutator, val, ctx);
}
</code></pre>



</details>

<a name="0x3_nft_borrow_extend"></a>

## Function `borrow_extend`



<pre><code><b>public</b> <b>fun</b> <a href="nft.md#0x3_nft_borrow_extend">borrow_extend</a>&lt;V: key&gt;(mutator: &<a href="_ObjectRef">object_ref::ObjectRef</a>&lt;<a href="nft.md#0x3_nft_MutatorRef">nft::MutatorRef</a>&gt;, ctx: &<b>mut</b> <a href="_Context">context::Context</a>): &V
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="nft.md#0x3_nft_borrow_extend">borrow_extend</a>&lt;V: key&gt;(mutator: &ObjectRef&lt;<a href="nft.md#0x3_nft_MutatorRef">MutatorRef</a>&gt;, ctx: &<b>mut</b> Context):&V{
    <a href="nft.md#0x3_nft_borrow_extend_internal">borrow_extend_internal</a>(mutator, ctx)
}
</code></pre>



</details>

<a name="0x3_nft_borrow_mut_extend"></a>

## Function `borrow_mut_extend`



<pre><code><b>public</b> <b>fun</b> <a href="nft.md#0x3_nft_borrow_mut_extend">borrow_mut_extend</a>&lt;V: key&gt;(mutator: &<a href="_ObjectRef">object_ref::ObjectRef</a>&lt;<a href="nft.md#0x3_nft_MutatorRef">nft::MutatorRef</a>&gt;, ctx: &<b>mut</b> <a href="_Context">context::Context</a>): &<b>mut</b> V
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="nft.md#0x3_nft_borrow_mut_extend">borrow_mut_extend</a>&lt;V: key&gt;(mutator: &ObjectRef&lt;<a href="nft.md#0x3_nft_MutatorRef">MutatorRef</a>&gt;, ctx: &<b>mut</b> Context):&<b>mut</b> V{
    <a href="nft.md#0x3_nft_borrow_mut_extend_internal">borrow_mut_extend_internal</a>(mutator, ctx)
}
</code></pre>



</details>

<a name="0x3_nft_remove_extend"></a>

## Function `remove_extend`



<pre><code><b>public</b> <b>fun</b> <a href="nft.md#0x3_nft_remove_extend">remove_extend</a>&lt;V: key&gt;(mutator: &<a href="_ObjectRef">object_ref::ObjectRef</a>&lt;<a href="nft.md#0x3_nft_MutatorRef">nft::MutatorRef</a>&gt;, ctx: &<b>mut</b> <a href="_Context">context::Context</a>): V
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="nft.md#0x3_nft_remove_extend">remove_extend</a>&lt;V: key&gt;(mutator: &ObjectRef&lt;<a href="nft.md#0x3_nft_MutatorRef">MutatorRef</a>&gt;, ctx: &<b>mut</b> Context):V{
    <a href="nft.md#0x3_nft_remove_extend_internal">remove_extend_internal</a>(mutator, ctx)
}
</code></pre>



</details>

<a name="0x3_nft_contains_extend"></a>

## Function `contains_extend`



<pre><code><b>public</b> <b>fun</b> <a href="nft.md#0x3_nft_contains_extend">contains_extend</a>&lt;V: key&gt;(mutator: &<a href="_ObjectRef">object_ref::ObjectRef</a>&lt;<a href="nft.md#0x3_nft_MutatorRef">nft::MutatorRef</a>&gt;, ctx: &<b>mut</b> <a href="_Context">context::Context</a>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="nft.md#0x3_nft_contains_extend">contains_extend</a>&lt;V: key&gt;(mutator: &ObjectRef&lt;<a href="nft.md#0x3_nft_MutatorRef">MutatorRef</a>&gt;, ctx: &<b>mut</b> Context): bool{
    <a href="nft.md#0x3_nft_contains_extend_internal">contains_extend_internal</a>&lt;V&gt;(mutator, ctx)
}
</code></pre>



</details>

<a name="0x3_nft_get_name"></a>

## Function `get_name`



<pre><code><b>public</b> <b>fun</b> <a href="nft.md#0x3_nft_get_name">get_name</a>(objectId: <a href="_ObjectID">object::ObjectID</a>, ctx: &<a href="_Context">context::Context</a>): <a href="_String">string::String</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="nft.md#0x3_nft_get_name">get_name</a>(objectId: ObjectID, ctx: &Context): String {
    <a href="nft.md#0x3_nft_assert_nft_exist_of_id">assert_nft_exist_of_id</a>(objectId, ctx);
    <b>let</b> nft_object_ref = <a href="_borrow_object">context::borrow_object</a>&lt;<a href="nft.md#0x3_nft_NFT">NFT</a>&gt;(ctx, objectId);
    <b>let</b> <a href="nft.md#0x3_nft">nft</a> = <a href="_borrow">object::borrow</a>(nft_object_ref);
    <a href="nft.md#0x3_nft">nft</a>.name
}
</code></pre>



</details>

<a name="0x3_nft_get_uri"></a>

## Function `get_uri`



<pre><code><b>public</b> <b>fun</b> <a href="nft.md#0x3_nft_get_uri">get_uri</a>(objectId: <a href="_ObjectID">object::ObjectID</a>, ctx: &<a href="_Context">context::Context</a>): <a href="_String">string::String</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="nft.md#0x3_nft_get_uri">get_uri</a>(objectId: ObjectID, ctx: &Context): String {
    <a href="nft.md#0x3_nft_assert_nft_exist_of_id">assert_nft_exist_of_id</a>(objectId, ctx);
    <b>let</b> nft_object_ref = <a href="_borrow_object">context::borrow_object</a>&lt;<a href="nft.md#0x3_nft_NFT">NFT</a>&gt;(ctx, objectId);
    <b>let</b> <a href="nft.md#0x3_nft">nft</a> = <a href="_borrow">object::borrow</a>(nft_object_ref);
    <a href="nft.md#0x3_nft">nft</a>.uri
}
</code></pre>



</details>

<a name="0x3_nft_get_collection"></a>

## Function `get_collection`



<pre><code><b>public</b> <b>fun</b> <a href="nft.md#0x3_nft_get_collection">get_collection</a>(objectId: <a href="_ObjectID">object::ObjectID</a>, ctx: &<a href="_Context">context::Context</a>): <a href="_ObjectID">object::ObjectID</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="nft.md#0x3_nft_get_collection">get_collection</a>(objectId: ObjectID, ctx: &Context): ObjectID {
    <a href="nft.md#0x3_nft_assert_nft_exist_of_id">assert_nft_exist_of_id</a>(objectId, ctx);
    <b>let</b> nft_object_ref = <a href="_borrow_object">context::borrow_object</a>&lt;<a href="nft.md#0x3_nft_NFT">NFT</a>&gt;(ctx, objectId);
    <b>let</b> <a href="nft.md#0x3_nft">nft</a> = <a href="_borrow">object::borrow</a>(nft_object_ref);
    <a href="nft.md#0x3_nft">nft</a>.<a href="collection.md#0x3_collection">collection</a>
}
</code></pre>



</details>

<a name="0x3_nft_get_creator"></a>

## Function `get_creator`



<pre><code><b>public</b> <b>fun</b> <a href="nft.md#0x3_nft_get_creator">get_creator</a>(objectId: <a href="_ObjectID">object::ObjectID</a>, ctx: &<a href="_Context">context::Context</a>): <b>address</b>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="nft.md#0x3_nft_get_creator">get_creator</a>(objectId: ObjectID, ctx: &Context): <b>address</b> {
    <a href="nft.md#0x3_nft_assert_nft_exist_of_id">assert_nft_exist_of_id</a>(objectId, ctx);
    <b>let</b> nft_object_ref = <a href="_borrow_object">context::borrow_object</a>&lt;<a href="nft.md#0x3_nft_NFT">NFT</a>&gt;(ctx, objectId);
    <b>let</b> <a href="nft.md#0x3_nft">nft</a> = <a href="_borrow">object::borrow</a>(nft_object_ref);
    <a href="nft.md#0x3_nft">nft</a>.creator
}
</code></pre>



</details>

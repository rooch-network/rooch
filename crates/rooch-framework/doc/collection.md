
<a name="0x3_collection"></a>

# Module `0x3::collection`



-  [Resource `Collection`](#0x3_collection_Collection)
-  [Struct `Supply`](#0x3_collection_Supply)
-  [Resource `MutatorRef`](#0x3_collection_MutatorRef)
-  [Struct `CreateCollectionEvent`](#0x3_collection_CreateCollectionEvent)
-  [Constants](#@Constants_0)
-  [Function `create_collection`](#0x3_collection_create_collection)
-  [Function `generate_mutator_ref`](#0x3_collection_generate_mutator_ref)
-  [Function `destroy_mutator_ref`](#0x3_collection_destroy_mutator_ref)
-  [Function `get_collection_id`](#0x3_collection_get_collection_id)
-  [Function `increment_supply`](#0x3_collection_increment_supply)
-  [Function `decrement_supply`](#0x3_collection_decrement_supply)
-  [Function `assert_collection_exist_of_ref`](#0x3_collection_assert_collection_exist_of_ref)
-  [Function `assert_collection_exist_of_id`](#0x3_collection_assert_collection_exist_of_id)
-  [Function `assert_mutator_exist_of_ref`](#0x3_collection_assert_mutator_exist_of_ref)
-  [Function `assert_mutator_exist_of_id`](#0x3_collection_assert_mutator_exist_of_id)
-  [Function `add_display`](#0x3_collection_add_display)
-  [Function `borrow_display`](#0x3_collection_borrow_display)
-  [Function `borrow_mut_display`](#0x3_collection_borrow_mut_display)
-  [Function `remove_display`](#0x3_collection_remove_display)
-  [Function `contains_display`](#0x3_collection_contains_display)
-  [Function `add_extend`](#0x3_collection_add_extend)
-  [Function `borrow_extend`](#0x3_collection_borrow_extend)
-  [Function `borrow_mut_extend`](#0x3_collection_borrow_mut_extend)
-  [Function `remove_extend`](#0x3_collection_remove_extend)
-  [Function `contains_extend`](#0x3_collection_contains_extend)
-  [Function `get_collection_name`](#0x3_collection_get_collection_name)
-  [Function `get_collection_uri`](#0x3_collection_get_collection_uri)
-  [Function `get_collection_creator`](#0x3_collection_get_collection_creator)
-  [Function `get_collection_current_supply`](#0x3_collection_get_collection_current_supply)
-  [Function `get_collection_maximum_supply`](#0x3_collection_get_collection_maximum_supply)


<pre><code><b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="">0x1::string</a>;
<b>use</b> <a href="">0x2::context</a>;
<b>use</b> <a href="">0x2::event</a>;
<b>use</b> <a href="">0x2::object</a>;
<b>use</b> <a href="">0x2::object_ref</a>;
<b>use</b> <a href="">0x2::type_table</a>;
<b>use</b> <a href="display.md#0x3_display">0x3::display</a>;
</code></pre>



<a name="0x3_collection_Collection"></a>

## Resource `Collection`



<pre><code><b>struct</b> <a href="collection.md#0x3_collection_Collection">Collection</a> <b>has</b> key
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
<code>creator: <b>address</b></code>
</dt>
<dd>

</dd>
<dt>
<code>supply: <a href="collection.md#0x3_collection_Supply">collection::Supply</a></code>
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

<a name="0x3_collection_Supply"></a>

## Struct `Supply`



<pre><code><b>struct</b> <a href="collection.md#0x3_collection_Supply">Supply</a> <b>has</b> store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>current: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>maximum: <a href="_Option">option::Option</a>&lt;u64&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x3_collection_MutatorRef"></a>

## Resource `MutatorRef`



<pre><code><b>struct</b> <a href="collection.md#0x3_collection_MutatorRef">MutatorRef</a> <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code><a href="collection.md#0x3_collection">collection</a>: <a href="_ObjectID">object::ObjectID</a></code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x3_collection_CreateCollectionEvent"></a>

## Struct `CreateCollectionEvent`



<pre><code><b>struct</b> <a href="collection.md#0x3_collection_CreateCollectionEvent">CreateCollectionEvent</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>objectID: <a href="_ObjectID">object::ObjectID</a></code>
</dt>
<dd>

</dd>
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
<code>creator: <b>address</b></code>
</dt>
<dd>

</dd>
<dt>
<code>maximum: <a href="_Option">option::Option</a>&lt;u64&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>description: <a href="_String">string::String</a></code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="@Constants_0"></a>

## Constants


<a name="0x3_collection_ECollectionMaximumSupply"></a>



<pre><code><b>const</b> <a href="collection.md#0x3_collection_ECollectionMaximumSupply">ECollectionMaximumSupply</a>: u64 = 102;
</code></pre>



<a name="0x3_collection_ECollectionNotExist"></a>



<pre><code><b>const</b> <a href="collection.md#0x3_collection_ECollectionNotExist">ECollectionNotExist</a>: u64 = 101;
</code></pre>



<a name="0x3_collection_EMutatorNotExist"></a>



<pre><code><b>const</b> <a href="collection.md#0x3_collection_EMutatorNotExist">EMutatorNotExist</a>: u64 = 100;
</code></pre>



<a name="0x3_collection_create_collection"></a>

## Function `create_collection`



<pre><code><b>public</b> <b>fun</b> <a href="collection.md#0x3_collection_create_collection">create_collection</a>(name: <a href="_String">string::String</a>, uri: <a href="_String">string::String</a>, creator: <b>address</b>, description: <a href="_String">string::String</a>, max_supply: <a href="_Option">option::Option</a>&lt;u64&gt;, ctx: &<b>mut</b> <a href="_Context">context::Context</a>): <a href="_ObjectRef">object_ref::ObjectRef</a>&lt;<a href="collection.md#0x3_collection_Collection">collection::Collection</a>&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="collection.md#0x3_collection_create_collection">create_collection</a>(
    name: String,
    uri: String,
    creator: <b>address</b>,
    description: String,
    max_supply: Option&lt;u64&gt;,
    ctx: &<b>mut</b> Context
):ObjectRef&lt;<a href="collection.md#0x3_collection_Collection">Collection</a>&gt; {

    <b>let</b> <a href="collection.md#0x3_collection">collection</a> = <a href="collection.md#0x3_collection_Collection">Collection</a> {
        name,
        uri,
        creator,
        supply: <a href="collection.md#0x3_collection_Supply">Supply</a> {
            current: 0,
            maximum: max_supply,
        },
        extend: <a href="_new">type_table::new</a>(ctx)
    };

    <b>let</b> <a href="">object_ref</a> = <a href="_new_object_with_owner">context::new_object_with_owner</a>(
        ctx,
        creator,
        <a href="collection.md#0x3_collection">collection</a>
    );

    <a href="_emit">event::emit</a>(
        ctx,
        <a href="collection.md#0x3_collection_CreateCollectionEvent">CreateCollectionEvent</a> {
            objectID: <a href="_id">object_ref::id</a>(&<a href="">object_ref</a>),
            name,
            uri,
            creator,
            maximum: max_supply,
            description,
        }
    );
    <a href="">object_ref</a>
}
</code></pre>



</details>

<a name="0x3_collection_generate_mutator_ref"></a>

## Function `generate_mutator_ref`



<pre><code><b>public</b> <b>fun</b> <a href="collection.md#0x3_collection_generate_mutator_ref">generate_mutator_ref</a>(<a href="collection.md#0x3_collection">collection</a>: &<a href="_ObjectRef">object_ref::ObjectRef</a>&lt;<a href="collection.md#0x3_collection_Collection">collection::Collection</a>&gt;, ctx: &<b>mut</b> <a href="_Context">context::Context</a>): <a href="_ObjectRef">object_ref::ObjectRef</a>&lt;<a href="collection.md#0x3_collection_MutatorRef">collection::MutatorRef</a>&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="collection.md#0x3_collection_generate_mutator_ref">generate_mutator_ref</a>(<a href="collection.md#0x3_collection">collection</a>: &ObjectRef&lt;<a href="collection.md#0x3_collection_Collection">Collection</a>&gt;, ctx: &<b>mut</b> Context):ObjectRef&lt;<a href="collection.md#0x3_collection_MutatorRef">MutatorRef</a>&gt;{
    <b>let</b> mutator_ref = <a href="_new_object_with_owner">context::new_object_with_owner</a>(
        ctx,
        <a href="_owner">object_ref::owner</a>(<a href="collection.md#0x3_collection">collection</a>),
        <a href="collection.md#0x3_collection_MutatorRef">MutatorRef</a> {
            <a href="collection.md#0x3_collection">collection</a>: <a href="_id">object_ref::id</a>(<a href="collection.md#0x3_collection">collection</a>),
        }
    );
    mutator_ref
}
</code></pre>



</details>

<a name="0x3_collection_destroy_mutator_ref"></a>

## Function `destroy_mutator_ref`



<pre><code><b>public</b> <b>fun</b> <a href="collection.md#0x3_collection_destroy_mutator_ref">destroy_mutator_ref</a>(mutator_ref: <a href="_ObjectRef">object_ref::ObjectRef</a>&lt;<a href="collection.md#0x3_collection_MutatorRef">collection::MutatorRef</a>&gt;): <a href="_ObjectID">object::ObjectID</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="collection.md#0x3_collection_destroy_mutator_ref">destroy_mutator_ref</a>(mutator_ref :ObjectRef&lt;<a href="collection.md#0x3_collection_MutatorRef">MutatorRef</a>&gt;):ObjectID{
    <a href="collection.md#0x3_collection_assert_mutator_exist_of_ref">assert_mutator_exist_of_ref</a>(&mutator_ref);
    <b>let</b> <a href="collection.md#0x3_collection_MutatorRef">MutatorRef</a> {
        <a href="collection.md#0x3_collection">collection</a>
    } = <a href="_remove">object_ref::remove</a>&lt;<a href="collection.md#0x3_collection_MutatorRef">MutatorRef</a>&gt;(mutator_ref);
    <a href="collection.md#0x3_collection">collection</a>
}
</code></pre>



</details>

<a name="0x3_collection_get_collection_id"></a>

## Function `get_collection_id`



<pre><code><b>public</b> <b>fun</b> <a href="collection.md#0x3_collection_get_collection_id">get_collection_id</a>(mutator: &<a href="_ObjectRef">object_ref::ObjectRef</a>&lt;<a href="collection.md#0x3_collection_MutatorRef">collection::MutatorRef</a>&gt;): <a href="_ObjectID">object::ObjectID</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="collection.md#0x3_collection_get_collection_id">get_collection_id</a>(mutator: &ObjectRef&lt;<a href="collection.md#0x3_collection_MutatorRef">MutatorRef</a>&gt;): ObjectID{
    <a href="collection.md#0x3_collection_assert_mutator_exist_of_ref">assert_mutator_exist_of_ref</a>(mutator);
    <b>let</b> mutator_object_ref = <a href="_borrow">object_ref::borrow</a>(mutator);
    mutator_object_ref.<a href="collection.md#0x3_collection">collection</a>
}
</code></pre>



</details>

<a name="0x3_collection_increment_supply"></a>

## Function `increment_supply`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="collection.md#0x3_collection_increment_supply">increment_supply</a>(mutator: &<a href="_ObjectRef">object_ref::ObjectRef</a>&lt;<a href="collection.md#0x3_collection_MutatorRef">collection::MutatorRef</a>&gt;, ctx: &<b>mut</b> <a href="_Context">context::Context</a>): <a href="_Option">option::Option</a>&lt;u64&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="collection.md#0x3_collection_increment_supply">increment_supply</a>(mutator: &ObjectRef&lt;<a href="collection.md#0x3_collection_MutatorRef">MutatorRef</a>&gt;, ctx: &<b>mut</b> Context): Option&lt;u64&gt;{
    <a href="collection.md#0x3_collection_assert_mutator_exist_of_ref">assert_mutator_exist_of_ref</a>(mutator);
    <b>let</b> mutator_object_ref = <a href="_borrow">object_ref::borrow</a>(mutator);
    <a href="collection.md#0x3_collection_assert_collection_exist_of_id">assert_collection_exist_of_id</a>(mutator_object_ref.<a href="collection.md#0x3_collection">collection</a>, ctx);
    <b>let</b> collection_object_mut_ref = <a href="_borrow_object_mut">context::borrow_object_mut</a>&lt;<a href="collection.md#0x3_collection_Collection">Collection</a>&gt;(ctx, mutator_object_ref.<a href="collection.md#0x3_collection">collection</a>);
    <b>let</b> collection_mut_ref = <a href="_borrow_mut">object::borrow_mut</a>(collection_object_mut_ref);
    collection_mut_ref.supply.current = collection_mut_ref.supply.current + 1;
    <b>if</b>(<a href="_is_some">option::is_some</a>(&collection_mut_ref.supply.maximum)){
        <b>assert</b>!(collection_mut_ref.supply.current &lt;= *<a href="_borrow">option::borrow</a>(&collection_mut_ref.supply.maximum), <a href="collection.md#0x3_collection_ECollectionMaximumSupply">ECollectionMaximumSupply</a>);
        <a href="_some">option::some</a>(collection_mut_ref.supply.current)
    }<b>else</b>{
        <a href="_none">option::none</a>&lt;u64&gt;()
    }
}
</code></pre>



</details>

<a name="0x3_collection_decrement_supply"></a>

## Function `decrement_supply`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="collection.md#0x3_collection_decrement_supply">decrement_supply</a>(mutator: &<a href="_ObjectRef">object_ref::ObjectRef</a>&lt;<a href="collection.md#0x3_collection_MutatorRef">collection::MutatorRef</a>&gt;, ctx: &<b>mut</b> <a href="_Context">context::Context</a>): <a href="_Option">option::Option</a>&lt;u64&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> (<b>friend</b>) <b>fun</b> <a href="collection.md#0x3_collection_decrement_supply">decrement_supply</a>(mutator: &ObjectRef&lt;<a href="collection.md#0x3_collection_MutatorRef">MutatorRef</a>&gt;, ctx: &<b>mut</b> Context): Option&lt;u64&gt;{
    <a href="collection.md#0x3_collection_assert_mutator_exist_of_ref">assert_mutator_exist_of_ref</a>(mutator);
    <b>let</b> mutator_object_ref = <a href="_borrow">object_ref::borrow</a>(mutator);
    <a href="collection.md#0x3_collection_assert_collection_exist_of_id">assert_collection_exist_of_id</a>(mutator_object_ref.<a href="collection.md#0x3_collection">collection</a>, ctx);
    <b>let</b> collection_object_mut_ref = <a href="_borrow_object_mut">context::borrow_object_mut</a>&lt;<a href="collection.md#0x3_collection_Collection">Collection</a>&gt;(ctx, mutator_object_ref.<a href="collection.md#0x3_collection">collection</a>);
    <b>let</b> collection_mut_ref = <a href="_borrow_mut">object::borrow_mut</a>(collection_object_mut_ref);
    collection_mut_ref.supply.current = collection_mut_ref.supply.current - 1;
    <b>if</b>(<a href="_is_some">option::is_some</a>(&collection_mut_ref.supply.maximum)){
        <a href="_some">option::some</a>(collection_mut_ref.supply.current)
    }<b>else</b>{
        <a href="_none">option::none</a>&lt;u64&gt;()
    }
}
</code></pre>



</details>

<a name="0x3_collection_assert_collection_exist_of_ref"></a>

## Function `assert_collection_exist_of_ref`



<pre><code><b>public</b> <b>fun</b> <a href="collection.md#0x3_collection_assert_collection_exist_of_ref">assert_collection_exist_of_ref</a>(collectionRef: &<a href="_ObjectRef">object_ref::ObjectRef</a>&lt;<a href="collection.md#0x3_collection_Collection">collection::Collection</a>&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="collection.md#0x3_collection_assert_collection_exist_of_ref">assert_collection_exist_of_ref</a>(collectionRef: &ObjectRef&lt;<a href="collection.md#0x3_collection_Collection">Collection</a>&gt;){
    <b>assert</b>!( <a href="_exist_object">object_ref::exist_object</a>(collectionRef), <a href="collection.md#0x3_collection_ECollectionNotExist">ECollectionNotExist</a>);
}
</code></pre>



</details>

<a name="0x3_collection_assert_collection_exist_of_id"></a>

## Function `assert_collection_exist_of_id`



<pre><code><b>public</b> <b>fun</b> <a href="collection.md#0x3_collection_assert_collection_exist_of_id">assert_collection_exist_of_id</a>(collectionID: <a href="_ObjectID">object::ObjectID</a>, ctx: &<a href="_Context">context::Context</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="collection.md#0x3_collection_assert_collection_exist_of_id">assert_collection_exist_of_id</a>(collectionID: ObjectID, ctx: & Context){
    <b>assert</b>!( <a href="_exist_object">context::exist_object</a>(ctx, collectionID), <a href="collection.md#0x3_collection_ECollectionNotExist">ECollectionNotExist</a>);
    <a href="_borrow_object">context::borrow_object</a>&lt;<a href="collection.md#0x3_collection_Collection">Collection</a>&gt;(ctx,collectionID);
}
</code></pre>



</details>

<a name="0x3_collection_assert_mutator_exist_of_ref"></a>

## Function `assert_mutator_exist_of_ref`



<pre><code><b>public</b> <b>fun</b> <a href="collection.md#0x3_collection_assert_mutator_exist_of_ref">assert_mutator_exist_of_ref</a>(mutatorRef: &<a href="_ObjectRef">object_ref::ObjectRef</a>&lt;<a href="collection.md#0x3_collection_MutatorRef">collection::MutatorRef</a>&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="collection.md#0x3_collection_assert_mutator_exist_of_ref">assert_mutator_exist_of_ref</a>(mutatorRef: &ObjectRef&lt;<a href="collection.md#0x3_collection_MutatorRef">MutatorRef</a>&gt;){
    <b>assert</b>!( <a href="_exist_object">object_ref::exist_object</a>(mutatorRef), <a href="collection.md#0x3_collection_EMutatorNotExist">EMutatorNotExist</a>);
}
</code></pre>



</details>

<a name="0x3_collection_assert_mutator_exist_of_id"></a>

## Function `assert_mutator_exist_of_id`



<pre><code><b>public</b> <b>fun</b> <a href="collection.md#0x3_collection_assert_mutator_exist_of_id">assert_mutator_exist_of_id</a>(mutatorID: <a href="_ObjectID">object::ObjectID</a>, ctx: &<a href="_Context">context::Context</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="collection.md#0x3_collection_assert_mutator_exist_of_id">assert_mutator_exist_of_id</a>(mutatorID: ObjectID, ctx: & Context){
    <b>assert</b>!( <a href="_exist_object">context::exist_object</a>(ctx, mutatorID), <a href="collection.md#0x3_collection_EMutatorNotExist">EMutatorNotExist</a>);
    <a href="_borrow_object">context::borrow_object</a>&lt;<a href="collection.md#0x3_collection_MutatorRef">MutatorRef</a>&gt;(ctx, mutatorID);
}
</code></pre>



</details>

<a name="0x3_collection_add_display"></a>

## Function `add_display`



<pre><code><b>public</b> <b>fun</b> <a href="collection.md#0x3_collection_add_display">add_display</a>(mutator: &<a href="_ObjectRef">object_ref::ObjectRef</a>&lt;<a href="collection.md#0x3_collection_MutatorRef">collection::MutatorRef</a>&gt;, <a href="display.md#0x3_display">display</a>: <a href="display.md#0x3_display_Display">display::Display</a>, ctx: &<b>mut</b> <a href="_Context">context::Context</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="collection.md#0x3_collection_add_display">add_display</a>(mutator: &ObjectRef&lt;<a href="collection.md#0x3_collection_MutatorRef">MutatorRef</a>&gt;, <a href="display.md#0x3_display">display</a>: Display, ctx: &<b>mut</b> Context){
    <a href="collection.md#0x3_collection_add_extend_internal">add_extend_internal</a>(mutator, <a href="display.md#0x3_display">display</a>, ctx);
}
</code></pre>



</details>

<a name="0x3_collection_borrow_display"></a>

## Function `borrow_display`



<pre><code><b>public</b> <b>fun</b> <a href="collection.md#0x3_collection_borrow_display">borrow_display</a>(mutator: &<a href="_ObjectRef">object_ref::ObjectRef</a>&lt;<a href="collection.md#0x3_collection_MutatorRef">collection::MutatorRef</a>&gt;, ctx: &<b>mut</b> <a href="_Context">context::Context</a>): &<a href="display.md#0x3_display_Display">display::Display</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="collection.md#0x3_collection_borrow_display">borrow_display</a>(mutator: &ObjectRef&lt;<a href="collection.md#0x3_collection_MutatorRef">MutatorRef</a>&gt;, ctx: &<b>mut</b> Context):&Display{
    <a href="collection.md#0x3_collection_borrow_extend_internal">borrow_extend_internal</a>(mutator, ctx)
}
</code></pre>



</details>

<a name="0x3_collection_borrow_mut_display"></a>

## Function `borrow_mut_display`



<pre><code><b>public</b> <b>fun</b> <a href="collection.md#0x3_collection_borrow_mut_display">borrow_mut_display</a>(mutator: &<a href="_ObjectRef">object_ref::ObjectRef</a>&lt;<a href="collection.md#0x3_collection_MutatorRef">collection::MutatorRef</a>&gt;, ctx: &<b>mut</b> <a href="_Context">context::Context</a>): &<b>mut</b> <a href="display.md#0x3_display_Display">display::Display</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="collection.md#0x3_collection_borrow_mut_display">borrow_mut_display</a>(mutator: &ObjectRef&lt;<a href="collection.md#0x3_collection_MutatorRef">MutatorRef</a>&gt;, ctx: &<b>mut</b> Context):&<b>mut</b> Display{
    <a href="collection.md#0x3_collection_borrow_mut_extend_internal">borrow_mut_extend_internal</a>(mutator, ctx)
}
</code></pre>



</details>

<a name="0x3_collection_remove_display"></a>

## Function `remove_display`



<pre><code><b>public</b> <b>fun</b> <a href="collection.md#0x3_collection_remove_display">remove_display</a>(mutator: &<a href="_ObjectRef">object_ref::ObjectRef</a>&lt;<a href="collection.md#0x3_collection_MutatorRef">collection::MutatorRef</a>&gt;, ctx: &<b>mut</b> <a href="_Context">context::Context</a>): <a href="display.md#0x3_display_Display">display::Display</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="collection.md#0x3_collection_remove_display">remove_display</a>(mutator: &ObjectRef&lt;<a href="collection.md#0x3_collection_MutatorRef">MutatorRef</a>&gt;, ctx: &<b>mut</b> Context):Display{
    <a href="collection.md#0x3_collection_remove_extend_internal">remove_extend_internal</a>(mutator, ctx)
}
</code></pre>



</details>

<a name="0x3_collection_contains_display"></a>

## Function `contains_display`



<pre><code><b>public</b> <b>fun</b> <a href="collection.md#0x3_collection_contains_display">contains_display</a>(mutator: &<a href="_ObjectRef">object_ref::ObjectRef</a>&lt;<a href="collection.md#0x3_collection_MutatorRef">collection::MutatorRef</a>&gt;, ctx: &<b>mut</b> <a href="_Context">context::Context</a>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="collection.md#0x3_collection_contains_display">contains_display</a>(mutator: &ObjectRef&lt;<a href="collection.md#0x3_collection_MutatorRef">MutatorRef</a>&gt;, ctx: &<b>mut</b> Context): bool{
    <a href="collection.md#0x3_collection_contains_extend_internal">contains_extend_internal</a>&lt;Display&gt;(mutator, ctx)
}
</code></pre>



</details>

<a name="0x3_collection_add_extend"></a>

## Function `add_extend`



<pre><code><b>public</b> <b>fun</b> <a href="collection.md#0x3_collection_add_extend">add_extend</a>&lt;V: key&gt;(mutator: &<a href="_ObjectRef">object_ref::ObjectRef</a>&lt;<a href="collection.md#0x3_collection_MutatorRef">collection::MutatorRef</a>&gt;, val: V, ctx: &<b>mut</b> <a href="_Context">context::Context</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="collection.md#0x3_collection_add_extend">add_extend</a>&lt;V: key&gt;(mutator: &ObjectRef&lt;<a href="collection.md#0x3_collection_MutatorRef">MutatorRef</a>&gt;, val: V, ctx: &<b>mut</b> Context){
    <a href="collection.md#0x3_collection_add_extend_internal">add_extend_internal</a>(mutator, val, ctx);
}
</code></pre>



</details>

<a name="0x3_collection_borrow_extend"></a>

## Function `borrow_extend`



<pre><code><b>public</b> <b>fun</b> <a href="collection.md#0x3_collection_borrow_extend">borrow_extend</a>&lt;V: key&gt;(mutator: &<a href="_ObjectRef">object_ref::ObjectRef</a>&lt;<a href="collection.md#0x3_collection_MutatorRef">collection::MutatorRef</a>&gt;, ctx: &<b>mut</b> <a href="_Context">context::Context</a>): &V
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="collection.md#0x3_collection_borrow_extend">borrow_extend</a>&lt;V: key&gt;(mutator: &ObjectRef&lt;<a href="collection.md#0x3_collection_MutatorRef">MutatorRef</a>&gt;, ctx: &<b>mut</b> Context):&V{
    <a href="collection.md#0x3_collection_borrow_extend_internal">borrow_extend_internal</a>(mutator, ctx)
}
</code></pre>



</details>

<a name="0x3_collection_borrow_mut_extend"></a>

## Function `borrow_mut_extend`



<pre><code><b>public</b> <b>fun</b> <a href="collection.md#0x3_collection_borrow_mut_extend">borrow_mut_extend</a>&lt;V: key&gt;(mutator: &<a href="_ObjectRef">object_ref::ObjectRef</a>&lt;<a href="collection.md#0x3_collection_MutatorRef">collection::MutatorRef</a>&gt;, ctx: &<b>mut</b> <a href="_Context">context::Context</a>): &<b>mut</b> V
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="collection.md#0x3_collection_borrow_mut_extend">borrow_mut_extend</a>&lt;V: key&gt;(mutator: &ObjectRef&lt;<a href="collection.md#0x3_collection_MutatorRef">MutatorRef</a>&gt;, ctx: &<b>mut</b> Context):&<b>mut</b> V{
    <a href="collection.md#0x3_collection_borrow_mut_extend_internal">borrow_mut_extend_internal</a>(mutator, ctx)
}
</code></pre>



</details>

<a name="0x3_collection_remove_extend"></a>

## Function `remove_extend`



<pre><code><b>public</b> <b>fun</b> <a href="collection.md#0x3_collection_remove_extend">remove_extend</a>&lt;V: key&gt;(mutator: &<a href="_ObjectRef">object_ref::ObjectRef</a>&lt;<a href="collection.md#0x3_collection_MutatorRef">collection::MutatorRef</a>&gt;, ctx: &<b>mut</b> <a href="_Context">context::Context</a>): V
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="collection.md#0x3_collection_remove_extend">remove_extend</a>&lt;V: key&gt;(mutator: &ObjectRef&lt;<a href="collection.md#0x3_collection_MutatorRef">MutatorRef</a>&gt;, ctx: &<b>mut</b> Context):V{
    <a href="collection.md#0x3_collection_remove_extend_internal">remove_extend_internal</a>(mutator, ctx)
}
</code></pre>



</details>

<a name="0x3_collection_contains_extend"></a>

## Function `contains_extend`



<pre><code><b>public</b> <b>fun</b> <a href="collection.md#0x3_collection_contains_extend">contains_extend</a>&lt;V: key&gt;(mutator: &<a href="_ObjectRef">object_ref::ObjectRef</a>&lt;<a href="collection.md#0x3_collection_MutatorRef">collection::MutatorRef</a>&gt;, ctx: &<b>mut</b> <a href="_Context">context::Context</a>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="collection.md#0x3_collection_contains_extend">contains_extend</a>&lt;V: key&gt;(mutator: &ObjectRef&lt;<a href="collection.md#0x3_collection_MutatorRef">MutatorRef</a>&gt;, ctx: &<b>mut</b> Context): bool{
    <a href="collection.md#0x3_collection_contains_extend_internal">contains_extend_internal</a>&lt;V&gt;(mutator, ctx)
}
</code></pre>



</details>

<a name="0x3_collection_get_collection_name"></a>

## Function `get_collection_name`



<pre><code><b>public</b> <b>fun</b> <a href="collection.md#0x3_collection_get_collection_name">get_collection_name</a>(collectionID: <a href="_ObjectID">object::ObjectID</a>, ctx: &<b>mut</b> <a href="_Context">context::Context</a>): <a href="_String">string::String</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="collection.md#0x3_collection_get_collection_name">get_collection_name</a>(collectionID: ObjectID, ctx: &<b>mut</b> Context): String{
    <a href="collection.md#0x3_collection_assert_collection_exist_of_id">assert_collection_exist_of_id</a>(collectionID, ctx);
    <b>let</b> collection_object_ref = <a href="_borrow_object">context::borrow_object</a>&lt;<a href="collection.md#0x3_collection_Collection">Collection</a>&gt;(ctx, collectionID);
    <b>let</b> collection_ref = <a href="_borrow">object::borrow</a>(collection_object_ref);
    collection_ref.name
}
</code></pre>



</details>

<a name="0x3_collection_get_collection_uri"></a>

## Function `get_collection_uri`



<pre><code><b>public</b> <b>fun</b> <a href="collection.md#0x3_collection_get_collection_uri">get_collection_uri</a>(collectionID: <a href="_ObjectID">object::ObjectID</a>, ctx: &<b>mut</b> <a href="_Context">context::Context</a>): <a href="_String">string::String</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="collection.md#0x3_collection_get_collection_uri">get_collection_uri</a>(collectionID: ObjectID, ctx: &<b>mut</b> Context): String{
    <a href="collection.md#0x3_collection_assert_collection_exist_of_id">assert_collection_exist_of_id</a>(collectionID, ctx);
    <b>let</b> collection_object_ref = <a href="_borrow_object">context::borrow_object</a>&lt;<a href="collection.md#0x3_collection_Collection">Collection</a>&gt;(ctx, collectionID);
    <b>let</b> collection_ref = <a href="_borrow">object::borrow</a>(collection_object_ref);
    collection_ref.uri
}
</code></pre>



</details>

<a name="0x3_collection_get_collection_creator"></a>

## Function `get_collection_creator`



<pre><code><b>public</b> <b>fun</b> <a href="collection.md#0x3_collection_get_collection_creator">get_collection_creator</a>(collectionID: <a href="_ObjectID">object::ObjectID</a>, ctx: &<b>mut</b> <a href="_Context">context::Context</a>): <b>address</b>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="collection.md#0x3_collection_get_collection_creator">get_collection_creator</a>(collectionID: ObjectID, ctx: &<b>mut</b> Context): <b>address</b>{
    <a href="collection.md#0x3_collection_assert_collection_exist_of_id">assert_collection_exist_of_id</a>(collectionID, ctx);
    <b>let</b> collection_object_ref = <a href="_borrow_object">context::borrow_object</a>&lt;<a href="collection.md#0x3_collection_Collection">Collection</a>&gt;(ctx, collectionID);
    <b>let</b> collection_ref = <a href="_borrow">object::borrow</a>(collection_object_ref);
    collection_ref.creator
}
</code></pre>



</details>

<a name="0x3_collection_get_collection_current_supply"></a>

## Function `get_collection_current_supply`



<pre><code><b>public</b> <b>fun</b> <a href="collection.md#0x3_collection_get_collection_current_supply">get_collection_current_supply</a>(collectionID: <a href="_ObjectID">object::ObjectID</a>, ctx: &<b>mut</b> <a href="_Context">context::Context</a>): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="collection.md#0x3_collection_get_collection_current_supply">get_collection_current_supply</a>(collectionID: ObjectID, ctx: &<b>mut</b> Context): u64{
    <a href="collection.md#0x3_collection_assert_collection_exist_of_id">assert_collection_exist_of_id</a>(collectionID, ctx);
    <b>let</b> collection_object_ref = <a href="_borrow_object">context::borrow_object</a>&lt;<a href="collection.md#0x3_collection_Collection">Collection</a>&gt;(ctx, collectionID);
    <b>let</b> collection_ref = <a href="_borrow">object::borrow</a>(collection_object_ref);
    collection_ref.supply.current
}
</code></pre>



</details>

<a name="0x3_collection_get_collection_maximum_supply"></a>

## Function `get_collection_maximum_supply`



<pre><code><b>public</b> <b>fun</b> <a href="collection.md#0x3_collection_get_collection_maximum_supply">get_collection_maximum_supply</a>(collectionID: <a href="_ObjectID">object::ObjectID</a>, ctx: &<b>mut</b> <a href="_Context">context::Context</a>): <a href="_Option">option::Option</a>&lt;u64&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="collection.md#0x3_collection_get_collection_maximum_supply">get_collection_maximum_supply</a>(collectionID: ObjectID, ctx: &<b>mut</b> Context): Option&lt;u64&gt;{
    <a href="collection.md#0x3_collection_assert_collection_exist_of_id">assert_collection_exist_of_id</a>(collectionID, ctx);
    <b>let</b> collection_object_ref = <a href="_borrow_object">context::borrow_object</a>&lt;<a href="collection.md#0x3_collection_Collection">Collection</a>&gt;(ctx, collectionID);
    <b>let</b> collection_ref = <a href="_borrow">object::borrow</a>(collection_object_ref);
    collection_ref.supply.maximum
}
</code></pre>



</details>

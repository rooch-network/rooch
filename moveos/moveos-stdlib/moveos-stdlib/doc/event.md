
<a name="0x2_event"></a>

# Module `0x2::event`

<code><a href="event.md#0x2_event_EventHandle">EventHandle</a></code>s with unique event handle id (GUID). It contains a counter for the number
of <code><a href="event.md#0x2_event_EventHandle">EventHandle</a></code>s it generates. An <code><a href="event.md#0x2_event_EventHandle">EventHandle</a></code> is used to count the number of
events emitted to a handle and emit events to the event store.


-  [Resource `EventHandle`](#0x2_event_EventHandle)
-  [Function `derive_event_handle_id`](#0x2_event_derive_event_handle_id)
-  [Function `get_event_handle`](#0x2_event_get_event_handle)
-  [Function `ensure_event_handle`](#0x2_event_ensure_event_handle)
-  [Function `emit`](#0x2_event_emit)


<pre><code><b>use</b> <a href="">0x1::hash</a>;
<b>use</b> <a href="bcs.md#0x2_bcs">0x2::bcs</a>;
<b>use</b> <a href="object.md#0x2_object">0x2::object</a>;
<b>use</b> <a href="object_id.md#0x2_object_id">0x2::object_id</a>;
<b>use</b> <a href="object_storage.md#0x2_object_storage">0x2::object_storage</a>;
<b>use</b> <a href="storage_context.md#0x2_storage_context">0x2::storage_context</a>;
<b>use</b> <a href="tx_context.md#0x2_tx_context">0x2::tx_context</a>;
<b>use</b> <a href="type_info.md#0x2_type_info">0x2::type_info</a>;
</code></pre>



<a name="0x2_event_EventHandle"></a>

## Resource `EventHandle`

A handle for an event such that:
1. Other modules can emit events to this handle.
2. Storage can use this handle to prove the total number of events that happened in the past.


<pre><code><b>struct</b> <a href="event.md#0x2_event_EventHandle">EventHandle</a> <b>has</b> store, key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>counter: u64</code>
</dt>
<dd>
 Total number of events emitted to this event stream.
</dd>
</dl>


</details>

<a name="0x2_event_derive_event_handle_id"></a>

## Function `derive_event_handle_id`

A globally unique ID for this event stream. event handler id equal to guid.


<pre><code><b>public</b> <b>fun</b> <a href="event.md#0x2_event_derive_event_handle_id">derive_event_handle_id</a>&lt;T&gt;(): <a href="object_id.md#0x2_object_id_ObjectID">object_id::ObjectID</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="event.md#0x2_event_derive_event_handle_id">derive_event_handle_id</a>&lt;T&gt;(): ObjectID {
    <b>let</b> <a href="type_info.md#0x2_type_info">type_info</a> = <a href="type_info.md#0x2_type_info_type_of">type_info::type_of</a>&lt;T&gt;();
    <b>let</b> event_handle_address = bcs::to_address(<a href="_sha3_256">hash::sha3_256</a>(<a href="_to_bytes">bcs::to_bytes</a>(&<a href="type_info.md#0x2_type_info">type_info</a>)));
    <a href="object_id.md#0x2_object_id_address_to_object_id">object_id::address_to_object_id</a>(event_handle_address)
}
</code></pre>



</details>

<a name="0x2_event_get_event_handle"></a>

## Function `get_event_handle`

use query this method to get event handle Metadata
is event_handle_id doesn't exist, sender will default 0x0


<pre><code><b>public</b> <b>fun</b> <a href="event.md#0x2_event_get_event_handle">get_event_handle</a>&lt;T&gt;(ctx: &<a href="storage_context.md#0x2_storage_context_StorageContext">storage_context::StorageContext</a>): (<a href="object_id.md#0x2_object_id_ObjectID">object_id::ObjectID</a>, <b>address</b>, u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="event.md#0x2_event_get_event_handle">get_event_handle</a>&lt;T&gt;(ctx: &StorageContext): (ObjectID, <b>address</b>, u64) {
    <b>let</b> event_handle_id = <a href="event.md#0x2_event_derive_event_handle_id">derive_event_handle_id</a>&lt;T&gt;();
    <b>let</b> sender = @0x0;
    <b>let</b> event_seq = 0;
    <b>if</b> (<a href="event.md#0x2_event_exists_event_handle">exists_event_handle</a>&lt;T&gt;(<a href="storage_context.md#0x2_storage_context_object_storage">storage_context::object_storage</a>(ctx))) {
        <b>let</b> event_handle = <a href="event.md#0x2_event_borrow_event_handle">borrow_event_handle</a>&lt;T&gt;(
            <a href="storage_context.md#0x2_storage_context_object_storage">storage_context::object_storage</a>(ctx)
        );
        event_seq = event_handle.counter;
        sender = <a href="event.md#0x2_event_get_event_handle_owner">get_event_handle_owner</a>&lt;T&gt;(<a href="storage_context.md#0x2_storage_context_object_storage">storage_context::object_storage</a>(ctx));
    };
    (event_handle_id, sender, event_seq)
}
</code></pre>



</details>

<a name="0x2_event_ensure_event_handle"></a>

## Function `ensure_event_handle`



<pre><code><b>public</b> <b>fun</b> <a href="event.md#0x2_event_ensure_event_handle">ensure_event_handle</a>&lt;T&gt;(ctx: &<b>mut</b> <a href="storage_context.md#0x2_storage_context_StorageContext">storage_context::StorageContext</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="event.md#0x2_event_ensure_event_handle">ensure_event_handle</a>&lt;T&gt;(ctx: &<b>mut</b> StorageContext) {
    <b>if</b> (!<a href="event.md#0x2_event_exists_event_handle">exists_event_handle</a>&lt;T&gt;(<a href="storage_context.md#0x2_storage_context_object_storage">storage_context::object_storage</a>(ctx))) {
        <a href="event.md#0x2_event_new_event_handle">new_event_handle</a>&lt;T&gt;(ctx);
    }
}
</code></pre>



</details>

<a name="0x2_event_emit"></a>

## Function `emit`

Emit a custom Move event, sending the data offchain.

Used for creating custom indexes and tracking onchain
activity in a way that suits a specific application the most.

The type T is the main way to index the event, and can contain
phantom parameters, eg emit(MyEvent<phantom T>).


<pre><code><b>public</b> <b>fun</b> <a href="event.md#0x2_event_emit">emit</a>&lt;T&gt;(ctx: &<b>mut</b> <a href="storage_context.md#0x2_storage_context_StorageContext">storage_context::StorageContext</a>, <a href="event.md#0x2_event">event</a>: T)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="event.md#0x2_event_emit">emit</a>&lt;T&gt;(ctx: &<b>mut</b> StorageContext, <a href="event.md#0x2_event">event</a>: T) {
    <a href="event.md#0x2_event_ensure_event_handle">ensure_event_handle</a>&lt;T&gt;(ctx);
    <b>let</b> event_handle_id = <a href="event.md#0x2_event_derive_event_handle_id">derive_event_handle_id</a>&lt;T&gt;();
    <b>let</b> event_handle_ref = <a href="event.md#0x2_event_borrow_event_handle_mut">borrow_event_handle_mut</a>&lt;T&gt;(
        <a href="storage_context.md#0x2_storage_context_object_storage_mut">storage_context::object_storage_mut</a>(ctx)
    );
    <a href="event.md#0x2_event_native_emit">native_emit</a>&lt;T&gt;(&event_handle_id, event_handle_ref.counter, <a href="event.md#0x2_event">event</a>);
    event_handle_ref.counter = event_handle_ref.counter + 1;
}
</code></pre>



</details>

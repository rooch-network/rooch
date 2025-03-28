
<a id="0x2_event"></a>

# Module `0x2::event`



-  [Function `named_event_handle_id`](#0x2_event_named_event_handle_id)
-  [Function `custom_event_handle_id`](#0x2_event_custom_event_handle_id)
-  [Function `emit`](#0x2_event_emit)
-  [Function `emit_with_handle`](#0x2_event_emit_with_handle)


<pre><code><b>use</b> <a href="object.md#0x2_object">0x2::object</a>;
</code></pre>



<a id="0x2_event_named_event_handle_id"></a>

## Function `named_event_handle_id`



<pre><code><b>public</b> <b>fun</b> <a href="event.md#0x2_event_named_event_handle_id">named_event_handle_id</a>&lt;T&gt;(): <a href="object.md#0x2_object_ObjectID">object::ObjectID</a>
</code></pre>



<a id="0x2_event_custom_event_handle_id"></a>

## Function `custom_event_handle_id`



<pre><code><b>public</b> <b>fun</b> <a href="event.md#0x2_event_custom_event_handle_id">custom_event_handle_id</a>&lt;ID: <b>copy</b>, drop, store, T: key&gt;(id: ID): <a href="object.md#0x2_object_ObjectID">object::ObjectID</a>
</code></pre>



<a id="0x2_event_emit"></a>

## Function `emit`

Emit a custom Move event, sending the data offchain.

Used for creating custom indexes and tracking onchain
activity in a way that suits a specific application the most.

The type T is the main way to index the event, and can contain
phantom parameters, eg. emit(MyEvent<phantom T>).


<pre><code>#[private_generics(#[T])]
<b>public</b> <b>fun</b> <a href="event.md#0x2_event_emit">emit</a>&lt;T: <b>copy</b>, drop&gt;(<a href="event.md#0x2_event">event</a>: T)
</code></pre>



<a id="0x2_event_emit_with_handle"></a>

## Function `emit_with_handle`

Emit a custom Move event with handle


<pre><code>#[private_generics(#[T])]
<b>public</b> <b>fun</b> <a href="event.md#0x2_event_emit_with_handle">emit_with_handle</a>&lt;T: <b>copy</b>, drop&gt;(event_handle_id: <a href="object.md#0x2_object_ObjectID">object::ObjectID</a>, <a href="event.md#0x2_event">event</a>: T)
</code></pre>

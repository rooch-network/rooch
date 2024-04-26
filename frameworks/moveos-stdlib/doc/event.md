
<a name="0x2_event"></a>

# Module `0x2::event`



-  [Function `emit`](#0x2_event_emit)


<pre><code></code></pre>



<a name="0x2_event_emit"></a>

## Function `emit`

Emit a custom Move event, sending the data offchain.

Used for creating custom indexes and tracking onchain
activity in a way that suits a specific application the most.

The type T is the main way to index the event, and can contain
phantom parameters, eg. emit(MyEvent<phantom T>).


<pre><code>#[private_generics(#[T])]
<b>public</b> <b>fun</b> <a href="event.md#0x2_event_emit">emit</a>&lt;T: <b>copy</b>, drop&gt;(<a href="event.md#0x2_event">event</a>: T)
</code></pre>

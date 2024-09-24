
<a name="0x2_event_queue"></a>

# Module `0x2::event_queue`



-  [Resource `EventQueue`](#0x2_event_queue_EventQueue)
-  [Struct `OnChainEvent`](#0x2_event_queue_OnChainEvent)
-  [Struct `OffChainEvent`](#0x2_event_queue_OffChainEvent)
-  [Resource `Subscriber`](#0x2_event_queue_Subscriber)
-  [Constants](#@Constants_0)
-  [Function `emit`](#0x2_event_queue_emit)
-  [Function `consume`](#0x2_event_queue_consume)
-  [Function `subscribe`](#0x2_event_queue_subscribe)
-  [Function `unsubscribe`](#0x2_event_queue_unsubscribe)
-  [Function `remove_expired_events`](#0x2_event_queue_remove_expired_events)
-  [Function `subscriber_info`](#0x2_event_queue_subscriber_info)
-  [Function `exists_new_events`](#0x2_event_queue_exists_new_events)


<pre><code><b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="">0x1::string</a>;
<b>use</b> <a href="event.md#0x2_event">0x2::event</a>;
<b>use</b> <a href="object.md#0x2_object">0x2::object</a>;
<b>use</b> <a href="timestamp.md#0x2_timestamp">0x2::timestamp</a>;
</code></pre>



<a name="0x2_event_queue_EventQueue"></a>

## Resource `EventQueue`



<pre><code><b>struct</b> <a href="event_queue.md#0x2_event_queue_EventQueue">EventQueue</a>&lt;E&gt; <b>has</b> key
</code></pre>



<a name="0x2_event_queue_OnChainEvent"></a>

## Struct `OnChainEvent`



<pre><code><b>struct</b> <a href="event_queue.md#0x2_event_queue_OnChainEvent">OnChainEvent</a>&lt;E&gt; <b>has</b> drop, store
</code></pre>



<a name="0x2_event_queue_OffChainEvent"></a>

## Struct `OffChainEvent`

The off-chain event
Every on-chain event also trigger an off-chain event


<pre><code><b>struct</b> <a href="event_queue.md#0x2_event_queue_OffChainEvent">OffChainEvent</a>&lt;E&gt; <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a name="0x2_event_queue_Subscriber"></a>

## Resource `Subscriber`



<pre><code><b>struct</b> <a href="event_queue.md#0x2_event_queue_Subscriber">Subscriber</a>&lt;E&gt; <b>has</b> key
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x2_event_queue_EVENT_EXPIRE_TIME"></a>



<pre><code><b>const</b> <a href="event_queue.md#0x2_event_queue_EVENT_EXPIRE_TIME">EVENT_EXPIRE_TIME</a>: u64 = 2678400000;
</code></pre>



<a name="0x2_event_queue_ErrorEventNotFound"></a>



<pre><code><b>const</b> <a href="event_queue.md#0x2_event_queue_ErrorEventNotFound">ErrorEventNotFound</a>: u64 = 3;
</code></pre>



<a name="0x2_event_queue_ErrorEventQueueNotFound"></a>



<pre><code><b>const</b> <a href="event_queue.md#0x2_event_queue_ErrorEventQueueNotFound">ErrorEventQueueNotFound</a>: u64 = 5;
</code></pre>



<a name="0x2_event_queue_ErrorInvalidSequenceNumber"></a>



<pre><code><b>const</b> <a href="event_queue.md#0x2_event_queue_ErrorInvalidSequenceNumber">ErrorInvalidSequenceNumber</a>: u64 = 4;
</code></pre>



<a name="0x2_event_queue_ErrorSubscriberNotFound"></a>



<pre><code><b>const</b> <a href="event_queue.md#0x2_event_queue_ErrorSubscriberNotFound">ErrorSubscriberNotFound</a>: u64 = 2;
</code></pre>



<a name="0x2_event_queue_ErrorTooManySubscribers"></a>



<pre><code><b>const</b> <a href="event_queue.md#0x2_event_queue_ErrorTooManySubscribers">ErrorTooManySubscribers</a>: u64 = 1;
</code></pre>



<a name="0x2_event_queue_REMOVE_EXPIRED_EVENT_BATCH_SIZE"></a>



<pre><code><b>const</b> <a href="event_queue.md#0x2_event_queue_REMOVE_EXPIRED_EVENT_BATCH_SIZE">REMOVE_EXPIRED_EVENT_BATCH_SIZE</a>: u64 = 10;
</code></pre>



<a name="0x2_event_queue_emit"></a>

## Function `emit`

Emit an event to the event queue, the event will be stored in the event queue
But if there are no subscribers, we do not store the event


<pre><code>#[private_generics(#[E])]
<b>public</b> <b>fun</b> <a href="event_queue.md#0x2_event_queue_emit">emit</a>&lt;E: <b>copy</b>, drop, store&gt;(name: <a href="_String">string::String</a>, <a href="event.md#0x2_event">event</a>: E)
</code></pre>



<a name="0x2_event_queue_consume"></a>

## Function `consume`

Consume the event from the event queue


<pre><code><b>public</b> <b>fun</b> <a href="event_queue.md#0x2_event_queue_consume">consume</a>&lt;E: <b>copy</b>, drop, store&gt;(subscriber_obj: &<b>mut</b> <a href="object.md#0x2_object_Object">object::Object</a>&lt;<a href="event_queue.md#0x2_event_queue_Subscriber">event_queue::Subscriber</a>&lt;E&gt;&gt;): <a href="_Option">option::Option</a>&lt;E&gt;
</code></pre>



<a name="0x2_event_queue_subscribe"></a>

## Function `subscribe`

Subscribe the event queue of <code>E</code> and the given queue name
Return the subscriber object


<pre><code><b>public</b> <b>fun</b> <a href="event_queue.md#0x2_event_queue_subscribe">subscribe</a>&lt;E: <b>copy</b>, drop, store&gt;(queue_name: <a href="_String">string::String</a>): <a href="object.md#0x2_object_Object">object::Object</a>&lt;<a href="event_queue.md#0x2_event_queue_Subscriber">event_queue::Subscriber</a>&lt;E&gt;&gt;
</code></pre>



<a name="0x2_event_queue_unsubscribe"></a>

## Function `unsubscribe`

Unsubscribe the subscriber


<pre><code><b>public</b> <b>fun</b> <a href="event_queue.md#0x2_event_queue_unsubscribe">unsubscribe</a>&lt;E: <b>copy</b>, drop, store&gt;(subscriber: <a href="object.md#0x2_object_Object">object::Object</a>&lt;<a href="event_queue.md#0x2_event_queue_Subscriber">event_queue::Subscriber</a>&lt;E&gt;&gt;)
</code></pre>



<a name="0x2_event_queue_remove_expired_events"></a>

## Function `remove_expired_events`

Remove the expired events from the event queue
Anyone can call this function to remove the expired events


<pre><code><b>public</b> <b>fun</b> <a href="event_queue.md#0x2_event_queue_remove_expired_events">remove_expired_events</a>&lt;E: <b>copy</b>, drop, store&gt;(queue_name: <a href="_String">string::String</a>)
</code></pre>



<a name="0x2_event_queue_subscriber_info"></a>

## Function `subscriber_info`



<pre><code><b>public</b> <b>fun</b> <a href="event_queue.md#0x2_event_queue_subscriber_info">subscriber_info</a>&lt;E: <b>copy</b>, drop, store&gt;(subscriber_obj: &<a href="object.md#0x2_object_Object">object::Object</a>&lt;<a href="event_queue.md#0x2_event_queue_Subscriber">event_queue::Subscriber</a>&lt;E&gt;&gt;): (u64, u64, u64)
</code></pre>



<a name="0x2_event_queue_exists_new_events"></a>

## Function `exists_new_events`



<pre><code><b>public</b> <b>fun</b> <a href="event_queue.md#0x2_event_queue_exists_new_events">exists_new_events</a>&lt;E: <b>copy</b>, drop, store&gt;(subscriber_obj: &<a href="object.md#0x2_object_Object">object::Object</a>&lt;<a href="event_queue.md#0x2_event_queue_Subscriber">event_queue::Subscriber</a>&lt;E&gt;&gt;): bool
</code></pre>


<a name="0x2_event_queue"></a>

# Module `0x2::event_queue`



-  [Resource `EventQueue`](#0x2_event_queue_EventQueue)
-  [Struct `OnChainEvent`](#0x2_event_queue_OnChainEvent)
-  [Resource `Subscriber`](#0x2_event_queue_Subscriber)
-  [Constants](#@Constants_0)
-  [Function `emit`](#0x2_event_queue_emit)
-  [Function `consume`](#0x2_event_queue_consume)
-  [Function `subscribe`](#0x2_event_queue_subscribe)
-  [Function `subscribe_with_sequence_number`](#0x2_event_queue_subscribe_with_sequence_number)
-  [Function `unsubscribe`](#0x2_event_queue_unsubscribe)
-  [Function `remove_expired_events`](#0x2_event_queue_remove_expired_events)


<pre><code><b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="">0x1::string</a>;
<b>use</b> <a href="">0x1::vector</a>;
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



<pre><code><b>const</b> <a href="event_queue.md#0x2_event_queue_REMOVE_EXPIRED_EVENT_BATCH_SIZE">REMOVE_EXPIRED_EVENT_BATCH_SIZE</a>: u64 = 100;
</code></pre>



<a name="0x2_event_queue_SUBSCRIBERS_KEY"></a>



<pre><code><b>const</b> <a href="event_queue.md#0x2_event_queue_SUBSCRIBERS_KEY">SUBSCRIBERS_KEY</a>: <a href="">vector</a>&lt;u8&gt; = [115, 117, 98, 115, 99, 114, 105, 98, 101, 114, 115];
</code></pre>



<a name="0x2_event_queue_emit"></a>

## Function `emit`



<pre><code>#[private_generics(#[E])]
<b>public</b> <b>fun</b> <a href="event_queue.md#0x2_event_queue_emit">emit</a>&lt;E: <b>copy</b>, drop, store&gt;(name: <a href="_String">string::String</a>, <a href="event.md#0x2_event">event</a>: E)
</code></pre>



<a name="0x2_event_queue_consume"></a>

## Function `consume`



<pre><code><b>public</b> <b>fun</b> <a href="event_queue.md#0x2_event_queue_consume">consume</a>&lt;E: <b>copy</b>, drop, store&gt;(subscriber_obj: &<b>mut</b> <a href="object.md#0x2_object_Object">object::Object</a>&lt;<a href="event_queue.md#0x2_event_queue_Subscriber">event_queue::Subscriber</a>&lt;E&gt;&gt;): <a href="_Option">option::Option</a>&lt;E&gt;
</code></pre>



<a name="0x2_event_queue_subscribe"></a>

## Function `subscribe`



<pre><code><b>public</b> <b>fun</b> <a href="event_queue.md#0x2_event_queue_subscribe">subscribe</a>&lt;E: <b>copy</b>, drop, store&gt;(queue_name: <a href="_String">string::String</a>): <a href="object.md#0x2_object_Object">object::Object</a>&lt;<a href="event_queue.md#0x2_event_queue_Subscriber">event_queue::Subscriber</a>&lt;E&gt;&gt;
</code></pre>



<a name="0x2_event_queue_subscribe_with_sequence_number"></a>

## Function `subscribe_with_sequence_number`



<pre><code><b>public</b> <b>fun</b> <a href="event_queue.md#0x2_event_queue_subscribe_with_sequence_number">subscribe_with_sequence_number</a>&lt;E: <b>copy</b>, drop, store&gt;(queue_name: <a href="_String">string::String</a>, sequence_number: u64): <a href="object.md#0x2_object_Object">object::Object</a>&lt;<a href="event_queue.md#0x2_event_queue_Subscriber">event_queue::Subscriber</a>&lt;E&gt;&gt;
</code></pre>



<a name="0x2_event_queue_unsubscribe"></a>

## Function `unsubscribe`



<pre><code><b>public</b> <b>fun</b> <a href="event_queue.md#0x2_event_queue_unsubscribe">unsubscribe</a>&lt;E: <b>copy</b>, drop, store&gt;(subscriber: <a href="object.md#0x2_object_Object">object::Object</a>&lt;<a href="event_queue.md#0x2_event_queue_Subscriber">event_queue::Subscriber</a>&lt;E&gt;&gt;)
</code></pre>



<a name="0x2_event_queue_remove_expired_events"></a>

## Function `remove_expired_events`

Remove the expired events from the event queue
Anyone can call this function to remove the expired events


<pre><code><b>public</b> <b>fun</b> <a href="event_queue.md#0x2_event_queue_remove_expired_events">remove_expired_events</a>&lt;E: <b>copy</b>, drop, store&gt;(event_queue_obj: &<b>mut</b> <a href="object.md#0x2_object_Object">object::Object</a>&lt;<a href="event_queue.md#0x2_event_queue_EventQueue">event_queue::EventQueue</a>&lt;E&gt;&gt;)
</code></pre>

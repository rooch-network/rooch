// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_nursery::event_queue {
    
    use std::vector;
    use std::option::{Self, Option};
    use std::string::String;
    use moveos_std::timestamp;
    use moveos_std::object::{Self, Object, ObjectID};

    const MAX_SUBSCRIBER_COUNT: u64 = 100;
    const MAX_EVENT_HISTORY: u64 = 1000;
    const SUBSCRIBERS_KEY: vector<u8> = b"subscribers";

    const ErrorTooManySubscribers: u64 = 1;
    const ErrorSubscriberNotFound: u64 = 2;
    const ErrorEventNotFound: u64 = 3;

    struct EventQueue<phantom E> has key{
        sequence_number: u64,
    }

    struct OnChainEvent<E> has store, drop {
        event: E,
        emit_time: u64,
        subscriber_count: u64,
        consumed_count: u64,
    }

    struct Subscriber<phantom E> has key{
        /// The event queue name
        queue_name: String,
        /// The subscriber consume sequence number
        sequence_number: u64,
    }    

    fun event_queue<E : copy + drop + store>(name: String): &mut Object<EventQueue<E>> {
        let object_id = object::custom_object_id<String, EventQueue<E>>(name);
        if (!object::exists_object(object_id)) {
            let event_queue_obj = object::new_with_id(name, EventQueue<E> {
                sequence_number: 0,
            });
            let subscribers : vector<ObjectID> = vector::empty();
            object::add_field(&mut event_queue_obj, SUBSCRIBERS_KEY, subscribers);
            //We transfer the display object to the moveos_std
            //And the caller do not need to care about the event queue object
            object::transfer_extend(event_queue_obj, @rooch_nursery);
        };
        object::borrow_mut_object_extend<EventQueue<E>>(object_id)
    }

    fun borrow_subscribers<E>(event_queue_obj: &Object<EventQueue<E>>) : &vector<ObjectID>{
        object::borrow_field(event_queue_obj, SUBSCRIBERS_KEY)
    }

    fun borrow_mut_subscribers<E>(event_queue_obj: &mut Object<EventQueue<E>>) : &mut vector<ObjectID>{
        object::borrow_mut_field(event_queue_obj, SUBSCRIBERS_KEY)
    }

    #[private_generics(E)]
    public fun emit<E : copy + drop + store>(name: String, event: E) {
        let event_queue_obj = event_queue<E>(name);
        let sequence_number = object::borrow(event_queue_obj).sequence_number;
        let subscribers = borrow_subscribers(event_queue_obj);
        //If there is no subscriber, we do not need to store the event
        if (!vector::is_empty(subscribers)) {
            let now = timestamp::now_milliseconds();
            let on_chain_event = OnChainEvent {
                event: event,
                emit_time: now,
                subscriber_count: vector::length(subscribers),
                consumed_count: 0,
            };
            object::add_field(event_queue_obj, sequence_number, on_chain_event);
            //Remove the oldest event if the event history is full
            if (sequence_number > MAX_EVENT_HISTORY) {
                let oldest_sequence_number = sequence_number - MAX_EVENT_HISTORY;
                if (object::contains_field(event_queue_obj, oldest_sequence_number)) {
                    let _event: OnChainEvent<E> = object::remove_field(event_queue_obj, oldest_sequence_number);
                }
            };
        };
        object::borrow_mut(event_queue_obj).sequence_number = sequence_number + 1;
    }

    public fun consume<E: copy + drop + store>(subscriber_obj: &mut Object<Subscriber<E>>) : Option<E> {
        let subscriber = object::borrow_mut(subscriber_obj);
        let subscriber_sequence_number = subscriber.sequence_number;
        let event_queue_obj = event_queue<E>(subscriber.queue_name);
        let sequence_number = object::borrow(event_queue_obj).sequence_number;
        
        if (subscriber_sequence_number >= sequence_number) {
            return option::none()
        };
        subscriber_sequence_number = if (sequence_number - subscriber_sequence_number > MAX_EVENT_HISTORY) {
            sequence_number - MAX_EVENT_HISTORY
        } else {
            subscriber_sequence_number
        };
        assert!(object::contains_field(event_queue_obj, subscriber_sequence_number), ErrorEventNotFound);
        let on_chain_event: &mut OnChainEvent<E> = object::borrow_mut_field(event_queue_obj, subscriber_sequence_number);
        let consumed_count = on_chain_event.consumed_count;
        subscriber.sequence_number = subscriber_sequence_number + 1;
        on_chain_event.consumed_count = consumed_count + 1;
        if (on_chain_event.consumed_count == on_chain_event.subscriber_count) {
            let on_chain_event: OnChainEvent<E> = object::remove_field(event_queue_obj, subscriber_sequence_number);
            option::some(on_chain_event.event)
        }else{
            option::some(on_chain_event.event)
        }
    }

    public fun subscribe<E: copy + drop + store>(queue_name: String) : Object<Subscriber<E>> {
        let event_queue_obj = event_queue<E>(queue_name);
        let sequence_number = object::borrow(event_queue_obj).sequence_number;
        let subscribers = borrow_mut_subscribers(event_queue_obj);
        assert!(vector::length(subscribers) < MAX_SUBSCRIBER_COUNT, ErrorTooManySubscribers);
        let subscriber = object::new(Subscriber {
            queue_name,
            sequence_number,
        });
        vector::push_back(subscribers, object::id(&subscriber));
        subscriber
    }

    public fun unsubscribe<E: copy + drop + store>(subscriber: Object<Subscriber<E>>) {
        //Consume all events before unsubscribe
        consumer_all(&mut subscriber);
        let subscriber_id = object::id(&subscriber);
        let Subscriber{sequence_number:_, queue_name} = object::remove(subscriber);
        let event_queue_obj = event_queue<E>(queue_name);
        let subscribers = borrow_mut_subscribers(event_queue_obj);
        
        let (find,index) = vector::index_of(subscribers, &subscriber_id);
        assert!(find, ErrorSubscriberNotFound);
        vector::remove(subscribers, index);
    }

    fun consumer_all<E: copy + drop + store>(subscriber: &mut Object<Subscriber<E>>){
        let event = consume(subscriber);
        while (option::is_some(&event)){
            event = consume(subscriber);
        }
    }

    #[test_only]
    struct TestEvent has copy, drop, store{
        value: u64,
    }

    #[test]
    fun test_event_queue_basic(){
        let queue_name = std::string::utf8(b"test_event_queue");
        let subscriber = subscribe<TestEvent>(queue_name);
        emit(queue_name, TestEvent{value: 1});
        let event = consume(&mut subscriber);
        assert!(option::is_some(&event), 1000);
        assert!(option::destroy_some(event).value == 1, 1001);
        let event = consume(&mut subscriber);
        assert!(option::is_none(&event), 1002);
        emit(queue_name, TestEvent{value: 2});
        let event = consume(&mut subscriber);
        assert!(option::is_some(&event), 1003);
        assert!(option::destroy_some(event).value == 2, 1004);
        unsubscribe(subscriber);
    }

    #[test]
    fun test_event_queue_two_subscriber(){
        let queue_name = std::string::utf8(b"test_event_queue_two_subscriber");
        let subscriber1 = subscribe<TestEvent>(queue_name);
        emit(queue_name, TestEvent{value: 1});
        let subscriber2 = subscribe<TestEvent>(queue_name);

        let event1 = consume(&mut subscriber1);
        let event2 = consume(&mut subscriber2);
        
        assert!(option::is_some(&event1), 1000);
        assert!(option::destroy_some(event1).value == 1, 1001);
        assert!(option::is_none(&event2), 1002);

        emit(queue_name, TestEvent{value: 2});
        let event1 = consume(&mut subscriber1);
        let event2 = consume(&mut subscriber2);
        
        assert!(option::is_some(&event1), 1003);
        assert!(option::destroy_some(event1).value == 2, 1004);
        assert!(option::is_some(&event2), 1005);
        assert!(option::destroy_some(event2).value == 2, 1006);

        unsubscribe(subscriber1);
        unsubscribe(subscriber2);
    }

    #[test]
    fun test_event_queue_many_events(){
        let queue_name = std::string::utf8(b"test_event_queue_many_events");
        let subscriber = subscribe<TestEvent>(queue_name);
        let i = 0;
        let event_count = MAX_EVENT_HISTORY + 1;
        while(i < event_count){
            emit(queue_name, TestEvent{value: i});
            i = i + 1;
        };
        let event = consume(&mut subscriber);
        assert!(option::is_some(&event), 1000);
        //The value == 0 event was removed because the event history is full,
        //So the first event is 1
        assert!(option::destroy_some(event).value == 1, 1001);
        i = 2;
        while(i < event_count){
            let event = consume(&mut subscriber);
            assert!(option::is_some(&event), 1000 + i);
            assert!(option::destroy_some(event).value == i, 1001 + i);
            i = i + 1;
        };
        let event = consume(&mut subscriber);
        assert!(option::is_none(&event), 1002);
        unsubscribe(subscriber);
    }

    #[test]
    fun test_many_subscribers(){
        let queue_name = std::string::utf8(b"test_many_subscribers");
        let i = 0;
        let subscriber_count = MAX_SUBSCRIBER_COUNT;
        let subscribers: vector<Object<Subscriber<TestEvent>>> = vector::empty();
        while(i < subscriber_count){
            let subscriber = subscribe<TestEvent>(queue_name);
            vector::push_back(&mut subscribers, subscriber);
            i = i + 1;
        };
        emit(queue_name, TestEvent{value: 1});
        i = 0;
        while(i < subscriber_count){
            let subscriber = vector::borrow_mut(&mut subscribers, i);
            let event = consume(subscriber);
            assert!(option::is_some(&event), 1000 + i);
            assert!(option::destroy_some(event).value == 1, 1001 + i);
            i = i + 1;
        };
        emit(queue_name, TestEvent{value: 2});
        i = 0;
        while(i < subscriber_count){
            let subscriber = vector::remove(&mut subscribers, i);
            unsubscribe(subscriber);
            i = i + 1;
        };
        vector::destroy_empty(subscribers);
        let event_queue = event_queue<TestEvent>(queue_name);
        assert!(object::borrow(event_queue).sequence_number == 2, 1002);
        
        let subscribers:vector<ObjectID> = object::remove_field(event_queue, SUBSCRIBERS_KEY);
        assert!(vector::is_empty(&subscribers), 1003);

        //If all subscribers are removed, the event queue should be empty
        assert!(object::field_size(event_queue) == 0, 1004);
    }
}
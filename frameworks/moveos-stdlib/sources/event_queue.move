// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module moveos_std::event_queue {
    
    use std::vector;
    use std::option::{Self, Option};
    use std::string::String;
    use moveos_std::timestamp;
    use moveos_std::object::{Self, Object, ObjectID};

    const EVENT_EXPIRE_TIME: u64 = 1000 * 60 * 60 * 24 * 31; // 31 days
    const REMOVE_EXPIRED_EVENT_BATCH_SIZE: u64 = 100;
    const SUBSCRIBERS_KEY: vector<u8> = b"subscribers";

    const ErrorTooManySubscribers: u64 = 1;
    const ErrorSubscriberNotFound: u64 = 2;
    const ErrorEventNotFound: u64 = 3;
    const ErrorInvalidSequenceNumber: u64 = 4;

    struct EventQueue<phantom E> has key{
        head_sequence_number: u64,
        tail_sequence_number: u64,
    }

    struct OnChainEvent<E> has store, drop {
        event: E,
        emit_time: u64,
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
                head_sequence_number: 0,
                tail_sequence_number: 0,
            });
            let subscribers : vector<ObjectID> = vector::empty();
            object::add_field(&mut event_queue_obj, SUBSCRIBERS_KEY, subscribers);
            //We transfer the display object to the moveos_std
            //And the caller do not need to care about the event queue object
            object::transfer_extend(event_queue_obj, @moveos_std);
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
        let head_sequence_number = object::borrow(event_queue_obj).head_sequence_number;
        let now = timestamp::now_milliseconds();
        let on_chain_event = OnChainEvent {
            event: event,
            emit_time: now,
        };
        object::add_field(event_queue_obj, head_sequence_number, on_chain_event);
        object::borrow_mut(event_queue_obj).head_sequence_number = head_sequence_number + 1;
        remove_expired_events(event_queue_obj);    
    }

    public fun consume<E: copy + drop + store>(subscriber_obj: &mut Object<Subscriber<E>>) : Option<E> {
        let subscriber = object::borrow_mut(subscriber_obj);
        let subscriber_sequence_number = subscriber.sequence_number;
        let event_queue_obj = event_queue<E>(subscriber.queue_name);
        let head_sequence_number = object::borrow(event_queue_obj).head_sequence_number;
        let tail_sequence_number = object::borrow(event_queue_obj).tail_sequence_number;
        
        if (subscriber_sequence_number >= head_sequence_number) {
            return option::none()
        };
        //If the subscriber sequence number is less than the tail sequence number,
        //It means the subscriber is too slow to consume the event
        //We should update the subscriber sequence number to the tail sequence number
        subscriber_sequence_number = if (tail_sequence_number > subscriber_sequence_number) {
            tail_sequence_number
        } else {
            subscriber_sequence_number
        };
        assert!(object::contains_field(event_queue_obj, subscriber_sequence_number), ErrorEventNotFound);
        let on_chain_event: &mut OnChainEvent<E> = object::borrow_mut_field(event_queue_obj, subscriber_sequence_number);
        subscriber.sequence_number = subscriber_sequence_number + 1;
        let event = option::some(on_chain_event.event);
        remove_expired_events(event_queue_obj);
        event
    }

    public fun subscribe<E: copy + drop + store>(queue_name: String) : Object<Subscriber<E>> {
        let event_queue_obj = event_queue<E>(queue_name);
        let head_sequence_number = object::borrow(event_queue_obj).head_sequence_number;
        let subscribers = borrow_mut_subscribers(event_queue_obj);
        let subscriber = object::new(Subscriber {
            queue_name,
            sequence_number: head_sequence_number,
        });
        vector::push_back(subscribers, object::id(&subscriber));
        subscriber
    }

    public fun subscribe_with_sequence_number<E: copy + drop + store>(queue_name: String, sequence_number: u64) : Object<Subscriber<E>> {
        let event_queue_obj = event_queue<E>(queue_name);
        let head_sequence_number = object::borrow(event_queue_obj).head_sequence_number;
        let tail_sequence_number = object::borrow(event_queue_obj).tail_sequence_number;
        assert!(sequence_number >= tail_sequence_number && sequence_number <= head_sequence_number, ErrorInvalidSequenceNumber);
        let subscribers = borrow_mut_subscribers(event_queue_obj);
        let subscriber = object::new(Subscriber {
            queue_name,
            sequence_number,
        });
        vector::push_back(subscribers, object::id(&subscriber));
        subscriber
    }

    public fun unsubscribe<E: copy + drop + store>(subscriber: Object<Subscriber<E>>) {
        let subscriber_id = object::id(&subscriber);
        let Subscriber{sequence_number:_, queue_name} = object::remove(subscriber);
        let event_queue_obj = event_queue<E>(queue_name);
        let subscribers = borrow_mut_subscribers(event_queue_obj);
        
        let (find,index) = vector::index_of(subscribers, &subscriber_id);
        assert!(find, ErrorSubscriberNotFound);
        vector::remove(subscribers, index);
    }

    /// Remove the expired events from the event queue
    /// Anyone can call this function to remove the expired events
    public fun remove_expired_events<E: copy + drop + store>(event_queue_obj: &mut Object<EventQueue<E>>){
        let head_sequence_number = object::borrow(event_queue_obj).head_sequence_number;
        let tail_sequence_number = object::borrow(event_queue_obj).tail_sequence_number;
        let now = timestamp::now_milliseconds();
        let remove_sequence_number = tail_sequence_number;
        while (remove_sequence_number < head_sequence_number){
            if (object::contains_field(event_queue_obj, remove_sequence_number)){
                let on_chain_event: &OnChainEvent<E> = object::borrow_field(event_queue_obj, remove_sequence_number);
                if (now - on_chain_event.emit_time > EVENT_EXPIRE_TIME){
                    let _event: OnChainEvent<E> = object::remove_field(event_queue_obj, remove_sequence_number);
                }else{
                    break
                }
            };
            remove_sequence_number = remove_sequence_number + 1;
            if (remove_sequence_number - tail_sequence_number >= REMOVE_EXPIRED_EVENT_BATCH_SIZE){
                break
            }
        };
        object::borrow_mut(event_queue_obj).tail_sequence_number = remove_sequence_number;
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
    fun test_event_queue_expired_events(){
        let queue_name = std::string::utf8(b"test_event_queue_expired_events");
        let subscriber = subscribe<TestEvent>(queue_name);
        emit(queue_name, TestEvent{value: 1});
        moveos_std::timestamp::fast_forward_milliseconds_for_test(EVENT_EXPIRE_TIME + 1);
        emit(queue_name, TestEvent{value: 2});
        let event = consume(&mut subscriber);
        assert!(option::is_some(&event), 1000);
        //The first event should be expired
        assert!(option::destroy_some(event).value == 2, 1001);
        unsubscribe(subscriber);
    }

}
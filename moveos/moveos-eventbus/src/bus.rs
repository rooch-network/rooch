use crossbeam_channel::{unbounded, Receiver, Sender};
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

type SenderType = Arc<RwLock<HashMap<TypeId, HashMap<String, Sender<Box<dyn Any + Send>>>>>>;
type ReceiverType = Arc<RwLock<HashMap<TypeId, HashMap<String, Receiver<Box<dyn Any + Send>>>>>>;
type CallBackType = Arc<
    RwLock<
        HashMap<TypeId, HashMap<String, Box<dyn Fn(Box<dyn Any + Send>) + Send + Sync + 'static>>>,
    >,
>;

/// The EventBus struct manages event subscription and notification.
#[derive(Clone)]
pub struct EventBus {
    senders: SenderType,
    receivers: ReceiverType,
    callbacks: CallBackType,
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

impl EventBus {
    /// Creates a new instance of EventBus.
    pub fn new() -> Self {
        EventBus {
            senders: Arc::new(RwLock::new(HashMap::new())),
            receivers: Arc::new(RwLock::new(HashMap::new())),
            callbacks: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Publishes an event, notifying all subscribers with the data of type `T`.
    pub fn notify<T: 'static + Send + Clone>(&self, event_data: T) {
        let event_type_id = TypeId::of::<T>();
        {
            let senders = self.senders.read().unwrap();
            if let Some(event_senders) = senders.get(&event_type_id) {
                for (subscriber, sender) in event_senders {
                    if sender.send(Box::new(event_data.clone())).is_err() {
                        log::error!(
                            "Failed to send event '{:?}' to subscriber '{}'",
                            event_type_id,
                            subscriber
                        );
                    }
                }
            }
        }

        let callbacks = self.callbacks.read().unwrap();
        if let Some(event_callbacks) = callbacks.get(&event_type_id) {
            for (subscriber, callback) in event_callbacks {
                log::debug!(
                    "Executing callback for subscriber: '{}' on event: '{:?}'",
                    subscriber,
                    event_type_id
                );
                callback(Box::new(event_data.clone()));
            }
        } else {
            log::debug!("No subscribers found for event: '{:?}'", event_type_id);
        }
    }

    /// Non-blocking check if the specified subscriber has received the event and returns the event data.
    pub fn get_event<T: 'static + Send>(&self, subscriber: &str) -> Option<T> {
        let event_type_id = TypeId::of::<T>();
        let receivers = self.receivers.read().unwrap();
        if let Some(event_receivers) = receivers.get(&event_type_id) {
            if let Some(receiver) = event_receivers.get(subscriber) {
                if let Ok(boxed_event) = receiver.try_recv() {
                    if let Ok(event_data) = boxed_event.downcast::<T>() {
                        return Some(*event_data);
                    } else {
                        log::error!(
                            "Failed to downcast event data for subscriber: '{}'",
                            subscriber
                        );
                    }
                }
            } else {
                log::debug!(
                    "Subscriber: '{}' is not registered for event: '{:?}'",
                    subscriber,
                    event_type_id
                );
            }
        } else {
            log::debug!("No event '{:?}' found in the system", event_type_id);
        }
        None
    }

    /// Registers an event and subscriber.
    pub fn register_event_subscriber<T: 'static + Send>(&self, subscriber: &str) {
        let event_type_id = TypeId::of::<T>();
        let (sender, receiver) = unbounded();

        {
            let mut senders = self.senders.write().unwrap();
            let event_senders = senders.entry(event_type_id).or_default();
            event_senders.insert(subscriber.to_string(), sender);
        }

        {
            let mut receivers = self.receivers.write().unwrap();
            let event_receivers = receivers.entry(event_type_id).or_default();
            event_receivers.insert(subscriber.to_string(), receiver);
        }

        log::debug!(
            "Registered subscriber: '{}' for event: '{:?}'",
            subscriber,
            event_type_id
        );
    }

    /// Subscribes to an event with a callback function.
    pub fn subscribe<T: 'static + Send, F>(&self, subscriber: &str, callback: F)
    where
        F: Fn(Box<dyn Any + Send>) + Send + Sync + 'static,
    {
        let event_type_id = TypeId::of::<T>();
        let mut callbacks = self.callbacks.write().unwrap();
        let event_callbacks = callbacks.entry(event_type_id).or_default();
        event_callbacks.insert(subscriber.to_string(), Box::new(callback));

        log::debug!(
            "Subscriber '{}' registered callback for event '{:?}'",
            subscriber,
            event_type_id
        );
    }

    /// Prints the current status of the event bus.
    pub fn print_status(&self) {
        let senders = self.senders.read().unwrap();
        for (event_type_id, subscribers) in senders.iter() {
            log::debug!(
                "Event: '{:?}', Subscribers: {}",
                event_type_id,
                subscribers.len()
            );
            for subscriber in subscribers.keys() {
                log::debug!("  - Subscriber: '{}'", subscriber);
            }
        }
    }

    /// Removes a specific subscriber's registration.
    pub fn remove_subscriber<T: 'static + Send>(&self, subscriber: &str) {
        let event_type_id = TypeId::of::<T>();
        {
            let mut senders = self.senders.write().unwrap();
            if let Some(event_senders) = senders.get_mut(&event_type_id) {
                event_senders.remove(subscriber);
                log::debug!(
                    "Removed sender for subscriber: '{}' from event: '{:?}'",
                    subscriber,
                    event_type_id
                );
            }
        }

        {
            let mut receivers = self.receivers.write().unwrap();
            if let Some(event_receivers) = receivers.get_mut(&event_type_id) {
                event_receivers.remove(subscriber);
                log::debug!(
                    "Removed receiver for subscriber: '{}' from event: '{:?}'",
                    subscriber,
                    event_type_id
                );
            }
        }

        {
            let mut callbacks = self.callbacks.write().unwrap();
            if let Some(event_callbacks) = callbacks.get_mut(&event_type_id) {
                event_callbacks.remove(subscriber);
                log::debug!(
                    "Removed callback for subscriber: '{}' from event: '{:?}'",
                    subscriber,
                    event_type_id
                );
            }
        }
    }

    /// Clears all events and subscribers.
    pub fn clear(&self) {
        let mut senders = self.senders.write().unwrap();
        let mut receivers = self.receivers.write().unwrap();
        let mut callbacks = self.callbacks.write().unwrap();
        senders.clear();
        receivers.clear();
        callbacks.clear();
        log::debug!("Cleared all events and subscribers.");
    }

    /// Gets the number of subscribers for a specific event.
    pub fn subscriber_count<T: 'static + Send>(&self) -> usize {
        let event_type_id = TypeId::of::<T>();
        let senders = self.senders.read().unwrap();
        if let Some(event_senders) = senders.get(&event_type_id) {
            return event_senders.len();
        }
        0
    }

    /// Gets the total number of events.
    pub fn event_count(&self) -> usize {
        let senders = self.senders.read().unwrap();
        senders.len()
    }

    /// Checks if there are any subscribers for a specific event.
    pub fn has_subscribers<T: 'static + Send>(&self) -> bool {
        let event_type_id = TypeId::of::<T>();
        let senders = self.senders.read().unwrap();
        if let Some(event_senders) = senders.get(&event_type_id) {
            return !event_senders.is_empty();
        }
        false
    }

    /// Checks if a specific event exists.
    pub fn has_event_type<T: 'static + Send>(&self) -> bool {
        let event_type_id = TypeId::of::<T>();
        let senders = self.senders.read().unwrap();
        senders.contains_key(&event_type_id)
    }

    /// Gets all subscribers for a specific event.
    pub fn get_subscribers<T: 'static + Send>(&self) -> Vec<String> {
        let event_type_id = TypeId::of::<T>();
        let senders = self.senders.read().unwrap();
        if let Some(event_senders) = senders.get(&event_type_id) {
            return event_senders.keys().cloned().collect();
        }
        Vec::new()
    }
}

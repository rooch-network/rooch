// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::{format_err, Error};
use coerce::actor::message::{Handler, Message};
use coerce::actor::{Actor, ActorRefErr, LocalActorRef};
use crossbeam_channel::{unbounded, Receiver, Sender};
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(thiserror::Error, Debug)]
pub enum EventBusError {
    #[error("Locker read error {0:?}.")]
    LockerReadError(Error),
    #[error("Locker write error {0:?}.")]
    LockerWriteError(Error),
    #[error("ActorRef error {0:?}.")]
    ActorRefError(Error),
}

pub struct EventData {
    pub data: Box<dyn Any + Send + Sync + 'static>,
}

impl EventData {
    pub fn new(data: Box<dyn Any + Send + Sync + 'static>) -> EventData {
        Self { data }
    }
}

impl Message for EventData {
    type Result = anyhow::Result<()>;
}

pub trait EventNotifier {
    fn send_event_data(&self, data: EventData) -> Result<(), ActorRefErr>;
}

impl<A: Actor + Handler<EventData>> EventNotifier for LocalActorRef<A> {
    fn send_event_data(&self, data: EventData) -> Result<(), ActorRefErr> {
        self.notify(data)
    }
}

type SenderType = Arc<RwLock<HashMap<TypeId, HashMap<String, Sender<Box<dyn Any + Send>>>>>>;
type ReceiverType = Arc<RwLock<HashMap<TypeId, HashMap<String, Receiver<Box<dyn Any + Send>>>>>>;
type CallBackType = Arc<
    RwLock<
        HashMap<TypeId, HashMap<String, Box<dyn Fn(Box<dyn Any + Send>) + Send + Sync + 'static>>>,
    >,
>;
type ActorsType =
    Arc<RwLock<HashMap<TypeId, HashMap<String, Box<dyn EventNotifier + Send + Sync + 'static>>>>>;

/// The EventBus struct manages event subscription and notification.
#[derive(Clone)]
pub struct EventBus {
    senders: SenderType,
    receivers: ReceiverType,
    callbacks: CallBackType,
    actors: ActorsType,
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
            actors: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Publishes an event, notifying all subscribers with the data of type `T`.
    pub fn notify<T: 'static + Send + Sync + Clone>(&self, event_data: T) -> anyhow::Result<()> {
        let event_type_id = TypeId::of::<T>();
        {
            let senders = match self.senders.read() {
                Ok(v) => v,
                Err(_) => {
                    return Err(Error::from(EventBusError::LockerReadError(format_err!(
                        "read the locker with poisoned error"
                    ))))
                }
            };
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

        let callbacks = match self.callbacks.read() {
            Ok(v) => v,
            Err(_) => {
                return Err(Error::from(EventBusError::LockerReadError(format_err!(
                    "read the locker with poisoned error"
                ))))
            }
        };
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

        let actors = match self.actors.read() {
            Ok(v) => v,
            Err(_) => {
                return Err(Error::from(EventBusError::LockerReadError(format_err!(
                    "read the locker with poisoned error"
                ))))
            }
        };
        if let Some(actors_map) = actors.get(&event_type_id) {
            for (subscriber, actor) in actors_map {
                log::debug!(
                    "Executing actor for subscriber: '{}' on event: '{:?}'",
                    subscriber,
                    event_type_id
                );
                match actor.send_event_data(EventData::new(Box::new(event_data.clone()))) {
                    Ok(_) => {}
                    Err(_) => {
                        return Err(Error::from(EventBusError::LockerReadError(format_err!(
                            "event actor send message failed"
                        ))))
                    }
                }
            }
        }

        Ok(())
    }

    /// Non-blocking check if the specified subscriber has received the event and returns the event data.
    pub fn get_event<T: 'static + Send>(&self, subscriber: &str) -> anyhow::Result<Option<T>> {
        let event_type_id = TypeId::of::<T>();
        let receivers = match self.receivers.read() {
            Ok(v) => v,
            Err(_) => {
                return Err(Error::from(EventBusError::LockerReadError(format_err!(
                    "read the locker with poisoned error"
                ))))
            }
        };
        if let Some(event_receivers) = receivers.get(&event_type_id) {
            if let Some(receiver) = event_receivers.get(subscriber) {
                if let Ok(boxed_event) = receiver.try_recv() {
                    if let Ok(event_data) = boxed_event.downcast::<T>() {
                        return Ok(Some(*event_data));
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
        Ok(None)
    }

    /// Registers an event and subscriber.
    pub fn register_event_subscriber<T: 'static + Send>(
        &self,
        subscriber: &str,
    ) -> anyhow::Result<()> {
        let event_type_id = TypeId::of::<T>();
        let (sender, receiver) = unbounded();

        {
            let mut senders = match self.senders.write() {
                Ok(v) => v,
                Err(_) => {
                    return Err(Error::from(EventBusError::LockerWriteError(format_err!(
                        "write the locker with poisoned error"
                    ))))
                }
            };
            let event_senders = senders.entry(event_type_id).or_default();
            event_senders.insert(subscriber.to_string(), sender);
        }

        {
            let mut receivers = match self.receivers.write() {
                Ok(v) => v,
                Err(_) => {
                    return Err(Error::from(EventBusError::LockerWriteError(format_err!(
                        "write the locker with poisoned error"
                    ))))
                }
            };
            let event_receivers = receivers.entry(event_type_id).or_default();
            event_receivers.insert(subscriber.to_string(), receiver);
        }

        log::debug!(
            "Registered subscriber: '{}' for event: '{:?}'",
            subscriber,
            event_type_id
        );
        Ok(())
    }

    /// Subscribes to an event with a callback function.
    pub fn callback_subscribe<T: 'static + Send, F>(
        &self,
        subscriber: &str,
        callback: F,
    ) -> anyhow::Result<()>
    where
        F: Fn(Box<dyn Any + Send>) + Send + Sync + 'static,
    {
        let event_type_id = TypeId::of::<T>();
        let mut callbacks = match self.callbacks.write() {
            Ok(v) => v,
            Err(_) => {
                return Err(Error::from(EventBusError::LockerWriteError(format_err!(
                    "write the locker with poisoned error"
                ))))
            }
        };
        let event_callbacks = callbacks.entry(event_type_id).or_default();
        event_callbacks.insert(subscriber.to_string(), Box::new(callback));

        log::debug!(
            "Subscriber '{}' registered callback for event '{:?}'",
            subscriber,
            event_type_id
        );
        Ok(())
    }

    pub fn actor_subscribe<T: Send + 'static>(
        &self,
        subscriber: &str,
        actor: Box<dyn EventNotifier + Send + Sync + 'static>,
    ) -> anyhow::Result<()> {
        let event_type_id = TypeId::of::<T>();

        let mut actors = match self.actors.write() {
            Ok(v) => v,
            Err(_) => {
                return Err(Error::from(EventBusError::LockerWriteError(format_err!(
                    "write the locker with poisoned error"
                ))))
            }
        };

        let event_actors = actors.entry(event_type_id).or_default();
        event_actors.insert(subscriber.to_string(), actor);
        Ok(())
    }

    /// Prints the current status of the event bus.
    pub fn print_status(&self) -> anyhow::Result<()> {
        let senders = match self.senders.read() {
            Ok(v) => v,
            Err(_) => {
                return Err(Error::from(EventBusError::LockerReadError(format_err!(
                    "read the locker with poisoned error"
                ))))
            }
        };

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
        Ok(())
    }

    /// Removes a specific subscriber's registration.
    pub fn remove_subscriber<T: 'static + Send>(&self, subscriber: &str) -> anyhow::Result<()> {
        let event_type_id = TypeId::of::<T>();
        {
            let mut senders = match self.senders.write() {
                Ok(v) => v,
                Err(_) => {
                    return Err(Error::from(EventBusError::LockerWriteError(format_err!(
                        "write the locker with poisoned error"
                    ))))
                }
            };

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
            let mut receivers = match self.receivers.write() {
                Ok(v) => v,
                Err(_) => {
                    return Err(Error::from(EventBusError::LockerWriteError(format_err!(
                        "write the locker with poisoned error"
                    ))))
                }
            };

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
            let mut callbacks = match self.callbacks.write() {
                Ok(v) => v,
                Err(_) => {
                    return Err(Error::from(EventBusError::LockerWriteError(format_err!(
                        "write the locker with poisoned error"
                    ))))
                }
            };

            if let Some(event_callbacks) = callbacks.get_mut(&event_type_id) {
                event_callbacks.remove(subscriber);
                log::debug!(
                    "Removed callback for subscriber: '{}' from event: '{:?}'",
                    subscriber,
                    event_type_id
                );
            }
        }

        Ok(())
    }

    /// Clears all events and subscribers.
    pub fn clear(&self) -> anyhow::Result<()> {
        let mut senders = match self.senders.write() {
            Ok(v) => v,
            Err(_) => {
                return Err(Error::from(EventBusError::LockerWriteError(format_err!(
                    "write the locker with poisoned error"
                ))))
            }
        };

        let mut receivers = match self.receivers.write() {
            Ok(v) => v,
            Err(_) => {
                return Err(Error::from(EventBusError::LockerWriteError(format_err!(
                    "write the locker with poisoned error"
                ))))
            }
        };

        let mut callbacks = match self.callbacks.write() {
            Ok(v) => v,
            Err(_) => {
                return Err(Error::from(EventBusError::LockerWriteError(format_err!(
                    "write the locker with poisoned error"
                ))))
            }
        };

        senders.clear();
        receivers.clear();
        callbacks.clear();
        log::debug!("Cleared all events and subscribers.");

        Ok(())
    }

    /// Gets the number of subscribers for a specific event.
    pub fn subscriber_count<T: 'static + Send>(&self) -> anyhow::Result<usize> {
        let event_type_id = TypeId::of::<T>();
        let senders = match self.senders.read() {
            Ok(v) => v,
            Err(_) => {
                return Err(Error::from(EventBusError::LockerReadError(format_err!(
                    "read the locker with poisoned error"
                ))))
            }
        };

        if let Some(event_senders) = senders.get(&event_type_id) {
            return Ok(event_senders.len());
        }

        Ok(0)
    }

    /// Gets the total number of events.
    pub fn event_count(&self) -> anyhow::Result<usize> {
        let senders = match self.senders.read() {
            Ok(v) => v,
            Err(_) => {
                return Err(Error::from(EventBusError::LockerReadError(format_err!(
                    "read the locker with poisoned error"
                ))))
            }
        };

        Ok(senders.len())
    }

    /// Checks if there are any subscribers for a specific event.
    pub fn has_subscribers<T: 'static + Send>(&self) -> anyhow::Result<bool> {
        let event_type_id = TypeId::of::<T>();
        let senders = match self.senders.read() {
            Ok(v) => v,
            Err(_) => {
                return Err(Error::from(EventBusError::LockerReadError(format_err!(
                    "read the locker with poisoned error"
                ))))
            }
        };

        if let Some(event_senders) = senders.get(&event_type_id) {
            return Ok(!event_senders.is_empty());
        }
        Ok(false)
    }

    /// Checks if a specific event exists.
    pub fn has_event_type<T: 'static + Send>(&self) -> anyhow::Result<bool> {
        let event_type_id = TypeId::of::<T>();
        let senders = match self.senders.read() {
            Ok(v) => v,
            Err(_) => {
                return Err(Error::from(EventBusError::LockerReadError(format_err!(
                    "read the locker with poisoned error"
                ))))
            }
        };

        Ok(senders.contains_key(&event_type_id))
    }

    /// Gets all subscribers for a specific event.
    pub fn get_subscribers<T: 'static + Send>(&self) -> anyhow::Result<Vec<String>> {
        let event_type_id = TypeId::of::<T>();

        let senders = match self.senders.read() {
            Ok(v) => v,
            Err(_) => {
                return Err(Error::from(EventBusError::LockerReadError(format_err!(
                    "read the locker with poisoned error"
                ))))
            }
        };

        if let Some(event_senders) = senders.get(&event_type_id) {
            return Ok(event_senders.keys().cloned().collect());
        }
        Ok(Vec::new())
    }
}

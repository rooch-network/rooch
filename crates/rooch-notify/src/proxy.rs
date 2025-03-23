// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::actor::NotifyActor;
use coerce::actor::ActorRef;

#[derive(Clone)]
pub struct NotifyProxy {
    pub actor: ActorRef<NotifyActor>,
}

impl NotifyProxy {
    pub fn new(actor: ActorRef<NotifyActor>) -> Self {
        Self { actor }
    }
}

impl From<ActorRef<NotifyActor>> for NotifyProxy {
    fn from(actor: ActorRef<NotifyActor>) -> Self {
        Self::new(actor)
    }
}

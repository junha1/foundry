// Copyright 2020 Kodebox, Inc.
// This file is part of CodeChain.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use crate::{EventTags, Events, Params, Sink};
use futures::Future;
use jsonrpc_core::futures;

enum ConnectionState {
    Connected,
}

pub struct Connection {
    status: ConnectionState,
    pub subscription_id: u64,
    pub interested_events: Vec<EventTags>,
    sink: Sink,
}

impl Connection {
    pub fn new(sink: Sink, sub_id: u64) -> Self {
        Self {
            status: ConnectionState::Connected,
            subscription_id: sub_id,
            interested_events: Vec::new(),
            sink,
        }
    }
    pub fn add_events(&mut self, params: Vec<String>) {
        match params[0].as_str() {
            "PeerAdded" => {
                let event = EventTags::PeerAdded;
                cinfo!(INFORMER, "The event is successfully added to user's interested events");
                self.interested_events.push(event);
            }
            _ => {
                cinfo!(INFORMER, "invalid Event: the event is not supported");
            }
        }
    }

    pub fn notify_client(&self, event: &Events) {
        let json_object = serde_json::to_value(event).expect("json format is not valid").as_object_mut().cloned();
        let params = Params::Map(json_object.expect("Event is serialized as object"));
        match self.status {
            ConnectionState::Connected => match self.sink.notify(params).wait() {
                Ok(_) => {}
                Err(_) => {
                    cinfo!(INFORMER, "Subscription has ended, finishing.");
                }
            },
        }
    }
}
// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, clap::ValueEnum, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum ServiceType {
    #[default]
    Both,
    Http,
    WebSocket,
}

impl ServiceType {
    pub fn is_both(&self) -> bool {
        matches!(self, ServiceType::Both)
    }

    pub fn is_http(&self) -> bool {
        matches!(self, ServiceType::Http)
    }

    pub fn is_web_socket(&self) -> bool {
        matches!(self, ServiceType::WebSocket)
    }
}

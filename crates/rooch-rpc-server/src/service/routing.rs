// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use rooch_open_rpc::MethodRouting;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct RpcRouter {
    routes: HashMap<String, MethodRouting>,
    route_to_methods: HashSet<String>,
    disable_routing: bool,
}

impl RpcRouter {
    pub fn new(routes: HashMap<String, MethodRouting>, disable_routing: bool) -> Self {
        let route_to_methods = routes.values().map(|v| v.route_to.clone()).collect();

        Self {
            routes,
            route_to_methods,
            disable_routing,
        }
    }

    pub fn route<'c, 'a: 'c, 'b: 'c>(&'a self, method: &'b str, version: Option<&str>) -> &'c str {
        // Reject direct access to the old methods
        if self.route_to_methods.contains(method) {
            "INVALID_ROUTING"
        } else if self.disable_routing {
            method
        } else {
            // Modify the method name if routing is enabled
            match (version, self.routes.get(method)) {
                (Some(v), Some(route)) if route.matches(v) => route.route_to.as_str(),
                _ => method,
            }
        }
    }
}

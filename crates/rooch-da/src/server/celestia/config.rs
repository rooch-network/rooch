// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

pub struct ServerConfig {
    pub rpc_version: u8,

}

// TODO hotfix
pub struct CelestiaRPCConfig {
    pub namespace: String,
    pub conn_str: String,
    pub auth_token: String,
}
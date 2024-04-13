// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;

use testcontainers::{
    core::{ContainerState, ExecCommand, WaitFor},
    Image, ImageArgs,
};

const NAME: &str = "bitseed/ord";
const TAG: &str = "0.17.0";

#[derive(Debug, Default, Clone)]
pub struct OrdImageArgs {
    pub bitcoin_rpc_url: String,
    pub bitcoin_rpc_user: String,
    pub bitcoin_rpc_pass: String,
}

impl ImageArgs for OrdImageArgs {
    fn into_iterator(self) -> Box<dyn Iterator<Item = String>> {
        Box::new(
            vec![
                "--regtest".to_string(),
                format!("--rpc-url={}", self.bitcoin_rpc_url),
                format!("--bitcoin-rpc-user={}", self.bitcoin_rpc_user),
                format!("--bitcoin-rpc-pass={}", self.bitcoin_rpc_pass),
                "server".to_string(),
            ]
            .into_iter(),
        )
    }
}

#[derive(Debug, Default, Clone)]
pub struct Ord {
    env_vars: HashMap<String, String>,
}

impl Image for Ord {
    type Args = OrdImageArgs;

    fn name(&self) -> String {
        NAME.to_owned()
    }

    fn tag(&self) -> String {
        TAG.to_owned()
    }

    fn ready_conditions(&self) -> Vec<WaitFor> {
        vec![WaitFor::message_on_stderr("Listening on")]
    }

    fn expose_ports(&self) -> Vec<u16> {
        vec![80]
    }

    fn env_vars(&self) -> Box<dyn Iterator<Item = (&String, &String)> + '_> {
        Box::new(self.env_vars.iter())
    }

    fn exec_after_start(&self, _cs: ContainerState) -> Vec<ExecCommand> {
        vec![ExecCommand {
            cmd: "/bin/rm -rf /data/.bitcoin/regtest/wallets/ord".to_owned(),
            ready_conditions: vec![WaitFor::Nothing],
        }]
    }
}

impl Ord {
    pub fn new(
        bitcoin_rpc_url: String,
        bitcoin_rpc_user: String,
        bitcoin_rpc_pass: String,
    ) -> (Self, OrdImageArgs) {
        (
            Ord {
                env_vars: HashMap::new(),
            },
            OrdImageArgs {
                bitcoin_rpc_url,
                bitcoin_rpc_user,
                bitcoin_rpc_pass,
            },
        )
    }
}

// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;

use testcontainers::{core::WaitFor, Image, ImageArgs};

const NAME: &str = "bitseed/bitseed";
const TAG: &str = "0.1.4";

#[derive(Debug, Default, Clone)]
pub struct BitseedImageArgs {
    pub bitcoin_rpc_url: String,
    pub bitcoin_rpc_user: String,
    pub bitcoin_rpc_pass: String,
    pub ord_server_url: String,
    pub ext_args: Vec<String>,
}

impl ImageArgs for BitseedImageArgs {
    fn into_iterator(self) -> Box<dyn Iterator<Item = String>> {
        let mut args = vec![
            "--regtest".to_string(),
            format!("--rpc-url={}", self.bitcoin_rpc_url),
            format!("--bitcoin-rpc-user={}", self.bitcoin_rpc_user),
            format!("--bitcoin-rpc-pass={}", self.bitcoin_rpc_pass),
            format!("--server-url={}", self.ord_server_url),
        ];

        args.extend(self.ext_args);

        Box::new(args.into_iter())
    }
}

#[derive(Debug, Default, Clone)]
pub struct Bitseed {
    env_vars: HashMap<String, String>,
}

impl Image for Bitseed {
    type Args = BitseedImageArgs;

    fn name(&self) -> String {
        NAME.to_owned()
    }

    fn tag(&self) -> String {
        TAG.to_owned()
    }

    fn ready_conditions(&self) -> Vec<WaitFor> {
        vec![WaitFor::message_on_stderr("Bitseed CLI started")]
    }

    fn env_vars(&self) -> Box<dyn Iterator<Item = (&String, &String)> + '_> {
        Box::new(self.env_vars.iter())
    }
}

impl Bitseed {
    pub fn new(
        bitcoin_rpc_url: String,
        bitcoin_rpc_user: String,
        bitcoin_rpc_pass: String,
        ord_server_url: String,
        ext_args: Vec<String>,
    ) -> (Self, BitseedImageArgs) {
        (
            Bitseed {
                env_vars: HashMap::new(),
            },
            BitseedImageArgs {
                bitcoin_rpc_url,
                bitcoin_rpc_user,
                bitcoin_rpc_pass,
                ord_server_url,
                ext_args,
            },
        )
    }
}

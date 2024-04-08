// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use hmac::{Hmac, Mac};
use rand::Rng;
use sha2::Sha256;
use std::collections::HashMap;

use testcontainers::{core::WaitFor, Image, ImageArgs};

const NAME: &str = "lncm/bitcoind";
const TAG: &str = "v25.1";

#[derive(Debug, Default, Clone)]
pub struct BitcoindImageArgs {
    pub rpc_bind: String,
    pub rpc_user: String,
    pub rpc_pass: String,
}

impl BitcoindImageArgs {
    // Function to generate salted hash for the rpcauth parameter
    fn generate_rpcauth(&self) -> String {
        let salt: [u8; 16] = rand::thread_rng().gen();
        let salt_hex = hex::encode(salt);
    
        let mut mac = Hmac::<Sha256>::new_from_slice(salt_hex.as_bytes()).unwrap();
        mac.update(self.rpc_pass.as_bytes());
        let result = mac.finalize();
        let password_hmac = hex::encode(result.into_bytes());
    
        format!("{}:{}${}", self.rpc_user, salt_hex, password_hmac)
    }
}

impl ImageArgs for BitcoindImageArgs {
    fn into_iterator(self) -> Box<dyn Iterator<Item = String>> {
        let rpcauth = self.generate_rpcauth();

        Box::new(
          vec![
            "-chain=regtest".to_string(),
            "-txindex=1".to_string(),
            "-fallbackfee=0.00001".to_string(),
            "-zmqpubrawblock=tcp://0.0.0.0:28332".to_string(),
            "-zmqpubrawtx=tcp://0.0.0.0:28333".to_string(),
            "-rpcallowip=0.0.0.0/0".to_string(),
            format!("-rpcbind={}", self.rpc_bind),
            format!("-rpcauth={}", rpcauth),
          ].into_iter(),
        )
    }
}

#[derive(Debug, Clone)]
pub struct BitcoinD {
    env_vars: HashMap<String, String>,
}

impl BitcoinD {
    pub fn new(
        rpc_bind: String,
        rpc_user: String,
        rpc_pass: String,
    ) -> (Self, BitcoindImageArgs) {
        (
            BitcoinD {
                env_vars: HashMap::new(),
            },
            BitcoindImageArgs{
                rpc_bind,
                rpc_user,
                rpc_pass,
            }
        )
    }
}

impl Image for BitcoinD {
    type Args = BitcoindImageArgs;

    fn name(&self) -> String {
        NAME.to_owned()
    }

    fn tag(&self) -> String {
        TAG.to_owned()
    }

    fn ready_conditions(&self) -> Vec<WaitFor> {
        vec![WaitFor::message_on_stdout("txindex thread start")]
    }

    fn expose_ports(&self) -> Vec<u16> {
        vec![18443, 18444, 28333, 28332]
    }

    fn env_vars(&self) -> Box<dyn Iterator<Item = (&String, &String)> + '_> {
        Box::new(self.env_vars.iter())
    }
}

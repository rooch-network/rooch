// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

#![allow(unused_imports)]
use anyhow::Result;
use clap::Parser;
use move_core_types::{
    account_address::AccountAddress,
    ident_str,
    identifier::Identifier,
    language_storage::{ModuleId, TypeTag},
    parser::parse_type_tag,
};
use moveos_client::Client;
use moveos_types::transaction::{MoveTransaction, SimpleTransaction};

/// Create a new account on-chain
///
/// An account can be created by transferring coins, or by making an explicit
/// call to create an account.  This will create an account with no coins, and
/// any coins will have to transferred afterwards.
#[derive(Debug, Parser)]
pub struct CreateAccount {
    /// Address of the new account
    #[clap(long)]
    pub address: AccountAddress,

    /// RPC client options.
    #[clap(flatten)]
    client: Client,
}

impl CreateAccount {
    pub async fn execute(self) -> Result<()> {
        let txn = MoveTransaction::new_function(
            ModuleId::new(
                AccountAddress::new([
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 1,
                ]),
                ident_str!("account").to_owned(),
            ),
            ident_str!("create_account_entry").to_owned(),
            vec![],
            vec![bcs::to_bytes(&self.address).unwrap()],
        );
        let txn = SimpleTransaction::new(self.address, txn);

        self.client.submit_txn(txn).await?;
        // let resp = self.client.submit_txn(txn).await?;
        // println!("{:?}", resp);
        Ok(())
    }
}

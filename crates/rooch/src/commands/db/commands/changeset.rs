// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::WalletContextOptions;
use clap::Parser;
use rooch_rpc_api::jsonrpc_types::{StateChangeSetPageView, StateChangeSetWithTxOrderView};
use rooch_rpc_client::Client;
use rooch_types::error::RoochResult;

#[derive(Debug, Parser)]
pub struct ChangesetCommand {
    #[clap(long = "tx-order", help = "Start with the tx order")]
    pub tx_order: Option<u64>,

    #[clap(
        long = "max-limit",
        help = "Max limit for data verify",
        default_value = "10000"
    )]
    pub max_limit: Option<u64>,

    #[clap(flatten)]
    pub(crate) context_options: WalletContextOptions,
}

impl ChangesetCommand {
    pub async fn execute(self) -> RoochResult<()> {
        let client = self.context_options.build()?.get_client().await?;

        let mut counter = 0u64;
        let per_limit = 200u64;
        let max_limit = self.max_limit.unwrap_or(10000);
        let tx_order = self.tx_order.unwrap_or(0);
        loop {
            if counter > max_limit {
                break;
            }

            let cursor = if tx_order >= counter {
                tx_order - counter + 1
            } else {
                0
            };
            let limit = if cursor >= per_limit {
                per_limit
            } else {
                cursor
            };
            let expect_tx_orders = ((cursor - limit)..cursor).collect::<Vec<_>>();
            let (data, _next_cursor, _has_next) = list_state_change_set(cursor, &client).await?;
            let result_tx_orders = data.iter().map(|v| v.tx_order.0).collect::<Vec<_>>();

            expect_tx_orders.iter().for_each(|v| {
                if !result_tx_orders.contains(v) {
                    println!("tx order {} in syncstate is inconsistent with statedb", v);
                }
            });

            if !expect_tx_orders.is_empty() {
                println!(
                    "verify tx order from {} to {} in syncstate and statedb done",
                    expect_tx_orders[0],
                    expect_tx_orders[expect_tx_orders.len() - 1]
                );
            }

            if limit < per_limit {
                break;
            }
            counter += per_limit;
        }

        Ok(())
    }
}

pub async fn list_state_change_set(
    tx_order: u64,
    client: &Client,
) -> anyhow::Result<(Vec<StateChangeSetWithTxOrderView>, Option<u64>, bool)> {
    let method = "rooch_syncStates";
    let filter = "all";
    let limit = 200u64;
    let params_str = format!(
        r#"["{}", "{}", "{}", {{"descending": true}}]"#,
        filter, tx_order, limit
    );
    let params_ret = serde_json::from_str(&params_str);
    let params = match params_ret {
        Ok(value) => value,
        Err(_) => {
            vec![serde_json::value::Value::String(params_str)]
        }
    };

    // Then cmd: "rpc request --method rooch_listStates --params '["/resource/0x3", null, null, {"decode":true}]' --json"
    let resp = client.request(method, params).await?;
    let state_change_set_view: StateChangeSetPageView = serde_json::from_value(resp)?;

    let next_cursor = state_change_set_view.next_cursor.map(|v| v.0);
    Ok((
        state_change_set_view.data,
        next_cursor,
        state_change_set_view.has_next_page,
    ))
}

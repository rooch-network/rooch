// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::config::DataConfig;
use crate::inscription::{
    read_ord_tx_json, resolve_inscription_body, InscriptionBody, Transaction,
};
use crate::rooch_client;
use anyhow::Result;
use rooch_rpc_api::jsonrpc_types::btc::ord::InscriptionStateView;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct OrdInscriptionInfo {
    pub id: String,
    pub txid: String,
    pub index: String,
    pub address: String,
    pub body: String,
    pub content_type: String,
}

impl OrdInscriptionInfo {
    pub fn new(tx: Transaction, inscription_body: InscriptionBody) -> Self {
        Self {
            id: tx.id,
            txid: tx.txid,
            index: tx.index,
            address: tx.address,
            body: inscription_body.body,
            content_type: inscription_body.content_type,
        }
    }
}

pub fn process_ord_data(
    config: &DataConfig,
    ord_tx_json: &str,
    _ord_inscription_succ_json: &str,
    _ord_inscription_fail_json: &str,
) -> Result<()> {
    // let transaction_path = format!("{}/data/id_txid_addr.json", project_path::PATH);
    if let Ok(transactions) = read_ord_tx_json(&ord_tx_json) {
        for transaction in transactions {
            let txid = &transaction.txid;
            let ord_inscription_body_result = resolve_inscription_body(config, txid);
            match ord_inscription_body_result {
                Ok(inscription_body_opt) => {

                    match inscription_body_opt {
                        Some(inscription_body) => {
                            let ord_inscription_info = OrdInscriptionInfo::new(transaction.clone(), inscription_body);
                            let rooch_inscription_info_result = rooch_client::query_inscription(config, transaction.txid.clone(), transaction.index.clone());
                            check_ord_data(&transaction, ord_inscription_info, rooch_inscription_info_result);
                        },
                        None => println!(
                            "process_ord_data txid: {} is not inscription transaction, no need to process!",
                            txid
                        ),
                    }
                }
                Err(err) => {
                    println!(
                        "[STAT] process_ord_data txid: {} index {} occurs error {}",
                        transaction.txid,
                        transaction.index,
                        err.to_string()
                    )
                }
            }
        }
    } else {
        eprintln!("Error read ord_tx_json file when process ord data");
    }

    Ok(())
}

pub fn check_ord_data(
    transaction: &Transaction,
    ord_inscription_info: OrdInscriptionInfo,
    rooch_inscription_info_result: Result<Option<InscriptionStateView>>,
) {
    match rooch_inscription_info_result {
        Ok(rooch_inscription_info_opt) => match rooch_inscription_info_opt {
            Some(rooch_inscription_info) => {
                let is_match = match_inscription_data(ord_inscription_info, rooch_inscription_info);
                if is_match {
                    println!(
                        "[STAT] check_ord_data match succ, txid: {} index {} ",
                        transaction.txid, transaction.index
                    )
                } else {
                    println!(
                        "[STAT] check_ord_data match fail, txid: {} index {} ",
                        transaction.txid, transaction.index
                    )
                }
            }
            None => println!(
                "[STAT] check_ord_data fail, txid: {} index {} rooch_inscription_info is none",
                transaction.txid, transaction.index
            ),
        },
        Err(err) => {
            println!(
                "[STAT] check_ord_data txid: {} index {} occurs error {}",
                transaction.txid,
                transaction.index,
                err.to_string()
            )
        }
    }
}

pub fn match_inscription_data(
    ord_inscription_info: OrdInscriptionInfo,
    rooch_inscription_info: InscriptionStateView,
) -> bool {
    let ord_inscription_info_body = format!("0x{}", ord_inscription_info.body);

    (rooch_inscription_info.owner_bitcoin_address.is_some()
        && ord_inscription_info.address
            == rooch_inscription_info
                .owner_bitcoin_address
                .unwrap()
                .to_string())
        && (rooch_inscription_info.value.content_type.is_some()
            && ord_inscription_info.content_type
                == rooch_inscription_info
                    .value
                    .content_type
                    .unwrap()
                    .to_string())
        && (ord_inscription_info_body == rooch_inscription_info.value.body.to_string())
}

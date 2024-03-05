// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::config::DataConfig;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs::File;
use std::io::{self, Read};
use std::process::Command;

// #[derive(Debug, Serialize, Deserialize)]
// pub struct OrdInscriptionInfo {
//     pub id: String,
//     pub txid: String,
//     pub index: String,
//     pub address: String,
//     pub body: String,
//     pub content_type: String,
// }

#[derive(Debug, Serialize, Deserialize)]
pub struct Transaction {
    pub id: String,
    pub txid: String,
    pub index: String,
    pub address: String,
}

pub fn read_ord_tx_json(file_path: &str) -> Result<Vec<Transaction>> {
    let mut file = File::open(file_path)?;
    let mut json_content = String::new();
    file.read_to_string(&mut json_content)?;

    let transactions: Vec<Transaction> = serde_json::from_str(&json_content)?;

    Ok(transactions)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InscriptionBody {
    pub body: String,
    pub content_type: String,
}

pub fn resolve_inscription_body(
    config: &DataConfig,
    txid: &str,
) -> Result<Option<InscriptionBody>> {
    let ord_cmd = format!(
        "{}ord decode --txid {} --compact | jq -r '.inscriptions[0]' ",
        // "{}ord decode --txid {} --compact | jq -r '.inscriptions[0].body' | xxd -r -p",
        // "{}ord -r decode --txid {} --compact | jq -r '.inscriptions[0].body' | xxd -r -p",
        config.ord_cli_path,
        txid
    );

    let output = Command::new("sh").arg("-c").arg(ord_cmd).output()?;

    // println!("Command output: {:?}", output);

    if output.status.success() {
        let inscription_body = String::from_utf8_lossy(&output.stdout);
        let json_value: serde_json::Value = serde_json::from_str(&inscription_body)?;

        let json_parsed_value = match json_value {
            serde_json::Value::Null => None,
            _ => {
                let inscription_body: InscriptionBody = serde_json::from_value(json_value)?;
                Some(inscription_body)
            }
        };

        Ok(json_parsed_value)
    } else {
        let error_message = String::from_utf8_lossy(&output.stderr);
        Err(anyhow::anyhow!(
            "resolve inscription body error {}",
            error_message.to_string()
        ))
    }
}

pub fn write_json(file_path: &str, data: &serde_json::Value) -> Result<()> {
    let json_content = serde_json::to_string_pretty(data)?;

    std::fs::write(file_path, json_content)?;

    Ok(())
}

pub fn process_ord_transactions(
    config: &DataConfig,
    ord_tx_json: &str,
    ord_inscription_json: &str,
) -> Result<()> {
    // let transaction_path = format!("{}/data/id_txid_addr.json", project_path::PATH);
    if let Ok(transactions) = read_ord_tx_json(&ord_tx_json) {
        let mut result: Vec<serde_json::Value> = Vec::new();

        for transaction in transactions {
            let txid = &transaction.txid;
            if let Ok(inscription_body) = resolve_inscription_body(config, txid) {
                let entry = json!({
                    "id": transaction.id,
                    "txid": txid,
                    "index": transaction.index,
                    "inscription_body": inscription_body,
                    "address": transaction.address
                });
                result.push(entry);
            } else {
                // eprintln!("txid: {} is not inscription transaction, no need to process!", txid);
                println!(
                    "txid: {} is not inscription transaction, no need to process!",
                    txid
                );
            }
        }

        let result_json_value: serde_json::Value = serde_json::Value::Array(result);
        if let Err(err) = write_json(&ord_inscription_json, &result_json_value) {
            eprintln!(
                "Error write ord inscription json file {}, error: {}",
                ord_inscription_json, err
            );
        }
    } else {
        eprintln!("Error read ord_tx_json file when process ord transactions");
    }

    Ok(())
}

// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::config::DataConfig;
use crate::inscription::Transaction;
use anyhow::Result;
use rooch_rpc_api::jsonrpc_types::btc::ord::InscriptionStateView;
use std::fs::File;
use std::io;
use std::io::BufRead;
use std::process::Command;

pub fn query_inscription(
    config: &DataConfig,
    txid: String,
    index: String,
) -> Result<Option<InscriptionStateView>> {
    // let index = index.parse::<u32>()?;
    let query_param = format!(
        "{{\"inscription_id\":{{\"txid\":\"{}\", \"index\":{} }} }}",
        txid, index
    );

    let rooch_cmd = format!(
        // "{}ord decode --txid {} --compact | jq -r '.inscriptions[0].body' | xxd -r -p",
        "{}rooch rpc request --method btc_queryInscriptions --params '[{}, null, \"1\", true]' | jq -r '.data[0]' ",
        config.ord_cli_path, query_param
    );

    let output = Command::new("sh").arg("-c").arg(rooch_cmd).output()?;

    // println!("Rooch command output: {:?}", output);

    if output.status.success() {
        let inscription_body = String::from_utf8_lossy(&output.stdout);
        let json_value: serde_json::Value = serde_json::from_str(&inscription_body)?;

        let json_parsed_value = match json_value {
            serde_json::Value::Null => None,
            _ => {
                let inscription_state_view: InscriptionStateView =
                    serde_json::from_value(json_value)?;
                Some(inscription_state_view)
            }
        };

        Ok(json_parsed_value)
    } else {
        let error_message = String::from_utf8_lossy(&output.stderr);
        Err(anyhow::anyhow!(
            "query inscription cli execute error {}",
            error_message.to_string()
        ))
    }
}

// #[derive(Debug, Serialize, Deserialize)]
// pub struct Transaction {
//     id: String,
//     txid: String,
//     index: String,
//     address: String,
// }

pub fn read_ord_export_data(file_path: &str, output_file: &str) -> Result<()> {
    let file = File::open(file_path)?;
    let reader = io::BufReader::new(file);
    let mut transactions = Vec::new();

    for line in reader.lines() {
        let line = line?;
        if line.starts_with('#') {
            continue;
        }
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() >= 4 {
            let id = parts[0].to_string();
            // 5fddcbdc3eb21a93e8dd1dd3f9087c3677f422b82d5ba39a6b1ec37338154af6:0:0
            let mut sat_point = parts[2].split(':');
            let txid = sat_point.next().unwrap_or_default().to_string();
            let index = sat_point.next().unwrap_or_default().to_string();
            let address = parts[3].to_string();

            let transaction = Transaction {
                id,
                txid,
                index,
                address,
            };
            transactions.push(transaction);
        }
    }

    write_json(output_file, &transactions)?;

    Ok(())
}

fn write_json(file_path: &str, transactions: &[Transaction]) -> Result<()> {
    let json_content = serde_json::to_string(transactions)?;
    std::fs::write(file_path, json_content)?;
    Ok(())
}

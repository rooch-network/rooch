// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::config::DataConfig;
use crate::inscription::Transaction;
use anyhow::Result;
use std::fs::File;
use std::io;
use std::io::BufRead;
use std::process::Command;

pub fn export_ord_index_data(config: &DataConfig, export_data_dir: &str) {
    let ord_cmd = format!("{}ord", config.ord_cli_path);
    let output = Command::new(ord_cmd)
        .arg("-r")
        .arg("index")
        .arg("export")
        .arg("--include-addresses")
        .arg("--tsv")
        .arg(export_data_dir)
        .output()
        .expect("Failed to execute the command");

    if output.status.success() {
        println!("Ord cli export ordinals indexer successfully!");
    } else {
        eprintln!("Ord cli export failed with error code: {:?}", output.status);
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

// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::config::DataConfig;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs::File;
use std::fs::OpenOptions;
use std::io;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Read;
use std::io::Write;

#[derive(Debug, Serialize, Deserialize)]
struct Transaction {
    address: String,
    id: String,
    inscription_body: Value,
    txid: String,
}

pub fn process_transactions(config: &DataConfig, file_path: &str) -> Result<()> {
    let transactions = read_json(file_path)?;
    let coin_path = format!("{}/data/coin.tsv", config.ord_data_path);
    let mut _coin_file = File::create(coin_path)?;

    let indexer_path = format!("{}/data/indexer.tsv", config.ord_data_path);
    let mut _indexer_file = File::create(indexer_path)?;

    for transaction in transactions {
        if let Some(op_value) = transaction.inscription_body.get("op") {
            let op_str = op_value.as_str().unwrap_or_default();
            // println!("{}", op_str);
            if op_str == "deploy" {
                if let (Some(tick_value), Some(max_value), Some(limit_value)) = (
                    transaction.inscription_body.get("tick"),
                    transaction.inscription_body.get("max"),
                    transaction.inscription_body.get("lim"),
                ) {
                    let tick_str = tick_value.as_str().unwrap_or_default();
                    let max_str = max_value.as_str().unwrap_or_default();
                    let limit_str = limit_value.as_str().unwrap_or_default();
                    // println!("{},{},{},{}", tick_str,max_str,limit_str,transaction.id);
                    if !tick_exists_in_coin_tsv(&config, &tick_str)? {
                        write_to_coin_tsv(
                            &config,
                            &tick_str,
                            &max_str,
                            &limit_str,
                            &transaction.id,
                        )?;
                        // println!("{},{},{}",tick_str,op_str,max_str)
                    }
                }
            } else if op_str == "mint" {
                if let (Some(tick_value), Some(amt_value)) = (
                    transaction.inscription_body.get("tick"),
                    transaction.inscription_body.get("amt"),
                ) {
                    let tick_str = tick_value.as_str().unwrap_or_default();
                    let amt_str = amt_value.as_str().unwrap_or_default();
                    // If tick has not been deployed and is not recorded in failinscription, then record it in failinscription
                    if !tick_exists_in_coin_tsv(&config, &tick_str)?
                        && !inscription_id_exists_in_failinscription_tsv(config, &transaction.txid)?
                    {
                        write_to_failinscription_tsv(config, &transaction.txid)?;
                    }
                    // If tick has not been deployed but is already recorded in failinscription, then skip
                    else if !tick_exists_in_coin_tsv(&config, &tick_str)?
                        && inscription_id_exists_in_failinscription_tsv(config, &transaction.txid)?
                    {
                        continue;
                    } else if tick_exists_in_coin_tsv(&config, &tick_str)? {
                        let (leftforsupply, limit) =
                            read_coin_tsv(config, tick_str).expect("Error reading coin.tsv");
                        // println!("tick:{} ,leftforsupply: {}, limit: {}", tick_str,leftforsupply, limit);
                        // Start comparison
                        // If limit < amt and failinscription has not been written yet, then write_to_failinscription_tsv(&transaction.txid)?;
                        let leftforsupply_numeric: i32 = leftforsupply.parse().unwrap_or_default();
                        let limit_numeric: i32 = limit.parse().unwrap_or_default();
                        let amt_numeric: i32 = amt_str.parse().unwrap_or_default();
                        if limit_numeric < amt_numeric
                            && !inscription_id_exists_in_failinscription_tsv(
                                config,
                                &transaction.txid,
                            )?
                        {
                            write_to_failinscription_tsv(config, &transaction.txid)?;
                        }
                        // If limit < amt and failinscription is written, then skip
                        if limit_numeric < amt_numeric
                            && inscription_id_exists_in_failinscription_tsv(
                                config,
                                &transaction.txid,
                            )?
                        {
                            continue;
                        }
                        // If leftforsupply is sufficient and amt has not exceeded the limit, then subtract amt from leftforsupply in coin.tsv, and simultaneously
                        // add tick, transaction.address, overallbalance, availablebalance, transferablebalance, transaction.id to the indexer.
                        // Where overallbalance += amt
                        // availablebalance  += amt
                        // transferablebalance remains unchanged, with an initial default value of 0
                        if leftforsupply_numeric >= amt_numeric && limit_numeric >= amt_numeric {
                            update_coin_tsv(config, tick_str, amt_numeric)
                                .expect("Error updating coin.tsv");
                            update_indexer_tsv(
                                config,
                                tick_str,
                                &transaction.address,
                                amt_numeric,
                                &transaction.id,
                            )
                            .expect("Error updating indexer.tsv");
                            // println!("{},{},{}",tick_str,op_str,amt_numeric)
                        }
                        // If leftforsupply is insufficient but amt has not exceeded the limit, then set leftforsupply in coin.tsv to 0;
                        // Simultaneously, add tick, transaction.address, overallbalance, availablebalance, transferablebalance, transaction.id to the indexer.
                        // Where overallbalance += leftforsupply
                        // availablebalance += leftforsupply
                        // transferablebalance remains unchanged, with an initial default value of 0

                        if leftforsupply_numeric < amt_numeric && limit_numeric >= amt_numeric {
                            update_coin_tsv(config, tick_str, leftforsupply_numeric).expect(
                                "Error updating coin.tsv when leftforsupply_numeric < amt_numeric",
                            );
                            update_indexer_tsv(config, tick_str, &transaction.address, leftforsupply_numeric, &transaction.id).expect("Error updating indexer.tsv when leftforsupply_numeric < amt_numeric");
                        }
                    }
                }
            } else if op_str == "transfer" {
                if let (Some(tick_value), Some(amt_value)) = (
                    transaction.inscription_body.get("tick"),
                    transaction.inscription_body.get("amt"),
                ) {
                    let tick_str = tick_value.as_str().unwrap_or_default();
                    let amt_str = amt_value.as_str().unwrap_or_default();
                    // If transaction.txid exists in failinscription.tsv, then skip
                    if inscription_id_exists_in_failinscription_tsv(config, &transaction.txid)? {
                        continue;
                    }
                    // If transaction.txid does not exist in failinscription.tsv
                    if !inscription_id_exists_in_failinscription_tsv(config, &transaction.txid)? {
                        // Check if tick\address exists in indexer.tsv
                        let _ = process_indexer_entry(
                            config,
                            tick_str,
                            &transaction.address,
                            amt_str,
                            &transaction.txid,
                            &transaction.id,
                        );
                    }
                }
            } else {
                continue;
            }
        }
    }
    Ok(())
}

fn process_indexer_entry(
    config: &DataConfig,
    tick_str: &str,
    address: &str,
    amt_str: &str,
    txid: &str,
    id: &str,
) -> Result<()> {
    let indexer1_path = format!("{}/data/indexer.tsv", config.ord_data_path);
    let temp_path = format!("{}/data/indexer_temp.tsv", config.ord_data_path);

    let indexer_file = File::open(indexer1_path)?;
    let mut temp_file = OpenOptions::new()
        .create(true)
        .write(true)
        .open(temp_path)?;

    let reader = BufReader::new(indexer_file);
    let mut entry_exists = false;

    for line_result in reader.lines() {
        let line = line_result?;
        let mut columns: Vec<&str> = line.split('\t').collect();

        if let (
            Some(existing_tick),
            Some(existing_address),
            Some(availablebalance),
            Some(transferablebalance),
        ) = (
            columns.first(),
            columns.get(1),
            columns.get(3),
            columns.get(4),
        ) {
            if existing_tick == &tick_str && existing_address == &address {
                entry_exists = true;
                let availablebalance_numeric: i32 = availablebalance.parse().unwrap_or_default();
                let amt_numeric: i32 = amt_str.parse().unwrap_or_default();

                if availablebalance_numeric < amt_numeric {
                    write_to_failinscription_tsv(config, &txid)?;
                } else {
                    let updated_availablebalance = availablebalance_numeric - amt_numeric;
                    let updated_transferablebalance =
                        transferablebalance.parse::<i32>().unwrap_or_default() + amt_numeric;

                    let updated_availablebalance = updated_availablebalance.to_string();
                    *columns.get_mut(3).unwrap() = Box::leak(Box::new(updated_availablebalance));

                    let updated_transferablebalance = updated_transferablebalance.to_string();
                    *columns.get_mut(4).unwrap() = Box::leak(Box::new(updated_transferablebalance));
                    if let Some(id_str) = columns.get_mut(5) {
                        let id = id.to_string();
                        *id_str = Box::leak(Box::new(id));
                    }
                }
            }
        }

        writeln!(temp_file, "{}", columns.join("\t"))?;
    }

    if !entry_exists {
        write_to_failinscription_tsv(config, &txid)?;
    }
    let indexer2_path = format!("{}/data/indexer.tsv", config.ord_data_path);

    let temp2_path = format!("{}/data/indexer_temp.tsv", config.ord_data_path);
    std::fs::rename(temp2_path, indexer2_path)?;

    Ok(())
}

fn update_indexer_tsv(
    config: &DataConfig,
    tick_str: &str,
    address: &str,
    amt_numeric: i32,
    id: &str,
) -> Result<()> {
    let indexer2_path = format!("{}/data/indexer.tsv", config.ord_data_path);
    let temp2_path = format!("{}/data/indexer_temp.tsv", config.ord_data_path);

    let indexer_file = File::open(indexer2_path)?;
    let mut temp_file = OpenOptions::new()
        .create(true)
        .write(true)
        .open(temp2_path)?;

    let reader = BufReader::new(indexer_file);

    let mut entry_exists = false;

    for line_result in reader.lines() {
        let line = line_result?;
        let mut columns: Vec<&str> = line.split('\t').collect();

        if columns.len() >= 2 {
            let existing_tick = columns[0];
            let existing_address = columns[1];

            if existing_tick == tick_str && existing_address == address {
                entry_exists = true;
                if let Some(overallbalance_str) = columns.get_mut(2) {
                    let overallbalance: i32 = overallbalance_str.parse().unwrap_or_default();
                    let updated_overallbalance = overallbalance + amt_numeric;
                    let updated_overallbalance_str = updated_overallbalance.to_string();
                    *overallbalance_str = Box::leak(Box::new(updated_overallbalance_str));
                }

                if let Some(availablebalance_str) = columns.get_mut(3) {
                    let availablebalance: i32 = availablebalance_str.parse().unwrap_or_default();
                    let updated_availablebalance = availablebalance + amt_numeric;
                    let updated_availablebalance = updated_availablebalance.to_string();
                    *availablebalance_str = Box::leak(Box::new(updated_availablebalance));
                }

                if let Some(id_str) = columns.get_mut(5) {
                    let id = id.to_string();
                    *id_str = Box::leak(Box::new(id));
                }
            }
        }

        writeln!(temp_file, "{}", columns.join("\t"))?;
    }

    if !entry_exists {
        writeln!(
            temp_file,
            "{}\t{}\t{}\t{}\t{}\t{}",
            tick_str, address, amt_numeric, amt_numeric, 0, id
        )?;
    }

    let indexer3_path = format!("{}/data/indexer.tsv", config.ord_data_path);
    let temp3_path = format!("{}/data/indexer_temp.tsv", config.ord_data_path);

    std::fs::rename(temp3_path, indexer3_path)?;

    Ok(())
}

fn update_coin_tsv(config: &DataConfig, tick_str: &str, amt_numeric: i32) -> Result<()> {
    let coin5_path = format!("{}/data/coin.tsv", config.ord_data_path);
    let temp5_path = format!("{}/data/coin_temp.tsv", config.ord_data_path);

    let coin_file = File::open(coin5_path)?;
    let mut temp_file = OpenOptions::new()
        .create(true)
        .write(true)
        .open(temp5_path)?;

    let reader = BufReader::new(coin_file);

    for line_result in reader.lines() {
        let line = line_result?;
        let mut columns: Vec<&str> = line.split('\t').collect();

        if let Some(existing_tick) = columns.first() {
            if existing_tick == &tick_str {
                if let Some(leftforsupply) = columns.get_mut(1) {
                    let leftforsupply_numeric: i32 = leftforsupply.parse().unwrap_or_default();
                    let updated_leftforsupply = leftforsupply_numeric - amt_numeric;
                    let updated_leftforsupply_str = updated_leftforsupply.to_string();
                    *leftforsupply = Box::leak(Box::new(updated_leftforsupply_str));
                }
            }
        }

        writeln!(temp_file, "{}", columns.join("\t"))?;
    }
    let coin6_path = format!("{}/data/coin.tsv", config.ord_data_path);
    let temp6_path = format!("{}/data/coin_temp.tsv", config.ord_data_path);
    std::fs::rename(temp6_path, coin6_path)?;

    Ok(())
}

fn read_coin_tsv(config: &DataConfig, name: &str) -> Result<(String, String)> {
    let coin6_path = format!("{}/data/coin.tsv", config.ord_data_path);
    let file = File::open(coin6_path)?;

    for line_result in io::BufReader::new(file).lines() {
        let line = line_result?;
        let columns: Vec<&str> = line.split('\t').collect();

        // Assuming the format is name, leftforsupply, limit, id
        if let Some(existing_name) = columns.first() {
            if existing_name == &name {
                if let (Some(leftforsupply), Some(limit)) = (columns.get(1), columns.get(2)) {
                    return Ok((leftforsupply.to_string(), limit.to_string()));
                }
            }
        }
    }

    // Err(io::Error::new(
    //     io::ErrorKind::NotFound,
    //     format!("Name '{}' not found in coin.tsv", name),
    // ))

    Err(anyhow::anyhow!("Name '{}' not found in coin.tsv", name))
}

fn tick_exists_in_coin_tsv(config: &DataConfig, tick: &str) -> Result<bool> {
    let coin7_path = format!("{}/data/coin.tsv", config.ord_data_path);
    let coin_file = OpenOptions::new().read(true).open(coin7_path);

    match coin_file {
        Ok(file) => {
            let reader = io::BufReader::new(file);
            for line_result in reader.lines() {
                let line = line_result?;
                let columns: Vec<&str> = line.split('\t').collect();
                if let Some(existing_tick) = columns.first() {
                    if existing_tick == &tick {
                        return Ok(true);
                    }
                }
            }
            Ok(false)
        }
        Err(_) => Ok(false),
    }
}

fn inscription_id_exists_in_failinscription_tsv(config: &DataConfig, txid: &str) -> Result<bool> {
    let failinscription_path = format!("{}/data/failinscription.tsv", config.ord_data_path);
    let failinscription_file = OpenOptions::new().read(true).open(failinscription_path);

    match failinscription_file {
        Ok(file) => {
            let reader = io::BufReader::new(file);
            for line_result in reader.lines() {
                let line = line_result?;
                let columns: Vec<&str> = line.split('\t').collect();
                if let Some(existing_txid) = columns.first() {
                    // let iid = ("{}i0",txid);
                    if existing_txid == &format!("{}i0", txid) {
                        return Ok(true);
                    }
                }
            }
            Ok(false)
        }
        Err(_) => Ok(false),
    }
}

fn write_to_coin_tsv(
    config: &DataConfig,
    tick: &str,
    leftforsupply: &str,
    limit: &str,
    id: &str,
) -> Result<()> {
    let coin8_path = format!("{}/data/coin.tsv", config.ord_data_path);
    let mut coin_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(coin8_path)?;
    writeln!(coin_file, "{}\t{}\t{}\t{}", tick, leftforsupply, limit, id)?;
    Ok(())
}

fn write_to_failinscription_tsv(config: &DataConfig, txid: &str) -> Result<()> {
    let failinscription1_path = format!("{}/data/failinscription.tsv", config.ord_data_path);
    let mut failinscription_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(failinscription1_path)?;
    writeln!(failinscription_file, "{}i0", txid)?;
    Ok(())
}

fn read_json(file_path: &str) -> Result<Vec<Transaction>> {
    let mut file = File::open(file_path)?;
    let mut json_content = String::new();
    file.read_to_string(&mut json_content)?;
    let transactions: Vec<Transaction> = serde_json::from_str(&json_content)?;
    Ok(transactions)
}

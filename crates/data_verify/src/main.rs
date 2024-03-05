// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use data_verify::config::DataConfig;
use data_verify::{brc20_helper, data_export, ord_verify};

//flight
fn main() {
    let config = DataConfig::default();
    let ord_export_data_path = format!("{}ord_export", config.ord_data_path);
    data_export::export_ord_index_data(&config, &ord_export_data_path.as_str());
    let ord_tx_json = format!("{}/ord_tx.json", config.ord_data_path);
    if let Err(err) = data_export::read_ord_export_data(&ord_export_data_path, &ord_tx_json) {
        eprintln!(
            "Error read ord export data {}, error: {}",
            ord_export_data_path, err
        );
    };
    let ord_inscription_succ_json = format!("{}/ord_inscription_succ.json", config.ord_data_path);
    let ord_inscription_fail_json = format!("{}/ord_inscription_fail.json", config.ord_data_path);
    let _ = ord_verify::process_ord_data(
        &config,
        &ord_tx_json,
        &ord_inscription_succ_json,
        &ord_inscription_fail_json,
    );
    // let ord_inscription_json = format!("{}/ord_inscription.json", config.ord_data_path);
    // let _ = brc20_helper::process_transactions(&config, &ord_inscription_json);
    println!("Indexer ord data verify successfully!")
}

// fmt::format, fs::{File, self}

#[cfg(test)]
mod tests {
    use data_verify::config;
    use data_verify::config::DataConfig;
    use data_verify::config::DataConfig;
    use serde::{Deserialize, Serialize};
    use serde_json::Value;
    use std::process::Command;

    #[test]
    fn test_check_chaindata_parse() {
        let ord_path = String::from("/Users/BCordinals/ord/target/release/ord");
        let ord_cookie = String::from("/Users/BC/ordinals/ord/target/release/.cookie");
        let delandlabsindexer_path = String::from("/Users/BC");
        let deploy_str = format!("{}tests/testdata/brc20_deploy.txt", &delandlabsindexer_path);
        let mint_str = format!("{}tests/testdata/brc20_mint.txt", &delandlabsindexer_path);
        let transfer_str = format!(
            "{}tests/testdata/brc20_transfer.txt",
            &delandlabsindexer_path
        );
        let indexerbin_str = format!("{}target/debug/delandlabsindexer", &delandlabsindexer_path);
        let _jsonscript_str = format!(
            "{}src/data/id_txid_inscription_addr.json",
            delandlabsindexer_path
        );

        let cmd1_output = Command::new(ord_path.clone())
            .arg("--cookie-file")
            .arg(&ord_cookie)
            .arg("-r")
            .arg("wallet")
            .arg("inscribe")
            .arg("--fee-rate")
            .arg("5")
            .arg("--file")
            .arg(deploy_str)
            .output()
            .expect("Failed to execute cmd1");

        let result1 = String::from_utf8_lossy(&cmd1_output.stdout);
        let error1 = String::from_utf8_lossy(&cmd1_output.stderr);

        if cmd1_output.status.success() {
            println!("{}", result1);
        } else {
            eprintln!(
                "cmd1 failed with exit code: {:?}",
                cmd1_output.status.code()
            );
            eprintln!("cmd1 output: {}", result1);
            eprintln!("cmd1 error: {}", error1);
        }

        let cmd2_output = Command::new(ord_path.clone())
            .arg("--cookie-file")
            .arg(&ord_cookie)
            .arg("-r")
            .arg("wallet")
            .arg("inscribe")
            .arg("--fee-rate")
            .arg("5")
            .arg("--file")
            .arg(mint_str)
            .output()
            .expect("Failed to execute cmd2");

        let result2 = String::from_utf8_lossy(&cmd1_output.stdout);
        let error2 = String::from_utf8_lossy(&cmd1_output.stderr);

        if cmd2_output.status.success() {
            // println!("{}",result1);
        } else {
            eprintln!(
                "cmd1 failed with exit code: {:?}",
                cmd2_output.status.code()
            );
            eprintln!("cmd1 output: {}", result2);
            eprintln!("cmd1 error: {}", error2);
        }

        let cmd3_output = Command::new(ord_path.clone())
            .arg("--cookie-file")
            .arg(&ord_cookie)
            .arg("-r")
            .arg("wallet")
            .arg("inscribe")
            .arg("--fee-rate")
            .arg("5")
            .arg("--file")
            .arg(transfer_str)
            .output()
            .expect("Failed to execute cmd3");

        let result3 = String::from_utf8_lossy(&cmd3_output.stdout);
        let error3 = String::from_utf8_lossy(&cmd3_output.stderr);

        if cmd3_output.status.success() {
            // println!("{}",result3);
        } else {
            eprintln!(
                "cmd3 failed with exit code: {:?}",
                cmd3_output.status.code()
            );
            eprintln!("cmd3 output: {}", result3);
            eprintln!("cmd3 error: {}", error3);
        }

        //bitcoin-cli -regtest -rpcwallet=wallet1 -generate 2
        let cmd4_output = Command::new("bitcoin-cli")
            .arg("-regtest")
            .arg("-rpcwallet=wallet1")
            .arg("-generate")
            .arg("2")
            .output()
            .expect("Failed to execute cmd4");

        let result4 = String::from_utf8_lossy(&cmd4_output.stdout);
        let error4 = String::from_utf8_lossy(&cmd4_output.stderr);

        if cmd4_output.status.success() {
            // println!("{}",result4);
        } else {
            eprintln!(
                "cmd4 failed with exit code: {:?}",
                cmd4_output.status.code()
            );
            eprintln!("cmd4 output: {}", result4);
            eprintln!("cmd4 error: {}", error4);
        }

        let cmd5_output = Command::new(indexerbin_str)
            .output()
            .expect("Failed to execute cmd4");

        let result5 = String::from_utf8_lossy(&cmd5_output.stdout);
        let error5 = String::from_utf8_lossy(&cmd5_output.stderr);

        if cmd5_output.status.success() {
            // println!("{}",result5);
        } else {
            eprintln!(
                "cmd5 failed with exit code: {:?}",
                cmd5_output.status.code()
            );
            eprintln!("cmd5 output: {}", result5);
            eprintln!("cmd5 error: {}", error5);
        }

        //compare

        #[derive(Debug, Deserialize, Serialize)]
        struct Brc20Deploy {
            p: String,
            op: String,
            tick: String,
            max: String,
            lim: String,
        }
        let deploy_json_str = include_str!("../tests/testdata/brc20_deploy.txt");
        // println!("Text data from ./testdata/brc20_deploy.txt:\n{}", deploy_json_str);

        #[derive(Debug, Deserialize, Serialize)]
        struct Brc20Operation {
            p: String,
            op: String,
            tick: String,
            amt: String,
        }
        let mint_json_str = include_str!("../tests/testdata/brc20_mint.txt");
        // println!("Text data from ./testdata/brc20_mint.txt:\n{}", mint_json_str);

        let transfer_json_str = include_str!("../tests/testdata/brc20_transfer.txt");
        // println!("Text data from ./testdata/brc20_transfer.txt:\n{}", transfer_json_str);

        #[derive(Debug, Deserialize, Serialize)]
        struct Transaction {
            address: String,
            id: String,
            inscription_body: Value,
            txid: String,
        }
        let json_str = include_str!("../src/data/id_txid_inscription_addr.json");
        let json_value: Value = serde_json::from_str(json_str).expect("Failed to parse JSON");
        let transactions: Vec<Transaction> =
            serde_json::from_value(json_value).expect("Failed to parse Transaction");
        // println!("Parsed Transaction: {:?}", transactions);
        for transaction in transactions.iter().rev().take(3) {
            // println!("{:?}",transaction.inscription_body)
            if let Some(op_value) = transaction.inscription_body.get("op") {
                let op_str = op_value.as_str().unwrap_or_default();
                // println!("{}", op_str);
                if op_str == "deploy" {
                    if let Ok(brc20_deploy) = serde_json::from_str::<Brc20Deploy>(&deploy_json_str)
                    {
                        let p_str = brc20_deploy.p;
                        if let Some(inscription_p) = transaction.inscription_body.get("p") {
                            let inscription_p_str = inscription_p.as_str().unwrap_or_default();
                            assert_eq!(
                                inscription_p_str, p_str,
                                "Comparison for p field in deploy operation failed"
                            );
                        } else {
                            panic!("Missing p field in inscription_body for deploy operation");
                        }

                        let max_str = &brc20_deploy.max;
                        if let Some(inscription_max) = transaction.inscription_body.get("max") {
                            let inscription_max_str = inscription_max.as_str().unwrap_or_default();
                            assert_eq!(
                                inscription_max_str, max_str,
                                "Comparison for max field in deploy operation failed"
                            );
                        } else {
                            panic!("Missing max field in inscription_body for deploy operation");
                        }

                        let lim_str = &brc20_deploy.lim;
                        if let Some(inscription_lim) = transaction.inscription_body.get("lim") {
                            let inscription_lim_str = inscription_lim.as_str().unwrap_or_default();
                            assert_eq!(
                                inscription_lim_str, lim_str,
                                "Comparison for lim field in deploy operation failed"
                            );
                        } else {
                            panic!("Missing lim field in inscription_body for deploy operation");
                        }
                        let tick_str = &brc20_deploy.tick;
                        if let Some(inscription_tick) = transaction.inscription_body.get("tick") {
                            let inscription_tick_str =
                                inscription_tick.as_str().unwrap_or_default();
                            assert_eq!(
                                inscription_tick_str, tick_str,
                                "Comparison for tick field failed"
                            );
                        } else {
                            panic!("Missing tick field in inscription_body");
                        }

                        let op_str = &brc20_deploy.op;
                        if let Some(inscription_op) = transaction.inscription_body.get("op") {
                            let inscription_op_str = inscription_op.as_str().unwrap_or_default();
                            assert_eq!(
                                inscription_op_str, op_str,
                                "Comparison for op field failed"
                            );
                        } else {
                            panic!("Missing op field in inscription_body");
                        }
                    } else {
                        eprintln!("Failed to parse JSON");
                    }
                }
                if op_str == "mint" {
                    if let Ok(brc20_mint) = serde_json::from_str::<Brc20Operation>(&mint_json_str) {
                        // Processing for brc20_mint, adapt based on your actual scenario
                        let p_str = brc20_mint.p;
                        if let Some(inscription_p) = transaction.inscription_body.get("p") {
                            let inscription_p_str = inscription_p.as_str().unwrap_or_default();
                            assert_eq!(
                                inscription_p_str, p_str,
                                "Comparison for p field in mint operation failed"
                            );
                        } else {
                            panic!("Missing p field in inscription_body for mint operation");
                        }

                        let amt_str = &brc20_mint.amt;
                        if let Some(inscription_amt) = transaction.inscription_body.get("amt") {
                            let inscription_amt_str = inscription_amt.as_str().unwrap_or_default();
                            assert_eq!(
                                inscription_amt_str, amt_str,
                                "Comparison for amt field in mint operation failed"
                            );
                        } else {
                            panic!("Missing amt field in inscription_body for mint operation");
                        }

                        let tick_str = &brc20_mint.tick;
                        if let Some(inscription_tick) = transaction.inscription_body.get("tick") {
                            let inscription_tick_str =
                                inscription_tick.as_str().unwrap_or_default();
                            assert_eq!(
                                inscription_tick_str, tick_str,
                                "Comparison for tick field failed"
                            );
                        } else {
                            panic!("Missing tick field in inscription_body");
                        }

                        let op_str = &brc20_mint.op;
                        if let Some(inscription_op) = transaction.inscription_body.get("op") {
                            let inscription_op_str = inscription_op.as_str().unwrap_or_default();
                            assert_eq!(
                                inscription_op_str, op_str,
                                "Comparison for op field failed"
                            );
                        } else {
                            panic!("Missing op field in inscription_body");
                        }
                    } else {
                        // JSON parsing failure handling
                        eprintln!("Failed to parse JSON for mint operation");
                    }
                }
                if op_str == "transfer" {
                    if let Ok(brc20_transfer) =
                        serde_json::from_str::<Brc20Operation>(&transfer_json_str)
                    {
                        // Processing for brc20_transfer, adapt based on your actual scenario
                        let p_str = brc20_transfer.p;
                        if let Some(inscription_p) = transaction.inscription_body.get("p") {
                            let inscription_p_str = inscription_p.as_str().unwrap_or_default();
                            assert_eq!(
                                inscription_p_str, p_str,
                                "Comparison for p field in transfer operation failed"
                            );
                        } else {
                            panic!("Missing p field in inscription_body for transfer operation");
                        }

                        let amt_str = &brc20_transfer.amt;
                        if let Some(inscription_amt) = transaction.inscription_body.get("amt") {
                            let inscription_amt_str = inscription_amt.as_str().unwrap_or_default();
                            assert_eq!(
                                inscription_amt_str, amt_str,
                                "Comparison for amt field in transfer operation failed"
                            );
                        } else {
                            panic!("Missing amt field in inscription_body for transfer operation");
                        }

                        let tick_str = &brc20_transfer.tick;
                        if let Some(inscription_tick) = transaction.inscription_body.get("tick") {
                            let inscription_tick_str =
                                inscription_tick.as_str().unwrap_or_default();
                            assert_eq!(
                                inscription_tick_str, tick_str,
                                "Comparison for tick field failed"
                            );
                        } else {
                            panic!("Missing tick field in inscription_body");
                        }

                        let op_str = &brc20_transfer.op;
                        if let Some(inscription_op) = transaction.inscription_body.get("op") {
                            let inscription_op_str = inscription_op.as_str().unwrap_or_default();
                            assert_eq!(
                                inscription_op_str, op_str,
                                "Comparison for op field failed"
                            );
                        } else {
                            panic!("Missing op field in inscription_body");
                        }
                    } else {
                        // JSON parsing failure handling
                        eprintln!("Failed to parse JSON for transfer operation");
                    }
                }
            }
        }
    }

    #[test]
    fn test_brc20_jsonscript_analyse() {
        let config = DataConfig::default();
        let export_json =
            format!("/Users/BC/tests/testdata/id_txid_inscription_addr_test.json");
        let _ = crate::brc20_helper::process_transactions(&config, &export_json);
        let export_indexer = format!("{}/data/indexer.tsv", config.ord_data_path);

        // Expected content
        let expected_content = "dela\tbcrt1p5dm3qd9sg323ach2gw7jy944kp98p42kusr4udg6fz2k5lh96z0q7zgruk\t4000\t3000\t1000\t6";

        // Read the content of the file
        if let Ok(indexer_content) = std::fs::read_to_string(&export_indexer) {
            // Compare each item
            let indexer_items: Vec<&str> = indexer_content.split('\t').collect();
            let expected_items: Vec<&str> = expected_content.split('\t').collect();

            // Check if the first 5 items are the same
            for i in 0..5 {
                if indexer_items[i] != expected_items[i] {
                    // Print which item is different
                    assert_eq!(
                        indexer_items[i],
                        expected_items[i],
                        "Mismatch at item {}: Expected '{}', Actual '{}'",
                        i + 1,
                        expected_items[i],
                        indexer_items[i]
                    );
                }
            }
        } else {
            // Handle the file read error
            eprintln!("Failed to read file: {}", export_indexer);
            assert!(false); // Mark the test as failed
        }
    }
}

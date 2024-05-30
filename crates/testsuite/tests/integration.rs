// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

mod images;

use anyhow::{bail, Result};
use clap::Parser;
use cucumber::{given, then, World as _};
use images::bitcoin::BitcoinD;
use images::bitseed::Bitseed;
use images::ord::Ord;
use jpst::TemplateContext;
use rooch::RoochCli;
use rooch_config::{rooch_config_dir, RoochOpt, ServerOpt};
use rooch_rpc_client::wallet_context::WalletContext;
use rooch_rpc_server::Service;
use rooch_types::crypto::RoochKeyPair;
use serde_json::Value;
use std::path::Path;
use std::time::Duration;
use testcontainers::{
    clients::Cli,
    core::{Container, ExecCommand, WaitFor},
    RunnableImage,
};
use tracing::{debug, error, info, trace};
use uuid::Uuid;

const RPC_USER: &str = "roochuser";
const RPC_PASS: &str = "roochpass";
const RPC_PORT: u16 = 18443;
const ORD_RPC_PORT: u16 = 80;

#[derive(cucumber::World, Debug)]
struct World {
    docker: Cli,
    container_network: String,
    service: Option<Service>,
    bitcoind: Option<Container<BitcoinD>>,
    ord: Option<Container<Ord>>,
    tpl_ctx: Option<TemplateContext>,
}

impl Default for World {
    fn default() -> Self {
        let network_uuid = Uuid::new_v4();

        World {
            docker: Cli::default(),
            container_network: format!("test_network_{}", network_uuid),
            service: None,
            bitcoind: None,
            ord: None,
            tpl_ctx: None,
        }
    }
}

#[given(expr = "a server for {word}")] // Cucumber Expression
async fn start_server(w: &mut World, _scenario: String) {
    tokio::time::sleep(Duration::from_secs(5)).await;

    let mut service = Service::new();
    let mut opt = RoochOpt::new_with_temp_store();
    wait_port_available(opt.port()).await;

    match w.bitcoind.take() {
        Some(bitcoind) => {
            let bitcoin_rpc_url =
                format!("http://127.0.0.1:{}", bitcoind.get_host_port_ipv4(RPC_PORT));
            opt.btc_rpc_url = Some(bitcoin_rpc_url);
            opt.btc_rpc_username = Some(RPC_USER.to_string());
            opt.btc_rpc_password = Some(RPC_PASS.to_string());
            opt.data_import_flag = false; // Enable data import without writing indexes
            opt.btc_sync_block_interval = Some(1u64); // Update sync interval as 1s

            info!("config btc rpc ok");

            w.bitcoind = Some(bitcoind);
        }
        None => {
            info!("bitcoind server is none");
        }
    }

    let mut server_opt = ServerOpt::new();

    let kp: RoochKeyPair = RoochKeyPair::generate_secp256k1();
    server_opt.sequencer_keypair = Some(kp.copy());
    server_opt.proposer_keypair = Some(kp.copy());

    service.start(&opt, server_opt).await.unwrap();

    w.service = Some(service);
}

#[then(expr = "stop the server")] // Cucumber Expression
async fn stop_server(w: &mut World) {
    println!("stop server");
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    match w.service.take() {
        Some(service) => {
            service.stop().unwrap();
            info!("Shutdown Sever");
        }
        None => {
            info!("service is none");
        }
    }
}

#[given(expr = "a bitcoind server for {word}")] // Cucumber Expression
async fn start_bitcoind_server(w: &mut World, _scenario: String) {
    tokio::time::sleep(Duration::from_secs(5)).await;

    let mut bitcoind_image: RunnableImage<BitcoinD> = BitcoinD::new(
        format!("0.0.0.0:{}", RPC_PORT),
        RPC_USER.to_string(),
        RPC_PASS.to_string(),
    )
    .into();
    bitcoind_image = bitcoind_image
        .with_network(w.container_network.clone())
        .with_run_option(("--network-alias", "bitcoind"));

    let bitcoind = w.docker.run(bitcoind_image);
    debug!("bitcoind ok");

    w.bitcoind = Some(bitcoind);
}

#[then(expr = "stop the bitcoind server")] // Cucumber Expression
async fn stop_bitcoind_server(w: &mut World) {
    println!("stop server");
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    match w.bitcoind.take() {
        Some(bitcoind) => {
            bitcoind.stop();
            info!("shutdown bitcoind server");
        }
        None => {
            info!("bitcoind server is none");
        }
    }
}

#[given(expr = "a ord server for {word}")] // Cucumber Expression
async fn start_ord_server(w: &mut World, _scenario: String) {
    tokio::time::sleep(Duration::from_secs(5)).await;

    let mut ord_image: RunnableImage<Ord> = Ord::new(
        format!("http://bitcoind:{}", RPC_PORT),
        RPC_USER.to_string(),
        RPC_PASS.to_string(),
    )
    .into();
    ord_image = ord_image
        .with_network(w.container_network.clone())
        .with_run_option(("--network-alias", "ord"));

    let ord = w.docker.run(ord_image);
    debug!("ord ok");

    w.ord = Some(ord);
}

#[then(expr = "stop the ord server")] // Cucumber Expression
async fn stop_ord_server(w: &mut World) {
    println!("stop ord server");
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    match w.ord.take() {
        Some(ord) => {
            ord.stop();
            info!("shutdown ord server");
        }
        None => {
            info!("ord server is none");
        }
    }
}

#[then(regex = r#"sleep: "(.*)?""#)]
async fn sleep(_world: &mut World, args: String) {
    let args = args.trim().parse::<u64>().unwrap();
    debug!("sleep: {}", args);
    tokio::time::sleep(tokio::time::Duration::from_secs(args)).await;
}

#[then(regex = r#"cmd: "(.*)?""#)]
async fn run_cmd(world: &mut World, args: String) {
    let config_dir = rooch_config_dir()
        .unwrap()
        .parent()
        .unwrap()
        .join("rooch_test");
    debug!("config_dir: {:?}", config_dir);
    if world.tpl_ctx.is_none() {
        let mut tpl_ctx = TemplateContext::new();
        if config_dir.exists() {
            let context = WalletContext::new(Some(config_dir.clone())).unwrap();
            let address_mapping = serde_json::Value::Object(
                context
                    .address_mapping
                    .iter()
                    .map(|(k, v)| (k.to_string(), Value::String(v.to_hex_literal())))
                    .collect(),
            );
            tpl_ctx.entry("address_mapping").set(address_mapping);
        }
        world.tpl_ctx = Some(tpl_ctx);
    }
    let tpl_ctx = world.tpl_ctx.as_mut().unwrap();
    let args = eval_command_args(tpl_ctx, args);

    let mut args = split_string_with_quotes(&args).expect("Invalid commands");
    let cmd_name = args[0].clone();
    args.insert(0, "rooch".to_owned());
    args.push("--config-dir".to_owned());
    args.push(config_dir.to_str().unwrap().to_string());
    let opts: RoochCli = RoochCli::parse_from(args);
    let ret = rooch::run_cli(opts).await;

    match ret {
        Ok(output) => {
            let result_json = serde_json::from_str::<Value>(&output);
            if result_json.is_ok() {
                debug!("run_cli {} ok: {:?}", cmd_name, &result_json);

                tpl_ctx
                    .entry(cmd_name)
                    .append::<Value>(result_json.unwrap());
            } else {
                debug!("run_cli {} result not json: {:?}", cmd_name, &output);
            }
        }
        Err(err) => {
            debug!("run_cli cmd: {} output err: {}", cmd_name, err.to_string());
            let err_msg = Value::String(err.to_string());
            error!("run_cli cmd: {} fail: {:?}", cmd_name, &err_msg);
            info!("current tpl_ctx: \n {:#}", tpl_ctx.as_value());
            tpl_ctx.entry(cmd_name).append::<Value>(err_msg);
        }
    }
    trace!("current tpl_ctx: {:#}", tpl_ctx.as_value());
}

#[then(regex = r#"cmd ord bash: "(.*)?""#)]
fn ord_bash_run_cmd(w: &mut World, input_tpl: String) {
    let ord = w.ord.as_ref().unwrap();

    let mut bitseed_args = vec!["/bin/bash".to_string()];

    if w.tpl_ctx.is_none() {
        let tpl_ctx = TemplateContext::new();
        w.tpl_ctx = Some(tpl_ctx);
    }
    let tpl_ctx = w.tpl_ctx.as_mut().unwrap();
    let input = eval_command_args(tpl_ctx, input_tpl);

    let args: Vec<&str> = input.split_whitespace().collect();
    let cmd_name = args[0];

    bitseed_args.extend(args.iter().map(|&s| s.to_string()));

    let joined_args = bitseed_args.join(" ");
    debug!("run cmd: ord {}", joined_args);

    let exec_cmd = ExecCommand {
        cmd: joined_args,
        ready_conditions: vec![WaitFor::Nothing],
    };

    let output = ord.exec(exec_cmd);

    let stdout_string = match String::from_utf8(output.stdout) {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to convert stdout to String: {}", e);
            String::from("Error converting stdout to String")
        }
    };

    let stderr_string = match String::from_utf8(output.stderr) {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to convert stderr to String: {}", e);
            String::from("Error converting stderr to String")
        }
    };

    debug!("run cmd: ord stdout: {}", stdout_string);

    // Check if stderr_string is not empty and panic if it contains any content.
    if !stderr_string.is_empty() {
        panic!("Command execution failed with errors: {}", stderr_string);
    }

    tpl_ctx
        .entry(format!("{}", cmd_name))
        .append::<String>(stdout_string);

    debug!("current tpl_ctx: {:?}", tpl_ctx);
}

#[then(regex = r#"cmd bitseed: "(.*)?""#)]
async fn bitseed_run_cmd(w: &mut World, input_tpl: String) {
    let _bitcoind = w.bitcoind.as_ref().unwrap();
    let _ord = w.ord.as_ref().unwrap();

    let mut bitseed_args = vec![];

    if w.tpl_ctx.is_none() {
        let tpl_ctx = TemplateContext::new();
        w.tpl_ctx = Some(tpl_ctx);
    }
    let tpl_ctx = w.tpl_ctx.as_mut().unwrap();
    let input = eval_command_args(tpl_ctx, input_tpl);

    let args: Vec<&str> = input.split_whitespace().collect();
    let cmd_name = args[0];

    bitseed_args.extend(args.iter().map(|&s| s.to_string()));

    let joined_args = bitseed_args.join(" ");
    debug!("run cmd: bitseed {}", joined_args);

    let mut bitseed_image: RunnableImage<Bitseed> = Bitseed::new(
        format!("http://bitcoind:{}", RPC_PORT),
        RPC_USER.to_string(),
        RPC_PASS.to_string(),
        format!("http://ord:{}", ORD_RPC_PORT),
        bitseed_args,
    )
    .into();

    let test_data_path = Path::new("./data")
        .canonicalize()
        .unwrap()
        .into_os_string()
        .into_string()
        .unwrap_or_else(|_| panic!("Invalid Unicode path"));
    debug!("test_data_path: {}", test_data_path);

    bitseed_image = bitseed_image
        .with_network(w.container_network.clone())
        .with_volume((test_data_path, "/app/test-data"));

    let mut bitseed_cmd = w.docker.run_cmd(bitseed_image);
    let output = bitseed_cmd.output().expect("run bitseed cmd should be ok");

    let stdout_string = match String::from_utf8(output.stdout) {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to convert stdout to String: {}", e);
            String::from("Error converting stdout to String")
        }
    };

    let stderr_string = match String::from_utf8(output.stderr) {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to convert stderr to String: {}", e);
            String::from("Error converting stderr to String")
        }
    };

    debug!("run cmd: bitseed stdout: {}", stdout_string);

    // Check if stderr_string is not empty and panic if it contains any content.
    if !stderr_string.is_empty() {
        panic!("Command execution failed with errors: {}", stderr_string);
    }

    let result_json = extract_json(&stdout_string);
    if let Ok(json_value) = result_json {
        debug!("cmd bitseed: {} output: {}", cmd_name, json_value);
        tpl_ctx.entry(cmd_name).append::<Value>(json_value);
    } else {
        debug!("result_json not ok!");
    }

    debug!("current tpl_ctx: {:?}", tpl_ctx);
}

#[then(regex = r#"cmd ord: "(.*)?""#)]
fn ord_run_cmd(w: &mut World, input_tpl: String) {
    let ord = w.ord.as_ref().unwrap();

    let mut bitseed_args = vec![
        "ord".to_string(),
        "--regtest".to_string(),
        format!("--rpc-url=http://bitcoind:{}", RPC_PORT),
        format!("--bitcoin-rpc-user={}", RPC_USER),
        format!("--bitcoin-rpc-pass={}", RPC_PASS),
    ];

    if w.tpl_ctx.is_none() {
        let tpl_ctx = TemplateContext::new();
        w.tpl_ctx = Some(tpl_ctx);
    }
    let tpl_ctx = w.tpl_ctx.as_mut().unwrap();
    let input = eval_command_args(tpl_ctx, input_tpl);

    let args: Vec<&str> = input.split_whitespace().collect();
    let cmd_name = args[0];

    bitseed_args.extend(args.iter().map(|&s| s.to_string()));

    let joined_args = bitseed_args.join(" ");
    debug!("run cmd: ord {}", joined_args);

    let exec_cmd = ExecCommand {
        cmd: joined_args,
        ready_conditions: vec![WaitFor::Nothing],
    };

    let output = ord.exec(exec_cmd);

    let stdout_string = match String::from_utf8(output.stdout) {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to convert stdout to String: {}", e);
            String::from("Error converting stdout to String")
        }
    };

    let stderr_string = match String::from_utf8(output.stderr) {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to convert stderr to String: {}", e);
            String::from("Error converting stderr to String")
        }
    };

    debug!("run cmd: ord stdout: {}", stdout_string);

    // Check if stderr_string is not empty and panic if it contains any content.
    if !stderr_string.is_empty() {
        panic!("Command execution failed with errors: {}", stderr_string);
    }

    let result_json = serde_json::from_str::<Value>(&stdout_string);
    if let Ok(json_value) = result_json {
        debug!("cmd ord: {} output: {}", cmd_name, json_value);
        tpl_ctx.entry(cmd_name).append::<Value>(json_value);
    } else {
        debug!("result_json not ok!");
    }

    debug!("current tpl_ctx: {:?}", tpl_ctx);
}

#[then(regex = r#"cmd bitcoin-cli: "(.*)?""#)]
fn bitcoincli_run_cmd(w: &mut World, input_tpl: String) {
    let bitcoind = w.bitcoind.as_ref().unwrap();

    let mut bitcoincli_args = vec!["bitcoin-cli".to_string(), "-regtest".to_string()];

    if w.tpl_ctx.is_none() {
        let tpl_ctx = TemplateContext::new();
        w.tpl_ctx = Some(tpl_ctx);
    }
    let tpl_ctx = w.tpl_ctx.as_mut().unwrap();
    let input = eval_command_args(tpl_ctx, input_tpl);

    let args: Vec<&str> = input.split_whitespace().collect();
    let cmd_name = args[0];

    bitcoincli_args.extend(args.iter().map(|&s| s.to_string()));

    let joined_args = bitcoincli_args.join(" ");
    debug!("run cmd: {}", joined_args);

    let exec_cmd = ExecCommand {
        cmd: joined_args,
        ready_conditions: vec![WaitFor::Nothing],
    };

    let output = bitcoind.exec(exec_cmd);

    let stdout_string = match String::from_utf8(output.stdout) {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to convert stdout to String: {}", e);
            String::from("Error converting stdout to String")
        }
    };

    let stderr_string = match String::from_utf8(output.stderr) {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to convert stderr to String: {}", e);
            String::from("Error converting stderr to String")
        }
    };

    debug!("run cmd: bitcoincli stdout: {}", stdout_string);

    // Check if stderr_string is not empty and panic if it contains any content.
    if !stderr_string.is_empty() {
        panic!("Command execution failed with errors: {}", stderr_string);
    }

    let result_json = serde_json::from_str::<Value>(&stdout_string);
    if let Ok(json_value) = result_json {
        debug!("cmd bitcoincli: {} output: {}", cmd_name, json_value);
        tpl_ctx.entry(cmd_name).append::<Value>(json_value);
    } else {
        debug!("result_json not ok!");
    }

    debug!("current tpl_ctx: {:?}", tpl_ctx);
}

#[then(regex = r#"assert: "([^"]*)""#)]
async fn assert_output(world: &mut World, orginal_args: String) {
    assert!(world.tpl_ctx.is_some(), "tpl_ctx is none");
    assert!(orginal_args.len() > 0, "assert args is empty");
    let args = eval_command_args(world.tpl_ctx.as_ref().unwrap(), orginal_args.clone());
    let splited_args = split_string_with_quotes(&args).expect("Invalid commands");
    debug!(
        "originl args: {}\n after eval: {}\n after split: {:?}",
        orginal_args, args, splited_args
    );
    assert!(
        !splited_args.is_empty(),
        "splited_args should not empty, the orginal_args:{}",
        orginal_args
    );
    for chunk in splited_args.chunks(3) {
        let first = chunk.get(0).cloned();
        let op = chunk.get(1).cloned();
        let second = chunk.get(2).cloned();

        debug!("assert value: {:?} {:?} {:?}", first, op, second);

        match (first, op, second) {
            (Some(first), Some(op), Some(second)) => match op.as_str() {
                "==" => assert_eq!(first, second, "Assert {:?} == {:?} failed", first, second),
                "!=" => assert_ne!(first, second, "Assert {:?} 1= {:?} failed", first, second),
                "contains" => assert!(
                    first.contains(&second),
                    "Assert {:?} contains {:?} failed",
                    first,
                    second
                ),
                "not_contains" => assert!(
                    !first.contains(&second),
                    "Assert {:?} not_contains {:?} failed",
                    first,
                    second
                ),
                _ => panic!("unsupported operator {:?}", op.as_str()),
            },
            _ => panic!(
                "expected 3 arguments: first [==|!=] second, but got input {:?}",
                args
            ),
        }
    }
    info!("assert ok!");
}

fn eval_command_args(ctx: &TemplateContext, args: String) -> String {
    let eval_args = jpst::format_str!(&args, ctx);
    eval_args
}

/// Split a string into a vector of strings, splitting on spaces, but ignoring spaces inside quotes.
/// And quotes will alse be removed.
fn split_string_with_quotes(s: &str) -> Result<Vec<String>> {
    let mut result = Vec::new();
    let mut chars = s.chars().peekable();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut in_escape = false;
    let mut in_single_quotes = false;

    while let Some(c) = chars.next() {
        match c {
            '\\' => {
                in_escape = true;
            }
            '"' => {
                if in_escape {
                    current.push(c);
                    in_escape = false;
                } else if in_single_quotes {
                    current.push(c);
                } else {
                    // Skip the quote
                    in_quotes = !in_quotes;
                }
            }
            '\'' => {
                if in_escape {
                    current.push(c);
                    in_escape = false;
                } else if in_quotes {
                    current.push(c);
                } else {
                    // Skip the quote
                    in_single_quotes = !in_single_quotes;
                }
            }
            ' ' if !in_quotes && !in_single_quotes => {
                if !current.is_empty() {
                    result.push(current.clone());
                    current.clear();
                }
            }
            _ => {
                current.push(c);
            }
        }
    }

    if in_quotes {
        bail!("Mismatched quotes")
    }

    if !current.is_empty() {
        result.push(current);
    }

    Ok(result)
}

async fn wait_port_available(port: u16) {
    let mut count = 0;
    while check_port_in_use(port) {
        debug!("Port {} is still in use, waiting...", port);
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        count += 1;
        if count > 60 {
            panic!("Port {} is still in use after 60 seconds", port);
        }
    }
}

/// check if `port` is available.
fn check_port_in_use(port: u16) -> bool {
    use std::net::TcpStream;
    let in_use = match TcpStream::connect(("0.0.0.0", port)) {
        Ok(_) => true,
        Err(_e) => false,
    };
    in_use
}

fn extract_json(output: &String) -> Result<Value> {
    let lines: Vec<&str> = output.lines().collect();
    let last_line = lines.last().expect("No JSON found in output");
    let json: Value = serde_json::from_str(last_line)?;
    Ok(json)
}

#[tokio::main]
async fn main() {
    World::cucumber()
        .run_and_exit("./features/cmd.feature")
        .await;
}

// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::{bail, Result};
use clap::Parser;
use cucumber::{given, then, World as _};
use jpst::TemplateContext;
use rooch::RoochCli;
use rooch_config::{rooch_config_dir, RoochOpt, ServerOpt};
use rooch_rpc_client::wallet_context::WalletContext;
use rooch_rpc_server::Service;
use serde_json::Value;
use tracing::{debug, info};

#[derive(cucumber::World, Debug, Default)]
struct World {
    service: Option<Service>,
    tpl_ctx: Option<TemplateContext>,
}

#[given(expr = "a server for {word}")] // Cucumber Expression
async fn start_server(w: &mut World, _scenario: String) {
    let mut service = Service::new();
    let opt = RoochOpt::new_with_temp_store();
    wait_port_available(opt.port()).await;

    let server_opt = ServerOpt::new();

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
    debug!("run_cli result: {:?}", ret);
    match ret {
        Ok(output) => {
            let result_json = serde_json::from_str::<Value>(&output);

            if result_json.is_ok() {
                tpl_ctx
                    .entry(cmd_name)
                    .append::<Value>(result_json.unwrap());
            }
        }
        Err(err) => {
            let err_msg = Value::String(err.to_string());
            tpl_ctx.entry(cmd_name).append::<Value>(err_msg);
        }
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

#[tokio::main]
async fn main() {
    World::cucumber()
        .run_and_exit("./tests/features/cmd.feature")
        .await;
}

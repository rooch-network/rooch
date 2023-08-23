// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::{bail, Result};
use clap::Parser;
use cucumber::{given, then, World as _};
use jpst::TemplateContext;
use move_core_types::account_address::AccountAddress;
use rooch::RoochCli;
use rooch_config::rooch_config_dir;
use rooch_rpc_client::wallet_context::WalletContext;
use rooch_rpc_server::Service;
use serde_json::Value;
use tracing::info;

#[derive(cucumber::World, Debug, Default)]
struct World {
    service: Option<Service>,
    tpl_ctx: Option<TemplateContext>,
}

#[given(expr = "a server for {word}")] // Cucumber Expression
async fn start_server(w: &mut World, _scenario: String) {
    let mut service = Service::new();
    service.start(true).await.unwrap();

    w.service = Some(service);
}

#[then(expr = "stop the server")] // Cucumber Expression
async fn stop_server(w: &mut World) {
    println!("stop server");
    match w.service.take() {
        Some(service) => {
            service.stop().unwrap();
            info!("Shutdown Sever");
        }
        None => {
            info!("service is none");
        }
    }
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
}

#[then(regex = r#"cmd: "(.*)?""#)]
async fn run_cmd(world: &mut World, args: String) {
    let config_dir = rooch_config_dir()
        .unwrap()
        .parent()
        .unwrap()
        .join("rooch_test");

    let default = if config_dir.exists() {
        let context = WalletContext::new(Some(config_dir.clone())).await.unwrap();

        match context.config.active_address {
            Some(addr) => AccountAddress::from(addr).to_hex_literal(),
            None => "".to_owned(),
        }
    } else {
        "".to_owned()
    };

    let args = args.replace("{default}", &default);

    if world.tpl_ctx.is_none() {
        world.tpl_ctx = Some(TemplateContext::new());
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
}

#[then(regex = r#"assert: "([^"]*)""#)]
async fn assert_output(world: &mut World, args: String) {
    assert!(world.tpl_ctx.is_some(), "tpl_ctx is none");
    let args = eval_command_args(world.tpl_ctx.as_ref().unwrap(), args);
    let parameters = split_string_with_quotes(&args).expect("Invalid commands");

    for chunk in parameters.chunks(3) {
        let first = chunk.get(0).cloned();
        let op = chunk.get(1).cloned();
        let second = chunk.get(2).cloned();

        info!("assert value: {:?} {:?} {:?}", first, op, second);

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
    // info!("args: {}", args);
    let args = args.replace("\\\"", "\"");
    let eval_args = jpst::format_str!(&args, ctx);
    // info!("eval args:{}", eval_args);
    eval_args
}

/// Split a string into a vector of strings, splitting on spaces, but ignoring spaces inside quotes.
/// And quotes will alse be removed.
fn split_string_with_quotes(s: &str) -> Result<Vec<String>> {
    let mut result = Vec::new();
    let mut chars = s.chars().peekable();
    let mut current = String::new();
    let mut in_quotes = false;

    while let Some(c) = chars.next() {
        match c {
            '"' => {
                in_quotes = !in_quotes;
                // Skip the quote
            }
            ' ' if !in_quotes => {
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

#[tokio::main]
async fn main() {
    World::cucumber()
        .run_and_exit("./features/cmd.feature")
        .await;
}

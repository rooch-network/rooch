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
use regex::Regex;
use rooch::RoochCli;
use rooch_config::{RoochOpt, ServerOpt, ROOCH_CONFIR_DIR};
use rooch_rpc_client::wallet_context::WalletContext;
use rooch_rpc_server::Service;
use rooch_types::crypto::RoochKeyPair;
use serde_json::Value;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use std::{env, path::Path, vec};
use testcontainers::{
    clients::Cli,
    core::{Container, ExecCommand, WaitFor},
    RunnableImage,
};
use tracing::{debug, error, info, trace, warn};
use uuid::Uuid;

const RPC_USER: &str = "roochuser";
const RPC_PASS: &str = "roochpass";
const RPC_PORT: u16 = 18443;
const ORD_RPC_PORT: u16 = 80;

// Default timeout for command execution (30 seconds)
const DEFAULT_COMMAND_TIMEOUT_SECS: u64 = 30;

/// Logging levels for test execution
#[derive(Debug, Clone, PartialEq, PartialOrd)]
enum LogLevel {
    Minimal,    // Only failures and final results
    Normal,     // Current behavior (default)
    Verbose,    // Include template context changes
    Debug,      // Full execution trace
}

impl Default for LogLevel {
    fn default() -> Self {
        LogLevel::Normal
    }
}

impl LogLevel {
    fn from_env() -> Self {
        match env::var("ROOCH_TEST_LOG_LEVEL").unwrap_or_default().to_lowercase().as_str() {
            "minimal" => LogLevel::Minimal,
            "normal" => LogLevel::Normal,
            "verbose" => LogLevel::Verbose,
            "debug" => LogLevel::Debug,
            _ => LogLevel::Normal,
        }
    }
}

/// Result of a command execution
#[derive(Debug, Clone)]
enum CommandResult {
    Success(String),
    Failure { 
        error: String, 
        exit_code: Option<i32>,
        template_vars_used: Vec<String>,
    },
}

/// Information about a single command execution
#[derive(Debug, Clone)]
struct CommandExecution {
    command: String,
    timestamp: Instant,
    result: CommandResult,
    template_vars_used: Vec<String>,
}

/// Context for tracking test execution state
#[derive(Debug)]
struct TestExecutionContext {
    current_step: String,
    step_number: usize,
    total_steps: usize,
    scenario_name: String,
    command_history: Vec<CommandExecution>,
    log_level: LogLevel,
    template_debug_enabled: bool,
    show_progress: bool,
    command_timeout: Duration,
}

impl Default for TestExecutionContext {
    fn default() -> Self {
        Self {
            current_step: String::new(),
            step_number: 0,
            total_steps: 0,
            scenario_name: String::new(),
            command_history: Vec::new(),
            log_level: LogLevel::from_env(),
            template_debug_enabled: env::var("ROOCH_TEST_TEMPLATE_DEBUG").unwrap_or_default() == "true",
            show_progress: env::var("ROOCH_TEST_SHOW_PROGRESS").unwrap_or_default() == "true",
            command_timeout: Duration::from_secs(
                env::var("ROOCH_TEST_TIMEOUT")
                    .unwrap_or_default()
                    .parse()
                    .unwrap_or(DEFAULT_COMMAND_TIMEOUT_SECS)
            ),
        }
    }
}

impl TestExecutionContext {
    /// Display test progress if enabled
    fn display_progress(&self) {
        if self.show_progress && self.log_level >= LogLevel::Normal {
            println!("🧪 [{}/{}] {} - {}", 
                self.step_number, 
                self.total_steps, 
                self.scenario_name,
                self.current_step
            );
        }
    }

    /// Display command execution result
    fn display_command_result(&self, cmd: &str, result: &CommandResult) {
        if self.log_level >= LogLevel::Normal {
            match result {
                CommandResult::Success(_) => {
                    if self.show_progress {
                        println!("  ✅ {}", cmd);
                    }
                }
                CommandResult::Failure { error, template_vars_used, .. } => {
                    eprintln!("  ❌ {}", cmd);
                    eprintln!("     Error: {}", error);
                    if self.log_level >= LogLevel::Verbose && !template_vars_used.is_empty() {
                        eprintln!("     Template vars used: {:?}", template_vars_used);
                    }
                }
            }
        }
    }

    /// Add a command execution to history
    fn add_command_execution(&mut self, command_execution: CommandExecution) {
        self.display_command_result(&command_execution.command, &command_execution.result);
        self.command_history.push(command_execution);
    }
}

/// Template debugging information
#[derive(Debug)]
struct TemplateDebugInfo {
    available_vars: Vec<String>,
    used_vars: Vec<String>,
    resolution_steps: Vec<String>,
    final_value: String,
    original_expression: String,
}

/// Enhanced template context with debugging capabilities
trait TemplateContextExt {
    fn debug_resolve(&self, expression: &str) -> TemplateDebugInfo;
    fn available_keys(&self) -> Vec<String>;
    fn snapshot(&self) -> HashMap<String, Value>;
}

impl TemplateContextExt for TemplateContext {
    fn debug_resolve(&self, expression: &str) -> TemplateDebugInfo {
        let available_vars = self.available_keys();
        let mut used_vars = Vec::new();
        let mut resolution_steps = Vec::new();
        
        // Extract template variables from the expression
        let template_var_pattern = r"\{\{\s*([^}]+)\s*\}\}";
        if let Ok(template_var_regex) = Regex::new(template_var_pattern) {
            for cap in template_var_regex.captures_iter(&expression) {
                if let Some(var_match) = cap.get(1) {
                    let var_name = var_match.as_str().trim();
                    used_vars.push(var_name.to_string());
                    resolution_steps.push(format!("Found template variable: {}", var_name));
                }
            }
        } else {
            resolution_steps.push("Failed to compile regex for template variable extraction".to_string());
        }
        
        // Attempt to resolve the template
        let final_value = jpst::format_str!(&expression, self);
        resolution_steps.push("Template resolution completed".to_string());
        
        TemplateDebugInfo {
            available_vars,
            used_vars,
            resolution_steps,
            final_value,
            original_expression: expression.to_string(),
        }
    }
    
    fn available_keys(&self) -> Vec<String> {
        let value = self.as_value();
        if let Value::Object(map) = &value {
            map.keys().cloned().collect()
        } else {
            Vec::new()
        }
    }
    
    fn snapshot(&self) -> HashMap<String, Value> {
        let value = self.as_value();
        if let Value::Object(map) = value {
            map.into_iter().collect()
        } else {
            HashMap::new()
        }
    }
}

#[derive(cucumber::World, Debug)]
struct World {
    opt: RoochOpt,
    docker: Cli,
    container_network: String,
    service: Option<Service>,
    bitcoind: Option<Container<BitcoinD>>,
    ord: Option<Container<Ord>>,
    tpl_ctx: Option<TemplateContext>,
    execution_ctx: Option<TestExecutionContext>,
}

impl Default for World {
    fn default() -> Self {
        let network_uuid = Uuid::new_v4();

        World {
            opt: RoochOpt::new_with_temp_store().expect("new rooch opt should be ok"),
            docker: Cli::default(),
            container_network: format!("test_network_{}", network_uuid),
            service: None,
            bitcoind: None,
            ord: None,
            tpl_ctx: None,
            execution_ctx: Some(TestExecutionContext::default()),
        }
    }
}

#[given(expr = "a server for {word}")] // Cucumber Expression
async fn start_server(w: &mut World, _scenario: String) {
    tokio::time::sleep(Duration::from_secs(5)).await;
    let mut service = Service::new();
    wait_port_available(w.opt.port()).await;

    match w.bitcoind.take() {
        Some(bitcoind) => {
            let bitcoin_rpc_url =
                format!("http://127.0.0.1:{}", bitcoind.get_host_port_ipv4(RPC_PORT));
            w.opt.btc_rpc_url = Some(bitcoin_rpc_url);
            w.opt.btc_rpc_username = Some(RPC_USER.to_string());
            w.opt.btc_rpc_password = Some(RPC_PASS.to_string());
            w.opt.btc_sync_block_interval = Some(1u64); // Update sync interval as 1s

            info!("config btc rpc ok");

            w.bitcoind = Some(bitcoind);
        }
        None => {
            info!("bitcoind server is none");
        }
    }
    w.opt.traffic_burst_size = Some(5000u32);
    w.opt.traffic_per_second = Some(0.001f64);

    let mut server_opt = ServerOpt::new();
    //TODO we should load keypair from cli config
    let kp: RoochKeyPair = RoochKeyPair::generate_secp256k1();
    server_opt.sequencer_keypair = Some(kp.copy());
    server_opt.proposer_keypair = Some(kp.copy());

    service.start(w.opt.clone(), server_opt).await.unwrap();

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
    let config_dir = world.opt.base().base_data_dir().join(ROOCH_CONFIR_DIR);
    debug!("config_dir: {:?}", config_dir);
    if world.tpl_ctx.is_none() {
        let mut tpl_ctx = TemplateContext::new();
        if !config_dir.exists() {
            run_cli_cmd(
                config_dir.as_path(),
                &mut tpl_ctx,
                "init --skip-password"
                    .split_whitespace()
                    .map(|s| s.to_owned())
                    .collect(),
            )
            .await;
            run_cli_cmd(
                config_dir.as_path(),
                &mut tpl_ctx,
                "env switch --alias local"
                    .split_whitespace()
                    .map(|s| s.to_owned())
                    .collect(),
            )
            .await;
        }

        let context = WalletContext::new(Some(config_dir.clone())).unwrap();
        let address_mapping = serde_json::Value::Object(
            context
                .address_mapping
                .iter()
                .map(|(k, v)| (k.to_string(), Value::String(v.to_hex_literal())))
                .collect(),
        );
        tpl_ctx.entry("address_mapping").set(address_mapping);

        world.tpl_ctx = Some(tpl_ctx);
    }
    let tpl_ctx = world.tpl_ctx.as_mut().unwrap();
    let args = eval_command_args(tpl_ctx, args);

    let args = split_string_with_quotes(&args).expect("Invalid commands");
    run_cli_cmd(config_dir.as_path(), tpl_ctx, args).await;
}

async fn run_cli_cmd(config_dir: &Path, tpl_ctx: &mut TemplateContext, mut args: Vec<String>) {
    let cmd_name = args[0].clone();
    let full_command = args.join(" ");
    
    args.insert(0, "rooch".to_owned());
    args.push("--config-dir".to_owned());
    args.push(config_dir.to_str().unwrap().to_string());
    
    // Extract template variables used in this command
    let template_vars_used = extract_template_vars(&full_command);
    
    let opts: RoochCli = RoochCli::parse_from(args);
    let start_time = Instant::now();
    let ret = rooch::run_cli(opts).await;

    let command_execution = CommandExecution {
        command: full_command.clone(),
        timestamp: start_time,
        result: match ret {
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
                CommandResult::Success(output)
            }
            Err(err) => {
                let err_msg = Value::String(err.to_string());
                error!("run_cli cmd: {} fail: {:?}", cmd_name, &err_msg);
                
                // Enhanced error reporting
                eprintln!("❌ Command failed: {}", full_command);
                eprintln!("   Error: {}", err);
                if !template_vars_used.is_empty() {
                    eprintln!("   Template vars used: {:?}", template_vars_used);
                    eprintln!("   Available context keys: {:?}", tpl_ctx.available_keys());
                }
                
                tpl_ctx.entry(cmd_name).append::<Value>(err_msg);
                CommandResult::Failure {
                    error: err.to_string(),
                    exit_code: None,
                    template_vars_used: template_vars_used.clone(),
                }
            }
        },
        template_vars_used,
    };

    // For now, we'll just log the command execution since we don't have access to execution context here
    // This will be enhanced when we refactor to pass execution context through
    trace!("Command execution: {:?}", command_execution);
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

    info!(
        "run cmd: bitseed {} ,stdout: {}",
        joined_args, stdout_string
    );

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
        format!("--bitcoin-rpc-url=http://bitcoind:{}", RPC_PORT),
        format!("--bitcoin-rpc-username={}", RPC_USER),
        format!("--bitcoin-rpc-password={}", RPC_PASS),
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
        cmd: joined_args.clone(),
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

    info!("run cmd: {} ,stdout: {}", joined_args, stdout_string);

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
        cmd: joined_args.clone(),
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

    debug!("run cmd: {}, stdout: {}", joined_args, stdout_string);

    // Check if stderr_string is not empty and panic if it contains any content.
    if !stderr_string.is_empty() {
        panic!("Command execution failed with errors: {}", stderr_string);
    }

    let result_json = serde_json::from_str::<Value>(&stdout_string);
    if let Ok(json_value) = result_json {
        debug!("cmd bitcoincli: {} output: {}", cmd_name, json_value);
        tpl_ctx.entry(cmd_name).append::<Value>(json_value);
    } else {
        debug!("result_json not ok, output as string");
        tpl_ctx
            .entry(cmd_name)
            .append::<Value>(Value::String(stdout_string.trim().to_string()));
    }

    debug!("current tpl_ctx: {:?}", tpl_ctx);
}

#[then(regex = r#"assert: "([^"]*)""#)]
async fn assert_output(world: &mut World, orginal_args: String) {
    assert!(world.tpl_ctx.is_some(), "tpl_ctx is none");
    assert!(orginal_args.len() > 0, "assert args is empty");
    
    let tpl_ctx = world.tpl_ctx.as_ref().unwrap();
    
    // Enhanced template debugging
    let template_debug = tpl_ctx.debug_resolve(&orginal_args);
    
    if template_debug.final_value.is_empty() && !orginal_args.is_empty() {
        eprintln!("❌ Template resolution failed:");
        eprintln!("   Expression: {}", orginal_args);
        eprintln!("   Available vars: {:?}", template_debug.available_vars);
        eprintln!("   Used vars: {:?}", template_debug.used_vars);
        eprintln!("   Resolution steps: {:#?}", template_debug.resolution_steps);
        panic!("Template variable resolution failed for: {}", orginal_args);
    }
    
    let args = template_debug.final_value;
    let splited_args = split_string_with_quotes(&args).unwrap_or_else(|e| {
        eprintln!("❌ Failed to parse assertion arguments:");
        eprintln!("   Original: {}", orginal_args);
        eprintln!("   After template resolution: {}", args);
        eprintln!("   Parse error: {}", e);
        panic!("Invalid assertion commands: {}", e);
    });
    
    debug!(
        "originl args: {}\n after eval: {}\n after split: {:?}",
        orginal_args, args, splited_args
    );
    
    if splited_args.is_empty() {
        eprintln!("❌ Empty assertion arguments:");
        eprintln!("   Original: {}", orginal_args);
        eprintln!("   After template resolution: {}", args);
        eprintln!("   Template vars used: {:?}", template_debug.used_vars);
        panic!("splited_args should not empty, the orginal_args:{}", orginal_args);
    }
    
    for chunk in splited_args.chunks(3) {
        let first = chunk.get(0).cloned();
        let op = chunk.get(1).cloned();
        let second = chunk.get(2).cloned();

        debug!("assert value: {:?} {:?} {:?}", first, op, second);

        match (first, op, second) {
            (Some(first), Some(op), Some(second)) => {
                let assertion_passed = match op.as_str() {
                    "==" => {
                        let passed = first == second;
                        if !passed {
                            eprintln!("❌ Assertion failed:");
                            eprintln!("   Expression: {} {} {}", first, op, second);
                            eprintln!("   Expected: {}", second);
                            eprintln!("   Actual: {}", first);
                            eprintln!("   Operator: {}", op);
                            eprintln!("   Template vars used: {:?}", template_debug.used_vars);
                        }
                        passed
                    },
                    "!=" => {
                        let passed = first != second;
                        if !passed {
                            eprintln!("❌ Assertion failed:");
                            eprintln!("   Expression: {} {} {}", first, op, second);
                            eprintln!("   Expected: NOT {}", second);
                            eprintln!("   Actual: {}", first);
                            eprintln!("   Operator: {}", op);
                            eprintln!("   Template vars used: {:?}", template_debug.used_vars);
                        }
                        passed
                    },
                    "contains" => {
                        let passed = first.contains(&second);
                        if !passed {
                            eprintln!("❌ Assertion failed:");
                            eprintln!("   Expression: {} {} {}", first, op, second);
                            eprintln!("   Expected: '{}' to contain '{}'", first, second);
                            eprintln!("   Operator: {}", op);
                            eprintln!("   Template vars used: {:?}", template_debug.used_vars);
                        }
                        passed
                    },
                    "not_contains" => {
                        let passed = !first.contains(&second) && !first.to_lowercase().contains(&second.to_lowercase());
                        if !passed {
                            eprintln!("❌ Assertion failed:");
                            eprintln!("   Expression: {} {} {}", first, op, second);
                            eprintln!("   Expected: '{}' to NOT contain '{}'", first, second);
                            eprintln!("   Operator: {}", op);
                            eprintln!("   Template vars used: {:?}", template_debug.used_vars);
                        }
                        passed
                    },
                    ">" => {
                        let passed = compare_numbers(&first, &second, |ord| ord == std::cmp::Ordering::Greater);
                        if !passed {
                            eprintln!("❌ Assertion failed:");
                            eprintln!("   Expression: {} {} {}", first, op, second);
                            eprintln!("   Expected: {} > {}", first, second);
                            eprintln!("   Operator: {}", op);
                            eprintln!("   Template vars used: {:?}", template_debug.used_vars);
                        }
                        passed
                    },
                    "<" => {
                        let passed = compare_numbers(&first, &second, |ord| ord == std::cmp::Ordering::Less);
                        if !passed {
                            eprintln!("❌ Assertion failed:");
                            eprintln!("   Expression: {} {} {}", first, op, second);
                            eprintln!("   Expected: {} < {}", first, second);
                            eprintln!("   Operator: {}", op);
                            eprintln!("   Template vars used: {:?}", template_debug.used_vars);
                        }
                        passed
                    },
                    ">=" => {
                        let passed = compare_numbers(&first, &second, |ord| ord != std::cmp::Ordering::Less);
                        if !passed {
                            eprintln!("❌ Assertion failed:");
                            eprintln!("   Expression: {} {} {}", first, op, second);
                            eprintln!("   Expected: {} >= {}", first, second);
                            eprintln!("   Operator: {}", op);
                            eprintln!("   Template vars used: {:?}", template_debug.used_vars);
                        }
                        passed
                    },
                    "<=" => {
                        let passed = compare_numbers(&first, &second, |ord| ord != std::cmp::Ordering::Greater);
                        if !passed {
                            eprintln!("❌ Assertion failed:");
                            eprintln!("   Expression: {} {} {}", first, op, second);
                            eprintln!("   Expected: {} <= {}", first, second);
                            eprintln!("   Operator: {}", op);
                            eprintln!("   Template vars used: {:?}", template_debug.used_vars);
                        }
                        passed
                    },
                    _ => {
                        eprintln!("❌ Unsupported operator: {}", op);
                        eprintln!("   Supported operators: ==, !=, contains, not_contains, >, <, >=, <=");
                        panic!("unsupported operator {:?}", op.as_str());
                    }
                };
                
                assert!(assertion_passed, "Assertion failed: {} {} {}", first, op, second);
            },
            _ => {
                eprintln!("❌ Invalid assertion format:");
                eprintln!("   Original: {}", orginal_args);
                eprintln!("   After parsing: {:?}", args);
                eprintln!("   Expected format: 'value operator value'");
                eprintln!("   Examples: 'foo == bar', '{{$.cmd[-1].status}} == success'");
                panic!("expected 3 arguments: first [==|!=] second, but got input {:?}", args);
            }
        }
    }
    
    if world.execution_ctx.as_ref().map_or(false, |ctx| ctx.show_progress) {
        println!("✅ Assertion passed: {}", orginal_args);
    }
    info!("assert ok!");
}

fn eval_command_args(ctx: &TemplateContext, args: String) -> String {
    let eval_args = jpst::format_str!(&args, ctx);
    eval_args
}

/// Extract template variable names from a command string
fn extract_template_vars(command: &str) -> Vec<String> {
    let template_var_pattern = r"\{\{\s*([^}]+)\s*\}\}";
    if let Ok(template_var_regex) = Regex::new(template_var_pattern) {
        template_var_regex
            .captures_iter(command)
            .filter_map(|cap| cap.get(1).map(|m| m.as_str().trim().to_string()))
            .collect()
    } else {
        Vec::new()
    }
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

/// Compare two number strings with high precision
/// Supports u128, i128, and f64 comparisons without precision loss
fn compare_numbers<F>(first: &str, second: &str, op: F) -> bool
where
    F: Fn(std::cmp::Ordering) -> bool,
{
    // Try to parse as u128 first (for blockchain amounts)
    if let (Ok(first_u128), Ok(second_u128)) = (first.parse::<u128>(), second.parse::<u128>()) {
        return op(first_u128.cmp(&second_u128));
    }

    // Try to parse as i128 (for signed integers)
    if let (Ok(first_i128), Ok(second_i128)) = (first.parse::<i128>(), second.parse::<i128>()) {
        return op(first_i128.cmp(&second_i128));
    }

    // Fall back to f64 parsing for floating point numbers
    match (first.parse::<f64>(), second.parse::<f64>()) {
        (Ok(first_f64), Ok(second_f64)) => {
            // Use partial_cmp for f64 and handle NaN cases
            match first_f64.partial_cmp(&second_f64) {
                Some(ordering) => op(ordering),
                None => panic!(
                    "Cannot compare {:?} and {:?} - one or both values are NaN",
                    first, second
                ),
            }
        }
        _ => panic!(
            "Cannot parse {:?} or {:?} as numbers for comparison",
            first, second
        ),
    }
}

#[tokio::main]
async fn main() {
    World::cucumber().run_and_exit("./features/").await;
}

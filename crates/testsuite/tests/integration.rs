use std::path::PathBuf;

use anyhow::{bail, Result};
use cucumber::{given, then, World as _};
use futures::FutureExt as _;
use rooch_server::Service;
use std::future;
use tracing::info;

fn rooch_root() -> Result<PathBuf> {
    let curr_dir = std::env::current_dir()?;
    let parent = curr_dir.parent();
    let rooch_root = parent.and_then(|p| p.parent());
    match rooch_root {
        Some(rooch_root) => Ok(rooch_root.to_path_buf()),
        None => bail!("rooch root not found"),
    }
}

#[derive(cucumber::World, Debug, Default)]
struct World {
    service: Option<Service>,
}

#[given(expr = "a server")] // Cucumber Expression
async fn start_server(w: &mut World) {
    let rooch_root = rooch_root().expect("rooch root not found");
    let config_dir = rooch_root.to_path_buf().join("fixtures/config.yml");
    std::env::set_var(
        "ROOCH_CONFIG",
        config_dir.to_str().expect("unexpected path error"),
    );
    let mut service = Service::new();
    service.start().await.unwrap();

    w.service = Some(service);
}

#[then(regex = r#"cmd: "(.*)?""#)]
async fn run_cmd(_w: &mut World, args: String) {
    let rooch_root = rooch_root().unwrap();
    let config_dir = rooch_root.to_path_buf().join("fixtures/config.yml");
    let mut cmd = assert_cmd::Command::cargo_bin("rooch").unwrap();
    cmd.env("ROOCH_CONFIG", config_dir.to_str().unwrap());

    let parameters = args.split_whitespace();
    for parameter in parameters {
        cmd.arg(parameter.to_owned());
    }
    let assert = cmd.assert().success();
}

#[then(regex = r#"assert: "([^"]*)""#)]
async fn assert_output(_w: &mut World, args: String) {
    let parameters = args.split_whitespace().collect::<Vec<_>>();

    for chunk in parameters.chunks(3) {
        let first = chunk.get(0).cloned();
        let op = chunk.get(1).cloned();
        let second = chunk.get(2).cloned();

        info!("assert value: {:?} {:?} {:?}", first, op, second);

        match (first, op, second) {
            (Some(first), Some(op), Some(second)) => match op {
                "==" => assert_eq!(first, second),
                "!=" => assert_ne!(first, second),
                _ => panic!("unsupported operator"),
            },
            _ => panic!("expected 3 arguments: first [==|!=] second"),
        }
    }
    info!("assert ok!");
}

#[tokio::main]
async fn main() {
    World::cucumber()
        .after(move |_feature, _rule, _scenario, _ev, world| {
            if let Some(service) = &world.unwrap().service {
                // TODO: sender signal to stop server
                service.stop().expect("failed to stop server");
            };
            future::ready(()).boxed()
        })
        .run_and_exit("./features/cmd.feature")
        .await;
}

use cucumber::{given, then, World as _};
use rooch_server::Service;
use tracing::info;

#[derive(cucumber::World, Debug, Default)]
struct World {
    service: Option<Service>,
}

#[given(expr = "a server")] // Cucumber Expression
async fn start_server(w: &mut World) {
    let mut service = Service::new();
    service.start().await.unwrap();

    w.service = Some(service);
}

#[then(regex = r#"cmd: "(.*)?""#)]
async fn run_cmd(_: &mut World, args: String) {
    let mut cmd = assert_cmd::Command::cargo_bin("rooch").unwrap();
    // Discard test data
    cmd.env("TEST_ENV", "true");

    if args.contains("\"") {
        let mut args1 = args.clone();
        let start = args.find("\"").unwrap();
        let commond: String = args1.drain(0..start - 1).collect();
        let args_a: String = args1.drain(2..args1.len() - 1).collect();

        cmd.args(commond.split_whitespace());
        cmd.arg(args_a);
    } else {
        cmd.args(args.split_whitespace());
    }
    let _assert = cmd.assert().success();
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
        .run_and_exit("./features/cmd.feature")
        .await;
}

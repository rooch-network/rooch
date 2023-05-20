use cucumber::{given, then, World as _};
use jpst::TemplateContext;
use rooch_server::Service;
use serde_json::Value;
use tracing::info;

#[derive(cucumber::World, Debug, Default)]
struct World {
    service: Option<Service>,
    tpl_ctx: Option<TemplateContext>,
}

#[given(expr = "a server")] // Cucumber Expression
async fn start_server(w: &mut World) {
    let mut service = Service::new();
    service.start().await.unwrap();

    w.service = Some(service);
}

#[then(regex = r#"cmd: "(.*)?""#)]
async fn run_cmd(world: &mut World, args: String) {
    let mut cmd = assert_cmd::Command::cargo_bin("rooch").unwrap();
    // Discard test data
    cmd.env("TEST_ENV", "true");

    if world.tpl_ctx.is_none() {
        world.tpl_ctx = Some(TemplateContext::new());
    }
    let tpl_ctx = world.tpl_ctx.as_mut().unwrap();
    let args = eval_command_args(tpl_ctx, args);

    let cmd_name;
    if args.contains("\"") {
        let mut args1 = args.clone();
        let start = args.find("\"").unwrap();
        let command: String = args1.drain(0..start - 1).collect();
        let args_a: String = args1.drain(2..args1.len() - 1).collect();

        let command = command.split_whitespace().collect::<Vec<_>>();
        cmd_name = command[0].to_string();
        cmd.args(command);
        cmd.arg(args_a);
    } else {
        let args = args.split_whitespace().collect::<Vec<_>>();
        cmd_name = args[0].to_string();
        cmd.args(args);
    }
    let assert = cmd.assert().success();

    let output = assert.get_output();
    let result_json: Value = serde_json::from_slice(&output.stdout).unwrap();
    tpl_ctx.entry(cmd_name).append(result_json);
}

#[then(regex = r#"assert: "([^"]*)""#)]
async fn assert_output(world: &mut World, args: String) {
    assert!(world.tpl_ctx.is_some(), "tpl_ctx is none");
    let args = eval_command_args(world.tpl_ctx.as_ref().unwrap(), args);
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

fn eval_command_args(ctx: &TemplateContext, args: String) -> String {
    info!("args: {}", args);
    let args = args.replace("\\\"", "\"");
    let eval_args = jpst::format_str!(&args, ctx);
    info!("eval args:{}", eval_args);
    eval_args
}

#[tokio::main]
async fn main() {
    World::cucumber()
        .run_and_exit("./features/cmd.feature")
        .await;
}

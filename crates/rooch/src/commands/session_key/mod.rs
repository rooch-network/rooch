// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::CommandAction;
use async_trait::async_trait;
use clap::Parser;
use commands::create::CreateCommand;
use commands::list::ListCommand;
use rooch_types::error::RoochResult;
use serde_json::Value;
use tabled::{builder::Builder, settings::Style};

pub mod commands;

#[derive(Parser)]
pub struct SessionKey {
    #[clap(subcommand)]
    cmd: SessionKeyCommand,
}

#[async_trait]
impl CommandAction<String> for SessionKey {
    async fn execute(self) -> RoochResult<String> {
        match self.cmd {
            SessionKeyCommand::Create(create) => create.execute().await.map(|resp| {
                serde_json::to_string_pretty(&resp).expect("Failed to serialize response")
            }),
            SessionKeyCommand::List(list) => {
                let display_as_table = list.table;
                let json_output = list.execute_serialized().await?;
                let json_value: Value =
                    serde_json::from_str(&json_output).expect("Failed to parse JSON");

                if display_as_table {
                    display_json_as_table(&json_value);
                    Ok(String::new())
                } else {
                    Ok(json_output)
                }
            }
        }
    }
}

#[derive(clap::Subcommand)]
#[clap(name = "session_key")]
pub enum SessionKeyCommand {
    Create(Box<CreateCommand>),
    List(ListCommand),
}

fn display_json_as_table(data: &Value) {
    if let Some(array) = data.as_array() {
        for item in array {
            let mut main_table = Builder::default();
            main_table.push_record(["Field", "Value"]);

            if let Some(name) = item.get("name").and_then(|v| v.as_str()) {
                main_table.push_record(["Name", name]);
            }
            if let Some(abilities) = item.get("value").and_then(|v| v.get("abilities")) {
                main_table.push_record(["Abilities", &abilities.to_string()]);
            }
            if let Some(type_str) = item
                .get("value")
                .and_then(|v| v.get("type"))
                .and_then(|v| v.as_str())
            {
                main_table.push_record(["Type", type_str]);
            }

            let mut details_table = Builder::default();
            details_table.push_record(["Detail", "Value"]);
            if let Some(details) = item.get("value").and_then(|v| v.get("value")) {
                if let Some(app_name) = details.get("app_name").and_then(|v| v.as_str()) {
                    details_table.push_record(["App Name", app_name]);
                }
                if let Some(app_url) = details.get("app_url").and_then(|v| v.as_str()) {
                    details_table.push_record(["App URL", app_url]);
                }
                if let Some(auth_key) = details.get("authentication_key").and_then(|v| v.as_str()) {
                    details_table.push_record(["Authentication Key", auth_key]);
                }
                if let Some(create_time) = details.get("create_time") {
                    details_table.push_record(["Create Time", &create_time.to_string()]);
                }
                if let Some(last_active) = details.get("last_active_time") {
                    details_table.push_record(["Last Active Time", &last_active.to_string()]);
                }
                if let Some(max_interval) = details.get("max_inactive_interval") {
                    details_table.push_record(["Max Inactive Interval", &max_interval.to_string()]);
                }
            }

            main_table.push_record([
                "Details",
                &format!("{}", details_table.build().with(Style::rounded())),
            ]);

            if let Some(scopes) = item
                .get("value")
                .and_then(|v| v.get("value"))
                .and_then(|v| v.get("scopes"))
                .and_then(|v| v.as_array())
            {
                let mut scopes_table = Builder::default();
                scopes_table.push_record([
                    "Abilities",
                    "Type",
                    "Function Name",
                    "Module Address",
                    "Module Name",
                ]);

                for scope in scopes {
                    scopes_table.push_record([
                        &scope
                            .get("abilities")
                            .map_or(String::new(), |v| v.to_string()),
                        scope.get("type").and_then(|v| v.as_str()).unwrap_or(""),
                        scope
                            .get("value")
                            .and_then(|v| v.get("function_name"))
                            .and_then(|v| v.as_str())
                            .unwrap_or(""),
                        scope
                            .get("value")
                            .and_then(|v| v.get("module_address"))
                            .and_then(|v| v.as_str())
                            .unwrap_or(""),
                        scope
                            .get("value")
                            .and_then(|v| v.get("module_name"))
                            .and_then(|v| v.as_str())
                            .unwrap_or(""),
                    ]);
                }

                main_table.push_record([
                    "Scopes",
                    &format!("{}", scopes_table.build().with(Style::rounded())),
                ]);
            }

            println!("{}", main_table.build().with(Style::rounded()));
        }
    }
}

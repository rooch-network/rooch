// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use serde_json::Value;
use tabled::{
    settings::{object::Rows, Disable, Panel, Style},
    Table, Tabled,
};

// Define the `json_to_table` function to be used by external callers
pub fn json_to_table(json_value: Value) {
    json_to_table_display(&json_value);
}

// Struct for a single-row table to display each line of content
#[derive(Tabled)]
struct TableRow {
    line: String,
}

// Wrap long strings to specified width
fn wrap_long_text(text: &str, width: usize) -> String {
    text.chars()
        .collect::<Vec<_>>()
        .chunks(width)
        .map(|chunk| chunk.iter().collect::<String>())
        .collect::<Vec<_>>()
        .join("\n")
}

// Display JSON object as a table
fn display_json(value: &Value, title: &str) {
    let mut rows = Vec::new();

    if let Some(obj) = value.as_object() {
        for (k, v) in obj {
            if k == "tx_order_signature" && v.is_string() {
                rows.push(TableRow {
                    line: format!("{}: \n{}", k, wrap_long_text(v.as_str().unwrap(), 96)),
                });
            } else if k == "status" && v.is_object() {
                rows.push(TableRow {
                    line: format!("[{}]", k),
                });
                rows.extend(format_nested_json(v));
            } else if v.is_object() || v.is_array() {
                rows.push(TableRow {
                    line: format!("{}: [complex data]", k),
                });
            } else {
                rows.push(TableRow {
                    line: format!("{}: {}", k, v),
                });
            }
        }
    }

    let main_table = Table::new(&rows)
        .with(Style::rounded())
        .with(Panel::header(title))
        .with(Disable::row(Rows::single(1)))
        .to_string();

    println!("{}", main_table);
}

// Display changeset table, with dynamic field extraction and nested field handling
fn display_changes(changeset: &Value) {
    let mut change_rows = Vec::new();

    if let Some(changes) = changeset.get("changes").and_then(|v| v.as_array()) {
        for change in changes {
            if let Some(obj) = change.as_object() {
                for (key, value) in obj {
                    if key == "metadata" || key == "value" {
                        change_rows.push(TableRow {
                            line: format!("[{}]", key),
                        });
                        change_rows.extend(format_nested_json(value));
                    }
                }
            }
            change_rows.push(TableRow {
                line: "─────────────────────".to_string(),
            });
        }
    }

    for (key, value) in changeset.as_object().unwrap() {
        if key != "changes" && !value.is_object() && !value.is_array() {
            let line = format!("{}: {}", key, value);
            change_rows.push(TableRow { line });
        }
    }

    let changes_table = Table::new(&change_rows)
        .with(Style::rounded())
        .with(Panel::header("Changeset"))
        .with(Disable::row(Rows::single(1)))
        .to_string();

    println!("{}", changes_table);
}

// Recursively parse nested JSON items, formatting for multi-line display
fn format_nested_json(value: &Value) -> Vec<TableRow> {
    let mut rows = Vec::new();
    if let Some(obj) = value.as_object() {
        for (k, v) in obj {
            if v.is_object() || v.is_array() {
                rows.push(TableRow {
                    line: format!("[{}]", k),
                });
                rows.extend(format_nested_json(v));
            } else {
                let line = format!("{}: {}", k, v);
                rows.push(TableRow { line });
            }
        }
    }
    rows
}

// Display events table, with dynamic field extraction and nested field handling
fn display_events(events: &Value) {
    let mut event_rows = Vec::new();

    for event in events.as_array().unwrap() {
        if let Some(obj) = event.as_object() {
            for (key, value) in obj {
                if key == "event_data" {
                    continue;
                }

                if value.is_object() {
                    event_rows.push(TableRow {
                        line: format!("[{}]", key),
                    });
                    event_rows.extend(format_nested_json(value));
                } else {
                    event_rows.push(TableRow {
                        line: format!("{}: {}", key, value),
                    });
                }
            }
        }
        event_rows.push(TableRow {
            line: "─────────────────────".to_string(),
        });
    }

    let events_table = Table::new(&event_rows)
        .with(Style::rounded())
        .with(Panel::header("Events"))
        .with(Disable::row(Rows::single(1)))
        .to_string();

    println!("{}", events_table);
}

// Main parsing function for handling JSON data
pub fn json_to_table_display(value: &Value) {
    let mut sequence_info_value = None;
    let mut unknown_data = Vec::new();

    if let Some(obj) = value.as_object() {
        for (key, val) in obj {
            match key.as_str() {
                "sequence_info" => sequence_info_value = Some(val.clone()),
                "execution_info" => display_json(val, "Execution Info"),
                "output" => {
                    if let Some(changeset) = val.get("changeset") {
                        display_changes(changeset);
                    }
                    if let Some(events) = val.get("events") {
                        display_events(events);
                    }
                    display_json(val, "Output Status");
                }
                "error_info" => {
                    if !val.is_null() {
                        display_json(val, "Error Info");
                    }
                }
                _ => unknown_data.push(TableRow {
                    line: format!("{}: {}", key, val),
                }),
            }
        }
    }

    if let Some(sequence_info) = sequence_info_value {
        display_json(&sequence_info, "Sequence Info");
    }

    if !unknown_data.is_empty() {
        let unknown_table = Table::new(&unknown_data).with(Style::rounded()).to_string();
        println!("Unknown Data:\n{}", unknown_table);
    }
}

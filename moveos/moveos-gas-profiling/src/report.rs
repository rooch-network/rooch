// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::log::TransactionGasLog;
use anyhow::Result;
use handlebars::Handlebars;
use move_core_types::gas_algebra::InternalGas;
use serde_json::{json, Map, Value};
use std::fmt::Write;
use std::path::Path;
use std::{fmt, fs};

const TEMPLATE: &str = include_str!("../templates/index.html");

fn ensure_dirs_exist(path: impl AsRef<Path>) -> Result<()> {
    if let Err(err) = fs::create_dir_all(&path) {
        match err.kind() {
            std::io::ErrorKind::AlreadyExists => (),
            _ => return Err(err.into()),
        }
    }
    Ok(())
}

impl TransactionGasLog {
    pub fn generate_html_report(&self, path: impl AsRef<Path>, header: String) -> Result<()> {
        let mut data = Map::new();
        data.insert("title".to_string(), Value::String(header));

        let graph_exec_io = self.exec_io.to_flamegraph("Execution & IO".to_string())?;

        data.insert(
            "graph-exec-io".to_string(),
            Value::Bool(graph_exec_io.is_some()),
        );

        // Aggregate Events
        let aggregated: crate::aggregate::AggregatedExecutionGasEvents =
            self.exec_io.aggregate_gas_events();

        let scaling_factor = u64::from(1u32) as f64;
        let total_exec_io = u64::from(self.exec_io.total) as f64;

        let convert_op = |(op, hits, cost): (String, usize, InternalGas)| {
            let cost_scaled = format!("{:.8}", (u64::from(cost) as f64 / scaling_factor));
            let cost_scaled = crate::misc::strip_trailing_zeros_and_decimal_point(&cost_scaled);

            let percentage = format!("{:.2}%", u64::from(cost) as f64 / total_exec_io * 100.0);

            json!({
                "name": op,
                "hits": hits,
                "cost": cost_scaled,
                "percentage": percentage,
            })
        };
        data.insert(
            "ops".to_string(),
            Value::Array(aggregated.ops.into_iter().map(convert_op).collect()),
        );

        // Execution trace
        let mut tree = self.exec_io.to_erased().tree;
        tree.include_child_costs();

        let mut table = vec![];
        tree.preorder_traversel(|depth, text, &cost| {
            let text_indented = format!("{}{}", " ".repeat(depth * 4), text);

            if cost.is_zero() {
                table.push([text_indented, "".to_string(), "".to_string()])
            } else {
                let cost_scaled = format!("{:.8}", (u64::from(cost) as f64 / scaling_factor));
                let cost_scaled = crate::misc::strip_trailing_zeros_and_decimal_point(&cost_scaled);

                let percentage = format!("{:.2}%", u64::from(cost) as f64 / total_exec_io * 100.0);

                table.push([text_indented, cost_scaled.to_string(), percentage])
            }
        });

        let mut trace = String::new();
        render_table(&mut trace, &table, 4)?;
        data.insert("trace".to_string(), Value::String(trace));

        // Rendering the html doc
        let mut handlebars = Handlebars::new();
        handlebars.register_template_string("index", TEMPLATE)?;
        let html = handlebars.render("index", &data)?;

        // Writing to disk
        let path_root = path.as_ref();

        ensure_dirs_exist(path_root)?;
        let path_assets = path_root.join("assets");
        ensure_dirs_exist(&path_assets)?;

        if let Some(graph_bytes) = graph_exec_io {
            fs::write(path_assets.join("exec_io.svg"), graph_bytes)?;
        }

        fs::write(path_root.join("index.html"), html)?;

        Ok(())
    }
}

fn indent(output: &mut impl Write, count: usize) -> fmt::Result {
    if count == 0 {
        return Ok(());
    }

    write!(output, "{}", " ".repeat(count))
}

fn render_table<R, S>(output: &mut impl Write, table: &[R], spacing: usize) -> fmt::Result
where
    R: AsRef<[S]>,
    S: AsRef<str>,
{
    let n_rows = table.len();
    assert!(n_rows >= 1, "there must be at least 1 row");

    let n_cols = table[0].as_ref().len();
    assert!(n_cols >= 1, "there must be at least 1 col");
    assert!(
        table.iter().skip(1).all(|row| row.as_ref().len() == n_cols),
        "mismatching row widths"
    );

    let text = |row: usize, col: usize| -> &str { table[row].as_ref()[col].as_ref() };

    let col_widths = (0..(n_cols - 1))
        .map(|col| (0..n_rows).map(|row| text(row, col).len()).max().unwrap())
        .collect::<Vec<_>>();

    #[allow(clippy::needless_range_loop)]
    for row in 0..n_rows {
        for col in 0..n_cols {
            if col > 0 {
                indent(output, spacing)?;
            }

            let t = text(row, col);
            write!(output, "{}", t)?;

            if col + 1 < n_cols {
                indent(output, col_widths[col] - t.len())?;
            }
        }
        writeln!(output)?;
    }

    Ok(())
}

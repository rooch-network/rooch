// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use clap::{Args, Parser};
use move_cli::Move;
use move_command_line_common::address::NumericalAddress;
use move_command_line_common::files::{extension_equals, find_filenames, MOVE_EXTENSION};
use move_command_line_common::parser::NumberFormat;
use move_command_line_common::testing::UPDATE_BASELINE;
use move_compiler::command_line::compiler::construct_pre_compiled_lib;
use move_compiler::shared::PackagePaths;
use move_compiler::FullyCompiledProgram;
use move_core_types::account_address::AccountAddress;
use move_package::source_package::layout::SourcePackageLayout;
use move_stdlib::path_in_crate;
use moveos_types::addresses::MOVEOS_NAMED_ADDRESS_MAPPING;
use once_cell::sync::Lazy;
use rooch_integration_test_runner;
use std::collections::BTreeMap;
use std::fmt::Display;
use std::num::NonZeroUsize;
use std::str::FromStr;
use std::sync::Mutex;

pub const INTEGRATION_TESTS_DIR: &str = "integration-tests";

static G_PRE_COMPILED_LIB: Lazy<Mutex<Option<FullyCompiledProgram>>> =
    Lazy::new(|| Mutex::new(None));

pub fn named_addresses() -> BTreeMap<String, NumericalAddress> {
    let mut address_mapping = move_stdlib::move_stdlib_named_addresses();
    address_mapping.extend(
        MOVEOS_NAMED_ADDRESS_MAPPING
            .iter()
            .map(|(name, addr)| (name.to_string(), NumericalAddress::parse_str(addr).unwrap())),
    );
    address_mapping
}

#[derive(Debug, Eq, PartialEq, Default)]
enum Format {
    #[default]
    Pretty,
    Terse,
    Json,
}

impl Display for Format {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Format::Pretty => write!(f, "pretty"),
            Format::Terse => write!(f, "terse"),
            Format::Json => write!(f, "json"),
        }
    }
}

impl Format {
    fn variants() -> Vec<&'static str> {
        vec!["pretty", "terse"]
    }
}

impl FromStr for Format {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Format, std::string::String> {
        match s {
            "pretty" => Ok(Format::Pretty),
            "terse" => Ok(Format::Terse),
            "json" => Ok(Format::Json),
            _ => Err(format!("Unsupported format: {}", s)),
        }
    }
}

#[derive(Debug, Args)]
pub struct TestOpts {
    /// The FILTER string is tested against the name of all tests, and only those tests whose names
    /// contain the filter are run.
    filter: Option<String>,

    #[clap(long = "exact")]
    /// Exactly match filters rather than by substring
    filter_exact: bool,

    #[clap(long, env = "RUST_TEST_THREADS", default_value = "32")]
    /// Number of threads used for running tests in parallel
    test_threads: NonZeroUsize,

    #[clap(short = 'q', long)]
    /// Output minimal information
    quiet: bool,

    #[clap(long)]
    /// List all tests
    list: bool,

    #[clap(long)]
    /// Configure formatting of output:
    ///   pretty = Print verbose output;
    ///   terse = Display one character per test;
    ///   (json is unsupported, exists for compatibility with the default test harness)
    #[clap(possible_values = Format::variants(), default_value_t, ignore_case = true)]
    format: Format,
}

/// Integration test
#[derive(Parser)]
pub struct IntegrationTest {
    #[clap(flatten)]
    test_opts: TestOpts,
    #[clap(long = "ub")]
    /// update test baseline.
    update_baseline: bool,

    #[clap(long)]
    /// Print usage of tasks.
    task_help: bool,

    #[clap(long)]
    /// Task name to print usage, if None, print all tasks.
    task_name: Option<String>,

    /// Named addresses for the move binary
    ///
    /// Example: alice=0x1234, bob=0x5678
    ///
    /// Note: This will fail if there are duplicates in the Move.toml file remove those first.
    #[clap(long, parse(try_from_str = crate::utils::parse_map), default_value = "")]
    pub(crate) named_addresses: BTreeMap<String, String>,
}

impl IntegrationTest {
    pub fn execute(self, move_arg: Move) -> anyhow::Result<()> {
        let rerooted_path = {
            let path = match move_arg.package_path {
                Some(_) => move_arg.package_path.clone(),
                None => Some(std::env::current_dir()?),
            };
            // Always root ourselves to the package root, and then compile relative to that.
            SourcePackageLayout::try_find_root(&path.as_ref().unwrap().canonicalize()?)?
        };

        let mut build_config = move_arg.build_config;
        build_config.additional_named_addresses = self
            .named_addresses
            .clone()
            .into_iter()
            .map(|(key, value)| (key, AccountAddress::from_hex_literal(&value).unwrap()))
            .collect();
        let resolved_graph =
            build_config.resolution_graph_for_package(&rerooted_path, &mut std::io::stdout())?;

        let path = path_in_crate(rerooted_path.join("sources").to_str().unwrap());
        let files = find_filenames(&[path], |p| extension_equals(p, MOVE_EXTENSION)).unwrap();
        let targets = vec![PackagePaths {
            name: None,
            paths: files,
            named_address_map: {
                let mut address_mapping = match &resolved_graph.root_package.addresses {
                    Some(named_address_map) => named_address_map
                        .iter()
                        .filter(|(_, v)| v.is_some())
                        .map(|(k, v)| {
                            (
                                k.clone().as_str().to_string(),
                                NumericalAddress::new(v.unwrap().into_bytes(), NumberFormat::Hex),
                            )
                        })
                        .collect(),
                    None => BTreeMap::new(),
                };
                // address_mapping.extend(named_addresses());
                address_mapping.extend(
                    self.named_addresses
                        .into_iter()
                        .map(|(key, value)| (key, NumericalAddress::parse_str(&value).unwrap()))
                        .collect::<BTreeMap<String, NumericalAddress>>(),
                );
                address_mapping
            },
        }];

        let program_res = construct_pre_compiled_lib(targets, None, move_compiler::Flags::empty())?;
        let pre_compiled_lib = match program_res {
            Ok(af) => af,
            Err((files, errors)) => {
                eprintln!("!!!Package failed to compile!!!");
                move_compiler::diagnostics::report_diagnostics(&files, errors)
            }
        };
        {
            // update the global
            *G_PRE_COMPILED_LIB.lock().unwrap() = Some(pre_compiled_lib);
        }

        let tests_dir = rerooted_path.join(INTEGRATION_TESTS_DIR);

        if !tests_dir.exists() || !tests_dir.is_dir() {
            eprintln!("No integration tests file in the dir `integration-tests`.");
            return Ok(());
        }

        let requirements = datatest_stable::Requirements::new(
            move |path| {
                rooch_integration_test_runner::run_test_impl(
                    path,
                    G_PRE_COMPILED_LIB.lock().unwrap().as_ref(),
                )
            },
            "integration-test333".to_string(),
            tests_dir.display().to_string(),
            r".*\.move".to_string(),
        );
        if self.update_baseline {
            std::env::set_var(UPDATE_BASELINE, "true");
        }
        let mut test_args = vec![
            "test_runner".to_string(),
            "--format".to_string(),
            self.test_opts.format.to_string(),
            "--test-threads".to_string(),
            self.test_opts.test_threads.to_string(),
        ];
        if self.test_opts.list {
            test_args.push("--list".to_string());
        }
        if self.test_opts.quiet {
            test_args.push("--quiet".to_string());
        }
        if self.test_opts.filter_exact {
            test_args.push("--exact".to_string());
        }

        if let Some(filter) = self.test_opts.filter {
            test_args.push("--".to_string());
            test_args.push(filter);
        }

        let test_opts = datatest_stable::TestOpts::try_parse_from(test_args.as_slice())?;
        datatest_stable::runner_with_opts(&[requirements], test_opts);
        Ok(())
    }
}

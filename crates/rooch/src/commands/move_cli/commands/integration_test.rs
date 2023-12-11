// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use clap::{Args, Parser};
use move_cli::Move;
use move_command_line_common::address::NumericalAddress;
use move_command_line_common::testing::UPDATE_BASELINE;
use move_compiler::command_line::compiler::construct_pre_compiled_lib_from_compiler;
use move_compiler::diagnostics::report_diagnostics;
use move_compiler::shared::unique_map::UniqueMap;
use move_compiler::shared::{NamedAddressMapIndex, NamedAddressMaps};
use move_compiler::{
    cfgir, expansion, hlir, naming, parser, typing, Compiler, Flags, FullyCompiledProgram,
};
use move_package::compilation::build_plan::BuildPlan;
use move_package::source_package::layout::SourcePackageLayout;
use moveos_types::addresses::MOVEOS_NAMED_ADDRESS_MAPPING;
use once_cell::sync::Lazy;
use rooch_integration_test_runner;
use std::collections::BTreeMap;
use std::fmt::Display;
use std::num::NonZeroUsize;
use std::str::FromStr;
use std::sync::Mutex;

use crate::cli_types::WalletContextOptions;

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

#[derive(Debug, Clone, Eq, PartialEq, Default)]
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
    #[clap(default_value_t, ignore_case = true)]
    format: Format,
}

/// Integration test
#[derive(Parser)]
pub struct IntegrationTest {
    #[clap(flatten)]
    context_options: WalletContextOptions,

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
    /// Example: alice=0x1234, bob=default, alice2=alice
    ///
    /// Note: This will fail if there are duplicates in the Move.toml file remove those first.
    #[clap(long, value_parser = crate::utils::parse_map::<String, String>, default_value = "")]
    pub(crate) named_addresses: BTreeMap<String, String>,
}

impl IntegrationTest {
    pub async fn execute(self, move_arg: Move) -> anyhow::Result<()> {
        let rerooted_path = {
            let path = match move_arg.package_path {
                Some(_) => move_arg.package_path,
                None => Some(std::env::current_dir()?),
            };
            // Always root ourselves to the package root, and then compile relative to that.
            SourcePackageLayout::try_find_root(&path.as_ref().unwrap().canonicalize()?)?
        };

        let context = self.context_options.build()?;

        // force move to rebuild all packages, so that we can use compile_driver to generate the full compiled program.
        let mut build_config = move_arg.build_config;
        build_config
            .additional_named_addresses
            .extend(context.parse_and_resolve_addresses(self.named_addresses)?);

        build_config.force_recompilation = true;

        let resolved_graph =
            build_config.resolution_graph_for_package(&rerooted_path, &mut std::io::stdout())?;

        let (pre_compiled_lib, _compiled_package) = {
            let mut pre_compiled_lib = FullyCompiledProgram {
                files: Default::default(),
                parser: parser::ast::Program {
                    named_address_maps: NamedAddressMaps::new(),
                    source_definitions: vec![],
                    lib_definitions: vec![],
                },
                expansion: expansion::ast::Program {
                    modules: UniqueMap::new(),
                    scripts: Default::default(),
                },
                naming: naming::ast::Program {
                    modules: UniqueMap::new(),
                    scripts: Default::default(),
                },
                typing: typing::ast::Program {
                    modules: UniqueMap::new(),
                    scripts: Default::default(),
                },
                inlining: typing::ast::Program {
                    modules: UniqueMap::new(),
                    scripts: Default::default(),
                },
                hlir: hlir::ast::Program {
                    modules: UniqueMap::new(),
                    scripts: Default::default(),
                },
                cfgir: cfgir::ast::Program {
                    modules: UniqueMap::new(),
                    scripts: Default::default(),
                },
                compiled: vec![],
            };
            let compiled = BuildPlan::create(resolved_graph)?.compile_with_driver(
                &mut std::io::stdout(),
                Some(6),
                |compiler: Compiler| {
                    let compiler =
                        compiler.set_flags(Flags::empty().set_keep_testing_functions(true));
                    let full_program = match construct_pre_compiled_lib_from_compiler(compiler)? {
                        Ok(full_program) => full_program,
                        Err((file, s)) => report_diagnostics(&file, s),
                    };
                    pre_compiled_lib.files.extend(full_program.files.clone());
                    pre_compiled_lib
                        .parser
                        .source_definitions
                        .extend(full_program.parser.source_definitions);
                    pre_compiled_lib.parser.named_address_maps =
                        full_program.parser.named_address_maps.clone();
                    pre_compiled_lib.expansion.modules =
                        pre_compiled_lib.expansion.modules.union_with(
                            &full_program.expansion.modules.filter_map(|_k, v| {
                                if v.is_source_module {
                                    Some(v)
                                } else {
                                    None
                                }
                            }),
                            |_k, v1, _v2| v1.clone(),
                        );
                    pre_compiled_lib.naming.modules = pre_compiled_lib.naming.modules.union_with(
                        &full_program.naming.modules.filter_map(|_k, v| {
                            if v.is_source_module {
                                Some(v)
                            } else {
                                None
                            }
                        }),
                        |_k, v1, _v2| v1.clone(),
                    );
                    pre_compiled_lib.typing.modules = pre_compiled_lib.typing.modules.union_with(
                        &full_program.typing.modules.filter_map(|_k, v| {
                            if v.is_source_module {
                                Some(v)
                            } else {
                                None
                            }
                        }),
                        |_k, v1, _v2| v1.clone(),
                    );
                    pre_compiled_lib.inlining.modules =
                        pre_compiled_lib.inlining.modules.union_with(
                            &full_program.inlining.modules.filter_map(|_k, v| {
                                if v.is_source_module {
                                    Some(v)
                                } else {
                                    None
                                }
                            }),
                            |_k, v1, _v2| v1.clone(),
                        );
                    pre_compiled_lib.hlir.modules = pre_compiled_lib.hlir.modules.union_with(
                        &full_program.hlir.modules.filter_map(|_k, v| {
                            if v.is_source_module {
                                Some(v)
                            } else {
                                None
                            }
                        }),
                        |_k, v1, _v2| v1.clone(),
                    );
                    pre_compiled_lib.cfgir.modules = pre_compiled_lib.cfgir.modules.union_with(
                        &full_program.cfgir.modules.filter_map(|_k, v| {
                            if v.is_source_module {
                                Some(v)
                            } else {
                                None
                            }
                        }),
                        |_k, v1, _v2| v1.clone(),
                    );
                    pre_compiled_lib
                        .compiled
                        .extend(full_program.compiled.clone());

                    Ok((full_program.files, full_program.compiled))
                },
            )?;
            (pre_compiled_lib, compiled)
        };

        let named_addresses_maps = pre_compiled_lib
            .parser
            .named_address_maps
            .get(NamedAddressMapIndex(0))
            .clone();

        {
            // update the global
            *G_PRE_COMPILED_LIB.lock().unwrap() = Some(pre_compiled_lib);
        }

        let tests_dir = rerooted_path.join(INTEGRATION_TESTS_DIR);

        if !tests_dir.exists() || !tests_dir.is_dir() {
            eprintln!("No integration tests file in the dir `integration-tests`.");
            return Ok(());
        }

        let mut named_address_string_map = BTreeMap::new();
        let _ = named_addresses_maps
            .iter()
            .map(|(key, value)| {
                named_address_string_map.insert(key.to_string(), value.to_string());
            })
            .collect::<Vec<_>>();

        let requirements = datatest_stable::Requirements::new(
            move |path, data| {
                rooch_integration_test_runner::run_integration_test_with_extended_check(
                    path,
                    G_PRE_COMPILED_LIB.lock().unwrap().as_ref(),
                    data,
                )
            },
            "integration-test333".to_owned(),
            tests_dir.display().to_string(),
            r".*\.move".to_owned(),
            named_address_string_map,
        );
        if self.update_baseline {
            std::env::set_var(UPDATE_BASELINE, "true");
        }
        let mut test_args = vec![
            "test_runner".to_owned(),
            "--format".to_owned(),
            self.test_opts.format.to_string(),
            "--test-threads".to_owned(),
            self.test_opts.test_threads.to_string(),
        ];
        if self.test_opts.list {
            test_args.push("--list".to_owned());
        }
        if self.test_opts.quiet {
            test_args.push("--quiet".to_owned());
        }
        if self.test_opts.filter_exact {
            test_args.push("--exact".to_owned());
        }

        if let Some(filter) = self.test_opts.filter {
            test_args.push("--".to_owned());
            test_args.push(filter);
        }

        let test_opts = datatest_stable::TestOpts::try_parse_from(test_args.as_slice())?;
        datatest_stable::runner_with_opts(&[requirements], test_opts);

        Ok(())
    }
}

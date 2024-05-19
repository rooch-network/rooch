// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use clap::Parser;
use framework_builder::releaser;
use framework_builder::stdlib_version::StdlibVersion;

#[derive(Parser)]
#[clap(name = "framework-release", author = "The Rooch Core Contributors")]
struct StdlibOpts {
    /// Version number for compiled stdlib, starting from 1 and increasing continuously.
    #[clap(short = 'v', long, value_name = "VERSION")]
    version: Option<u64>,

    /// don't check compatibility between the old and new standard library
    #[clap(short = 'n', long)]
    no_check_compatibility: bool,
}

fn main() {
    let _ = tracing_subscriber::fmt::try_init();
    let opts: StdlibOpts = StdlibOpts::parse();
    let version = StdlibVersion::new(opts.version.unwrap_or(0));
    releaser::release(version, !opts.no_check_compatibility).expect("release failed");
}

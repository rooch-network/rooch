// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use clap::Parser;
use rooch::RoochCli;
use std::process::exit;

#[cfg(not(target_env = "msvc"))]
mod allocator {
    use tikv_jemallocator::Jemalloc;

    pub type Allocator = Jemalloc;

    pub const fn allocator() -> Allocator {
        Jemalloc
    }
}

#[cfg(target_env = "msvc")]
mod allocator {
    use mimalloc::MiMalloc;

    pub type Allocator = MiMalloc;

    pub const fn allocator() -> Allocator {
        MiMalloc
    }
}

#[global_allocator]
static GLOBAL: allocator::Allocator = allocator::allocator();

/// rooch is a command line tools for Rooch Network
#[tokio::main]
async fn main() {
    // Initialize logging with ERROR level by default to prevent info! contamination
    // Use ERROR level unless GC_VERBOSE_MODE is set (for test scenarios)
    let log_level = if std::env::var("GC_VERBOSE_MODE").is_ok() {
        tracing::Level::INFO
    } else {
        tracing::Level::ERROR
    };

    let _ = tracing_subscriber::fmt()
        .with_max_level(log_level)
        .try_init();

    let opt = RoochCli::parse();
    let result = rooch::run_cli(opt).await;

    match result {
        Ok(s) => println!("{}", s),
        Err(e) => {
            println!("{}", e);
            exit(1);
        }
    }
}

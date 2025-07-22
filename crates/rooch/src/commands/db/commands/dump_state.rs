// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::anyhow;
use clap::Parser;
use metrics::RegistryService;
use moveos_store::state_store::statedb::STATEDB_DUMP_BATCH_SIZE;
use moveos_types::h256::H256;
use rooch_config::RoochOpt;
use rooch_db::RoochDB;
use rooch_types::error::RoochResult;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;
use tracing::info;

#[derive(Debug, Parser)]
pub struct DumpStateCommand {
    #[clap(long, short = 'o')]
    output_file: PathBuf,

    #[clap(long)]
    state_root: String,

    #[clap(long, default_value_t = STATEDB_DUMP_BATCH_SIZE)]
    batch_size: usize,

    #[clap(flatten)]
    rooch_opt: RoochOpt,
}

impl DumpStateCommand {
    pub async fn execute(self) -> RoochResult<String> {
        let state_root = H256::from_str(&self.state_root)
            .map_err(|e| anyhow!("Invalid state root hash: {}", e))?;

        let mut output_file = File::create(&self.output_file)
            .map_err(|e| anyhow!("Failed to create output file: {}", e))?;

        let registry_service = RegistryService::default();
        let rooch_db = RoochDB::init(
            self.rooch_opt.store_config(),
            &registry_service.default_registry(),
        )
        .map_err(|e| anyhow!("Failed to initialize RoochDB: {}", e))?;
        let state_store = rooch_db.moveos_store.get_state_store();
        let smt = &state_store.smt;

        let state_kvs = smt
            .dump(state_root)
            .map_err(|e| anyhow!("Failed to read state data: {}", e))?;

        let total_count = state_kvs.len();

        for (key, value) in &state_kvs {
            let line = format!(
                "0x{}:0x{}\n",
                hex::encode(key.as_slice()),
                hex::encode(value.value.as_slice())
            );
            output_file
                .write_all(line.as_bytes())
                .map_err(|e| anyhow!("Failed to write to file: {}", e))?;
        }

        output_file
            .flush()
            .map_err(|e| anyhow!("Failed to flush file: {}", e))?;

        let result = format!(
            "Successfully exported state data to file {:?}, total {} records",
            self.output_file, total_count
        );

        info!("{}", result);
        Ok(result)
    }
}

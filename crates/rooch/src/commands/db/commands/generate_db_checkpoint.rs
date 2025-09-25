use crate::utils::open_rooch_db_readonly;
use clap::Parser;
use raw_store::SchemaStore;
use rocksdb::checkpoint::Checkpoint;
use rooch_types::error::RoochResult;
use rooch_types::rooch_network::RoochChainID;
use std::path::PathBuf;

/// generate RocksDB's checkpoint to directory
#[derive(Debug, Parser)]
pub struct GenerateDBCheckPointCommand {
    #[clap(long, short = 'o')]
    output_dir: PathBuf,

    #[clap(long = "data-dir", short = 'd')]
    pub base_data_dir: Option<PathBuf>,

    #[clap(long, short = 'n')]
    pub chain_id: Option<RoochChainID>,
}

impl GenerateDBCheckPointCommand {
    pub async fn execute(self) -> RoochResult<()> {
        let (_root, rooch_db, _start_time) =
            open_rooch_db_readonly(self.base_data_dir.clone(), self.chain_id.clone());
        let rocks_db = rooch_db
            .moveos_store
            .node_store
            .get_store()
            .store()
            .db()
            .expect("open rocksdb instance failed.")
            .inner();

        let check_point = Checkpoint::new(rocks_db).expect("create checkpoint failed.");
        check_point
            .create_checkpoint(self.output_dir.as_path())
            .expect("create checkpoint failed.");
        println!("create checkpoint succeeded.");

        Ok(())
    }
}

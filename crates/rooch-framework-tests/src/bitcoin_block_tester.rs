// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::binding_test::RustBindingTest;
use anyhow::{anyhow, ensure, Result};
use bitcoin::{hashes::Hash, Block, BlockHash, Txid};
use framework_builder::stdlib_version::StdlibVersion;
use moveos_types::{
    moveos_std::{
        module_store::ModuleStore, object::ObjectMeta, simple_multimap::SimpleMultiMap,
        timestamp::Timestamp,
    },
    state::{MoveState, MoveType, ObjectChange, ObjectState},
    state_resolver::StateResolver,
};
use rooch_relayer::actor::bitcoin_client_proxy::BitcoinClientProxy;
use rooch_types::{
    bitcoin::{
        ord::InscriptionID,
        utxo::{BitcoinUTXOStore, UTXO},
    },
    genesis_config,
    into_address::IntoAddress,
    rooch_network::{BuiltinChainID, RoochNetwork},
    transaction::L1BlockWithBody,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
    vec,
};
use tracing::{debug, info};

/// Execute Bitcoin block and test base a emulated environment
/// We prepare the Block's previous dependencies and execute the block
pub struct BitcoinBlockTester {
    genesis: BitcoinTesterGenesis,
    binding_test: RustBindingTest,
}

impl BitcoinBlockTester {
    pub fn new(height: u64) -> Result<Self> {
        let genesis = BitcoinTesterGenesis::load(height)?;
        let utxo_store_change = genesis.utxo_store_change.clone();
        let mut binding_test = RustBindingTest::new_with_network(genesis.network.clone())?;
        let root_changes = vec![utxo_store_change];
        binding_test.apply_changes(root_changes)?;
        Ok(Self {
            genesis,
            binding_test,
        })
    }

    pub fn execute(&mut self) -> Result<()> {
        for (height, block) in &self.genesis.blocks {
            let l1_block = L1BlockWithBody::new_bitcoin_block(*height as u64, block.clone());
            self.binding_test.execute_l1_block_and_tx(l1_block)?;
        }
        Ok(())
    }

    pub fn verify_utxo(&self) -> Result<()> {
        //TODO verify utxo in state with block output
        Ok(())
    }

    pub fn verify_inscriptions(&self) -> Result<()> {
        //TODO verify inscription in state with ord rpc result.
        Ok(())
    }

    pub fn get_inscription(&self, inscription_id: &InscriptionID) -> Result<Option<ObjectState>> {
        let object_id = inscription_id.object_id();
        self.binding_test.get_object(&object_id)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BitcoinTesterGenesis {
    pub height: u64,
    pub blocks: Vec<(usize, Block)>,
    pub utxo_store_change: ObjectChange,
    pub network: RoochNetwork,
}

impl BitcoinTesterGenesis {
    pub fn save(&self) -> Result<()> {
        let genesis_path = path_in_crate(format!("tester/{}.tester.genesis", self.height));
        info!("Save genesis to: {:?}", genesis_path);
        let genesis_content = bcs::to_bytes(self)?;
        std::fs::write(genesis_path, genesis_content)?;
        Ok(())
    }

    pub fn load(height: u64) -> Result<Self> {
        let genesis_path = path_in_crate(format!("tester/{}.tester.genesis", height));
        info!("Load genesis from: {:?}", genesis_path);
        let genesis_content = std::fs::read(genesis_path)?;
        let genesis: BitcoinTesterGenesis = bcs::from_bytes(&genesis_content)?;
        Ok(genesis)
    }
}

pub struct TesterGenesisBuilder {
    bitcoin_client: BitcoinClientProxy,
    blocks: Vec<(usize, Block)>,
    block_txids: HashSet<Txid>,
    utxo_store_change: ObjectChange,
}

impl TesterGenesisBuilder {
    pub fn new(bitcoin_client: BitcoinClientProxy) -> Result<Self> {
        Ok(Self {
            bitcoin_client,
            blocks: vec![],
            block_txids: HashSet::new(),
            utxo_store_change: ObjectChange::meta(BitcoinUTXOStore::genesis_object().metadata),
        })
    }

    pub async fn add_block(mut self, block_hash: BlockHash) -> Result<Self> {
        let block = self.bitcoin_client.get_block(block_hash).await?;
        let block_header_result = self
            .bitcoin_client
            .get_block_header_info(block_hash)
            .await?;
        debug!("Add block: {:?}", block_header_result);
        if !self.blocks.is_empty() {
            let last_block = self.blocks.last().unwrap();
            ensure!(
                last_block.0 < block_header_result.height,
                "Block height should be incremental from {} to {}",
                last_block.0,
                block_header_result.height
            );
        }
        for tx in &block.txdata {
            self.block_txids.insert(tx.txid());
        }

        let depdent_txids = block
            .txdata
            .iter()
            .flat_map(|tx| {
                tx.input
                    .iter()
                    .map(|input| input.previous_output.txid)
                    .collect::<Vec<_>>()
            })
            .collect::<HashSet<_>>();

        let mut depdent_txs = HashMap::new();
        for txid in depdent_txids {
            if txid == Txid::all_zeros() {
                continue;
            }
            // Skip if tx already in block
            if self.block_txids.contains(&txid) {
                continue;
            }
            debug!("Get tx: {:?}", txid);
            let tx = self.bitcoin_client.get_raw_transaction(txid).await?;
            depdent_txs.insert(txid, tx);
        }

        for tx in &block.txdata {
            for input in tx.input.iter() {
                if input.previous_output.txid == Txid::all_zeros() {
                    continue;
                }
                if self.block_txids.contains(&input.previous_output.txid) {
                    continue;
                }
                let pre_tx = depdent_txs
                    .get(&input.previous_output.txid)
                    .ok_or_else(|| {
                        anyhow!("Missing previous tx:{:?}", input.previous_output.txid)
                    })?;
                let pre_output = pre_tx
                    .output
                    .get(input.previous_output.vout as usize)
                    .ok_or_else(|| {
                        anyhow!("Missing previous output:{:?}", input.previous_output)
                    })?;
                let rooch_pre_output: rooch_types::bitcoin::types::TxOut =
                    pre_output.clone().into();
                let txid = input.previous_output.txid;
                let utxo = UTXO::new(
                    txid.into_address(),
                    input.previous_output.vout,
                    pre_output.value.to_sat(),
                    SimpleMultiMap::create(),
                );
                let object_id = utxo.object_id();
                debug!("Add utxo: {}, {:?}", object_id, utxo);
                let mut object_meta = ObjectMeta::genesis_meta(object_id, UTXO::type_tag());
                object_meta.owner = rooch_pre_output.recipient_address.to_rooch_address().into();
                let utxo_obj = ObjectState::new_with_struct(object_meta, utxo)?;

                self.utxo_store_change
                    .add_field_change(ObjectChange::new_object(utxo_obj))?;
            }
        }
        self.blocks.push((block_header_result.height, block));
        Ok(self)
    }

    pub async fn build(self) -> Result<BitcoinTesterGenesis> {
        let block_height = self.blocks[0].0 as u64;
        let timestamp_milliseconds = (self.blocks[0].1.header.time as u64) * 1000;
        let mut genesis_config = genesis_config::G_MAIN_CONFIG.clone();
        genesis_config.bitcoin_block_hash = self.blocks[0].1.block_hash();
        genesis_config.bitcoin_block_height = block_height;
        genesis_config.bitcoin_reorg_block_count = 0;
        genesis_config.timestamp = timestamp_milliseconds;
        genesis_config.stdlib_version = StdlibVersion::Latest;
        genesis_config.genesis_objects = vec![
            (
                ObjectState::new_timestamp(Timestamp {
                    milliseconds: timestamp_milliseconds,
                }),
                Timestamp::type_layout(),
            ),
            (
                ObjectState::genesis_module_store(),
                ModuleStore::type_layout(),
            ),
        ];

        debug!(
            "utxo store changes: {}",
            self.utxo_store_change.metadata.size
        );
        debug_assert!(
            self.utxo_store_change.metadata.size == (self.utxo_store_change.fields.len() as u64)
        );
        let chain_id = BuiltinChainID::Main.chain_id();
        let network = RoochNetwork::new(chain_id, genesis_config);
        let bitcoin_block_genesis = BitcoinTesterGenesis {
            height: block_height,
            blocks: self.blocks,
            utxo_store_change: self.utxo_store_change,
            network,
        };

        Ok(bitcoin_block_genesis)
    }
}

pub(crate) fn path_in_crate<S>(relative: S) -> PathBuf
where
    S: AsRef<Path>,
{
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push(relative);
    path
}

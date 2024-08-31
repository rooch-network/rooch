// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::binding_test::RustBindingTest;
use anyhow::{anyhow, bail, ensure, Result};
use bitcoin::{hashes::Hash, Block, OutPoint, TxOut, Txid};
use framework_builder::stdlib_version::StdlibVersion;
use moveos_types::{
    move_std::string::MoveString,
    moveos_std::{
        module_store::ModuleStore, object::ObjectMeta, simple_multimap::SimpleMultiMap,
        timestamp::Timestamp,
    },
    state::{MoveState, MoveStructType, MoveType, ObjectChange, ObjectState},
    state_resolver::StateResolver,
};
use rooch_ord::ord_client::Charm;
use rooch_relayer::actor::bitcoin_client_proxy::BitcoinClientProxy;
use rooch_types::{
    bitcoin::{
        inscription_updater::{
            InscriptionCreatedEvent, InscriptionTransferredEvent, InscriptionUpdaterEvent,
        },
        ord::{Inscription, InscriptionID, SatPoint},
        utxo::{self, BitcoinUTXOStore, UTXO},
    },
    genesis_config,
    into_address::IntoAddress,
    rooch_network::{BuiltinChainID, RoochNetwork},
    transaction::L1BlockWithBody,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeSet, HashMap, HashSet},
    path::{Path, PathBuf},
    vec,
};
use tracing::{debug, error, info, trace};

/// Execute Bitcoin block and test base a emulated environment
/// We prepare the Block's previous dependencies and execute the block
pub struct BitcoinBlockTester {
    genesis: BitcoinTesterGenesis,
    binding_test: RustBindingTest,
    executed_block: Option<BlockData>,
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
            executed_block: None,
        })
    }

    /// Execute one block from genesis blocks
    pub fn execute(&mut self) -> Result<()> {
        if self.genesis.blocks.is_empty() {
            bail!("No block to execute");
        }
        let mut block_data = self.genesis.blocks.remove(0);
        info!("Execute block: {}", block_data.height);
        let l1_block =
            L1BlockWithBody::new_bitcoin_block(block_data.height, block_data.block.clone());
        let results = self.binding_test.execute_l1_block_and_tx(l1_block)?;
        for result in results {
            for event in result.output.events {
                if event.event_type == InscriptionCreatedEvent::struct_tag() {
                    let event = InscriptionCreatedEvent::from_bytes(&event.event_data)?;
                    block_data
                        .events_from_move
                        .push(InscriptionUpdaterEvent::InscriptionCreated(event));
                } else if event.event_type == InscriptionTransferredEvent::struct_tag() {
                    let event = InscriptionTransferredEvent::from_bytes(&event.event_data)?;
                    block_data
                        .events_from_move
                        .push(InscriptionUpdaterEvent::InscriptionTransferred(event));
                }
            }
        }

        block_data.expect_inscriptions = block_data
            .events_from_ord
            .iter()
            .filter_map(|event| {
                if let rooch_ord::event::Event::InscriptionCreated { inscription_id, .. } = event {
                    Some(*inscription_id)
                } else {
                    None
                }
            })
            .collect::<BTreeSet<_>>();

        info!(
            "Execute block: {} done, events from move: {}, events from ord: {}, expect inscriptions: {}",
            block_data.height,
            block_data.events_from_move.len(),
            block_data.events_from_ord.len(),
            block_data.expect_inscriptions.len()
        );
        self.executed_block = Some(block_data);
        Ok(())
    }

    pub fn verify_utxo(&self) -> Result<()> {
        ensure!(
            self.executed_block.is_some(),
            "No block executed, please execute block first"
        );
        let mut utxo_set = HashMap::<OutPoint, TxOut>::new();
        let block_data = self.executed_block.as_ref().unwrap();
        for tx in block_data.block.txdata.as_slice() {
            let txid = tx.txid();
            for (index, tx_out) in tx.output.iter().enumerate() {
                let vout = index as u32;
                let out_point = OutPoint::new(txid, vout);
                utxo_set.insert(out_point, tx_out.clone());
            }
            //remove spent utxo
            for tx_in in tx.input.iter() {
                utxo_set.remove(&tx_in.previous_output);
            }
        }

        for (outpoint, tx_out) in utxo_set.into_iter() {
            if tx_out.script_pubkey.is_op_return() {
                continue;
            }
            let utxo_object_id = utxo::derive_utxo_id(&outpoint.into());
            let utxo_obj = self.binding_test.get_object(&utxo_object_id)?;
            ensure!(
                utxo_obj.is_some(),
                "Missing utxo object: {}, {}",
                utxo_object_id,
                outpoint
            );
            let utxo_obj = utxo_obj.unwrap();
            let utxo_state = utxo_obj.value_as::<UTXO>().map_err(|e| {
                error!("Parse UTXO Error: {:?}, object: {:?}", e, utxo_obj);
                e
            })?;
            trace!(
                "Check utxo: outpoint {}, utxo obj metadata: {:?}, utxo value: {:?}",
                outpoint,
                utxo_obj.metadata,
                utxo_state
            );
            ensure!(
                utxo_state.value == tx_out.value.to_sat(),
                "UTXO not match: {:?}, {:?}",
                utxo_state,
                tx_out
            );
            //Ensure every utxo's seals are correct
            let seals = utxo_state.seals;
            if !seals.is_empty() {
                let inscription_obj_ids = seals
                    .borrow(&MoveString::from(
                        Inscription::type_tag().to_canonical_string(),
                    ))
                    .expect("Inscription seal not found");
                for inscription_obj_id in inscription_obj_ids {
                    let inscription_obj = self.binding_test.get_object(inscription_obj_id)?;
                    ensure!(
                        inscription_obj.is_some(),
                        "Missing inscription object: {:?}",
                        inscription_obj_id
                    );
                    let inscription_obj = inscription_obj.unwrap();
                    let inscription = inscription_obj.value_as::<Inscription>().map_err(|e| {
                        error!(
                            "Parse Inscription Error: {:?}, object meta: {:?}, object value: {}",
                            e,
                            inscription_obj.metadata,
                            hex::encode(&inscription_obj.value)
                        );
                        e
                    })?;
                    ensure!(
                        inscription.location.outpoint == outpoint.into(),
                        "Inscription location not match: {:?}, {:?}",
                        inscription,
                        outpoint
                    );
                }
            }
        }
        Ok(())
    }

    pub fn verify_inscriptions(&self) -> Result<()> {
        ensure!(
            self.executed_block.is_some(),
            "No block executed, please execute block first"
        );
        let block_data = self.executed_block.as_ref().unwrap();
        info!(
            "verify {} inscriptions in block",
            block_data.expect_inscriptions.len()
        );
        let inscription_created_events_from_ord = block_data
            .events_from_ord
            .iter()
            .filter_map(|event| {
                if matches!(event, rooch_ord::event::Event::InscriptionCreated { .. }) {
                    Some(event.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        let inscription_created_events_from_move = block_data
            .events_from_move
            .iter()
            .filter_map(|event| {
                if matches!(event, InscriptionUpdaterEvent::InscriptionCreated(_)) {
                    Some(event.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        ensure!(
            inscription_created_events_from_ord.len() == inscription_created_events_from_move.len(),
            "Inscription created events not match: ord: {}, move: {}",
            inscription_created_events_from_ord.len(),
            inscription_created_events_from_move.len()
        );

        for (event_from_ord, event_from_move) in inscription_created_events_from_ord
            .iter()
            .zip(inscription_created_events_from_move.iter())
        {
            match (event_from_ord, event_from_move) {
                (
                    rooch_ord::event::Event::InscriptionCreated {
                        charms,
                        inscription_id,
                        location,
                        ..
                    },
                    InscriptionUpdaterEvent::InscriptionCreated(event),
                ) => {
                    ensure!(
                        inscription_id == &event.inscription_id,
                        "Inscription id not match: {:?}, {:?}",
                        inscription_id,
                        event.inscription_id
                    );
                    let location_from_move: Option<SatPoint> = event.location.clone().into();
                    ensure!(
                        location == &location_from_move,
                        "Inscription {} location not match: {:?}, {:?}",
                        inscription_id,
                        location,
                        location_from_move
                    );
                    if charms != &event.charms {
                        let charms_from_ord = Charm::charms(*charms);
                        let charms_from_move = Charm::charms(event.charms);
                        debug!("charms from ord: {:?}", charms_from_ord);
                        debug!("charms from move: {:?}", charms_from_move);
                        let charms_from_ord = charms_from_ord
                            .into_iter()
                            .filter(|charm| match charm {
                                //we skip the reinscription charm, because the previous inscription may be not in testcase state.
                                Charm::Reinscription => false,
                                _ => true,
                            })
                            .collect::<Vec<_>>();
                        if charms_from_ord != charms_from_move {
                            bail!(
                                "Inscription {} charms not match: ord: {:?}, move: {:?}",
                                inscription_id,
                                charms_from_ord,
                                charms_from_move
                            );
                        }
                    }
                }
                _ => {
                    bail!(
                        "Inscription created events not match: {:?}, {:?}",
                        event_from_ord,
                        event_from_move
                    );
                }
            }
        }

        // Because the block_tester do not contains all previous block's inscriptions
        // So we only verify the transferred inscriptions occurred in Move
        let inscription_transferred_events_from_move = block_data
            .events_from_move
            .iter()
            .filter_map(|event| {
                if let InscriptionUpdaterEvent::InscriptionTransferred(e) = event {
                    Some(e.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        info!(
            "Verify {} transferred inscriptions",
            inscription_transferred_events_from_move.len()
        );
        for event in inscription_transferred_events_from_move {
            let find = block_data.events_from_ord.iter().find(|ord_event| {
                if let rooch_ord::event::Event::InscriptionTransferred {
                    inscription_id,
                    new_location,
                    old_location,
                    ..
                } = ord_event
                {
                    inscription_id == &event.inscription_id
                        && new_location == &event.new_location
                        && old_location == &event.old_location
                } else {
                    false
                }
            });
            ensure!(
                find.is_some(),
                "Missing inscription transferred event: {:?} from ord.",
                event
            );
        }

        for inscription_id in block_data.expect_inscriptions.iter() {
            let object_id = inscription_id.object_id();
            let inscription_obj = self.binding_test.get_object(&object_id)?;
            ensure!(
                inscription_obj.is_some(),
                "Missing inscription object: {:?}",
                inscription_id
            );
            let inscription_obj = inscription_obj.unwrap();
            let inscription = inscription_obj.value_as::<Inscription>()?;
            let utxo_object_id = utxo::derive_utxo_id(&inscription.location.outpoint);
            let utxo_obj = self.binding_test.get_object(&utxo_object_id)?;
            ensure!(
                utxo_obj.is_some(),
                "Missing utxo object: {:?}",
                inscription.location.outpoint
            );
            let utxo = utxo_obj.unwrap().value_as::<UTXO>()?;
            let seal_obj_ids = utxo.seals.borrow(&MoveString::from(
                Inscription::type_tag().to_canonical_string(),
            ));
            ensure!(
                seal_obj_ids.is_some(),
                "Missing inscription seal in utxo: {:?}",
                utxo
            );
            ensure!(
                seal_obj_ids.unwrap().contains(&object_id),
                "Inscription seal not match: {:?}, {:?}",
                seal_obj_ids,
                object_id
            );
        }
        Ok(())
    }

    pub fn get_inscription(&self, inscription_id: &InscriptionID) -> Result<Option<ObjectState>> {
        let object_id = inscription_id.object_id();
        self.binding_test.get_object(&object_id)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BlockData {
    pub height: u64,
    pub block: Block,
    pub expect_inscriptions: BTreeSet<InscriptionID>,
    pub events_from_ord: Vec<rooch_ord::event::Event>,
    pub events_from_move: Vec<InscriptionUpdaterEvent>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BitcoinTesterGenesis {
    pub height: u64,
    pub blocks: Vec<BlockData>,
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
    ord_event_dir: PathBuf,
    blocks: Vec<BlockData>,
    block_txids: HashSet<Txid>,
    utxo_store_change: ObjectChange,
}

impl TesterGenesisBuilder {
    pub fn new<P: AsRef<Path>>(
        bitcoin_client: BitcoinClientProxy,
        ord_event_dir: P,
    ) -> Result<Self> {
        Ok(Self {
            bitcoin_client,
            ord_event_dir: ord_event_dir.as_ref().to_path_buf(),
            blocks: vec![],
            block_txids: HashSet::new(),
            utxo_store_change: ObjectChange::meta(BitcoinUTXOStore::genesis_object().metadata),
        })
    }

    pub async fn add_block(mut self, block_height: u64) -> Result<Self> {
        let block_hash = self.bitcoin_client.get_block_hash(block_height).await?;
        let block = self.bitcoin_client.get_block(block_hash).await?;
        let block_header_result = self
            .bitcoin_client
            .get_block_header_info(block_hash)
            .await?;
        info!("Add block: {:?}", block_header_result);
        if !self.blocks.is_empty() {
            let last_block = self.blocks.last().unwrap();
            ensure!(
                last_block.height < (block_header_result.height as u64),
                "Block height should be incremental from {} to {}",
                last_block.height,
                block_header_result.height
            );
        }
        let ord_events = rooch_ord::event::load_events(
            self.ord_event_dir.join(format!("{}.blk", block_height)),
        )?;
        info!(
            "Load ord events: {} in block {}",
            ord_events.len(),
            block_height
        );

        // let mut expect_inscriptions = BTreeMap::new();
        // for inscription_id in inscription_ids {
        //     let inscription_info = self
        //         .ord_client
        //         .get_inscription(&inscription_id)
        //         .await?
        //         .ok_or_else(|| anyhow!("Missing inscription: {:?}", inscription_id))?;
        //     expect_inscriptions.insert(inscription_id, inscription_info);
        // }

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
        info!("Get depdent_txs: {:?}", depdent_txids.len());
        for txid in depdent_txids {
            if txid == Txid::all_zeros() {
                continue;
            }
            // Skip if tx already in block
            if self.block_txids.contains(&txid) {
                continue;
            }
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
        self.blocks.push(BlockData {
            height: block_height,
            block,
            //we calculate expect_inscriptions from ord events
            expect_inscriptions: BTreeSet::new(),
            events_from_ord: ord_events,
            events_from_move: vec![],
        });
        Ok(self)
    }

    pub async fn build(self) -> Result<BitcoinTesterGenesis> {
        let block_height = self.blocks[0].height;
        let timestamp_milliseconds = (self.blocks[0].block.header.time as u64) * 1000;
        let mut genesis_config = genesis_config::G_MAIN_CONFIG.clone();
        genesis_config.bitcoin_block_hash = self.blocks[0].block.block_hash();
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

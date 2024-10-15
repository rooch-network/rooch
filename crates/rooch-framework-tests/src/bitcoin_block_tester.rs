// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::{bbn_tx_loader::BBNStakingTxRecord, binding_test::RustBindingTest};
use anyhow::{anyhow, bail, ensure, Result};
use bitcoin::{hashes::Hash, Block, OutPoint, TxOut, Txid};
use framework_builder::stdlib_version::StdlibVersion;
use move_core_types::{account_address::AccountAddress, u256::U256, vm_status::KeptVMStatus};
use moveos_types::{
    move_std::string::MoveString,
    moveos_std::{
        event::Event, module_store::ModuleStore, object::ObjectMeta,
        simple_multimap::SimpleMultiMap, timestamp::Timestamp,
    },
    state::{MoveState, MoveStructType, MoveType, ObjectChange, ObjectState},
    state_resolver::StateResolver,
};
use rooch_ord::ord_client::Charm;
use rooch_relayer::actor::bitcoin_client_proxy::BitcoinClientProxy;
use rooch_types::{
    bitcoin::{
        bbn::{
            self, BBNModule, BBNParsedV0StakingTx, BBNStakeSeal, BBNStakingEvent,
            BBNStakingFailedEvent,
        },
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
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{
    collections::{BTreeSet, HashMap, HashSet},
    path::{Path, PathBuf},
    vec,
};
use tracing::{debug, error, info, trace};

#[derive(Debug)]
struct ExecutedBlockData {
    block_data: BlockData,
    events: Vec<Event>,
}

impl ExecutedBlockData {
    pub fn filter_events<T: MoveStructType + DeserializeOwned>(&self) -> Vec<T> {
        self.events
            .iter()
            .filter_map(|event| {
                if event.event_type == T::struct_tag() {
                    let event = bcs::from_bytes::<T>(&event.event_data).unwrap();
                    Some(event)
                } else {
                    None
                }
            })
            .collect()
    }
}

/// Execute Bitcoin block and test base a emulated environment
/// We prepare the Block's previous dependencies and execute the block
pub struct BitcoinBlockTester {
    genesis: BitcoinTesterGenesis,
    binding_test: RustBindingTest,
    executed_block: Option<ExecutedBlockData>,
}

impl BitcoinBlockTester {
    pub fn new(height: u64) -> Result<Self> {
        let genesis = BitcoinTesterGenesis::load(height)?;
        let utxo_store_change = genesis.utxo_store_change.clone();

        let block_height = genesis.blocks[0].height;
        let timestamp_milliseconds = (genesis.blocks[0].block.header.time as u64) * 1000;
        let mut genesis_config = genesis_config::G_MAIN_CONFIG.clone();
        genesis_config.bitcoin_block_hash = genesis.blocks[0].block.block_hash();
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

        let chain_id = BuiltinChainID::Main.chain_id();
        let network = RoochNetwork::new(chain_id, genesis_config);

        let mut binding_test = RustBindingTest::new_with_network(network)?;
        let root_changes = vec![utxo_store_change];
        binding_test.apply_changes(root_changes)?;
        binding_test.get_rgas(binding_test.sequencer, U256::from(100000000000000u64))?;
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
        let mut events = vec![];
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

                events.push(event);
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
        self.executed_block = Some(ExecutedBlockData { block_data, events });
        Ok(())
    }

    pub fn verify_utxo(&self) -> Result<()> {
        ensure!(
            self.executed_block.is_some(),
            "No block executed, please execute block first"
        );
        let mut utxo_set = HashMap::<OutPoint, TxOut>::new();
        let executed_block_data = self.executed_block.as_ref().unwrap();
        for tx in executed_block_data.block_data.block.txdata.as_slice() {
            let txid = tx.compute_txid();
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
            //TODO migrate to verify inscription.
            //Ensure every utxo's seals are correct
            let seals = utxo_state.seals;
            if !seals.is_empty() {
                let inscription_obj_ids = seals.borrow(&MoveString::from(
                    Inscription::type_tag().to_canonical_string(),
                ));
                if let Some(inscription_obj_ids) = inscription_obj_ids {
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
        }
        Ok(())
    }

    pub fn verify_inscriptions(&self) -> Result<()> {
        ensure!(
            self.executed_block.is_some(),
            "No block executed, please execute block first"
        );
        let block_data = &self.executed_block.as_ref().unwrap().block_data;
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

    pub fn execute_bbn_process(&mut self) -> Result<()> {
        ensure!(
            self.executed_block.is_some(),
            "No block executed, please execute block first"
        );
        let executed_block_data = self.executed_block.as_mut().unwrap();

        for tx in executed_block_data.block_data.block.txdata.iter() {
            let txid = tx.compute_txid();
            if BBNParsedV0StakingTx::is_possible_staking_tx(tx, &bbn::BBN_GLOBAL_PARAM_BBN1.tag) {
                let function_call = BBNModule::create_process_bbn_tx_entry_call(txid)?;
                let execute_result = self
                    .binding_test
                    .execute_function_call_via_sequencer(function_call)?;
                debug!("BBN process result: {:?}", execute_result);
                if execute_result.transaction_info.status != KeptVMStatus::Executed {
                    let op_return_data = bbn::try_get_bbn_op_return_ouput(&tx.output);
                    bail!(
                        "tx should success, txid: {:?}, status: {:?}, op_return_data from rust: {:?}",
                        txid,
                        execute_result.transaction_info.status,
                        op_return_data
                    );
                }
                for event in execute_result.output.events {
                    executed_block_data.events.push(event);
                }
            }
        }

        Ok(())
    }

    pub fn verify_bbn_stake(&self) -> Result<()> {
        ensure!(
            self.executed_block.is_some(),
            "No block executed, please execute block first"
        );

        let executed_block_data = self.executed_block.as_ref().unwrap();

        let bbn_staking_txs = executed_block_data
            .block_data
            .bbn_staking_records
            .iter()
            .map(|tx| (tx.txid().into_address(), tx.clone()))
            .collect::<HashMap<AccountAddress, BBNStakingTxRecord>>();

        let bbn_staking_failed_events =
            executed_block_data.filter_events::<BBNStakingFailedEvent>();
        let bbn_staking_events = executed_block_data.filter_events::<BBNStakingEvent>();

        info!(
            "BBN staking txs: {}, staking failed events: {}, staking events: {}, total_events: {}",
            bbn_staking_txs.len(),
            bbn_staking_failed_events.len(),
            bbn_staking_events.len(),
            executed_block_data.events.len()
        );

        for event in &bbn_staking_failed_events {
            debug!("Staking failed event: {:?}", event);
            let txid = event.txid.into_address();
            ensure!(
                !bbn_staking_txs.contains_key(&txid),
                "Staking failed txid {:?} in event but also in staking txs from bbn indexer",
                txid,
            );
        }

        for (txid, _tx) in bbn_staking_txs.iter() {
            let event = bbn_staking_events.iter().find(|event| event.txid == *txid);
            ensure!(
                event.is_some(),
                "Staking txid {:?} in staking txs from bbn indexer but not in event",
                txid,
            );
        }

        for event in bbn_staking_events {
            let stake_object_id = event.stake_object_id;
            let txid = event.txid;
            let bbn_staking_tx = bbn_staking_txs.get(&txid);
            ensure!(
                bbn_staking_tx.is_some(),
                "Missing staking tx: {:?} in staking txs from bbn indexer",
                txid
            );

            let bbn_staking_tx = bbn_staking_tx.unwrap();

            let stake_obj = self.binding_test.get_object(&stake_object_id)?;
            ensure!(
                stake_obj.is_some(),
                "Missing stake object: {:?}, staking tx: {:?}",
                stake_object_id,
                bbn_staking_tx
            );
            let bbn_stake = stake_obj.unwrap().value_as::<BBNStakeSeal>()?;
            ensure!(
                bbn_stake.staking_output_index == bbn_staking_tx.staking_output_index,
                "Seal not match: {:?}, staking tx: {:?}",
                bbn_stake,
                bbn_staking_tx
            );
            ensure!(
                bbn_stake.staking_value == bbn_staking_tx.staking_value,
                "Staking value not match: {:?}, staking tx: {:?}",
                bbn_stake,
                bbn_staking_tx
            );
            ensure!(
                bbn_stake.staking_time == bbn_staking_tx.staking_time,
                "Staking time not match: {:?}, staking tx: {:?}",
                bbn_stake,
                bbn_staking_tx
            );
            ensure!(
                bbn_stake.staker_pub_key == bbn_staking_tx.staker_public_key(),
                "Staker public key not match: {:?}, staking tx: {:?}",
                bbn_stake,
                bbn_staking_tx
            );
            ensure!(
                bbn_stake.finality_provider_pub_key
                    == bbn_staking_tx.finality_provider_public_key(),
                "Finality provider public key not match: {:?}, staking tx: {:?}",
                bbn_stake,
                bbn_staking_tx
            );

            let staking_output_index = bbn_stake.staking_output_index;
            let outpoint = rooch_types::bitcoin::types::OutPoint::new(txid, staking_output_index);
            let utxo_object_id = utxo::derive_utxo_id(&outpoint);
            let utxo_obj = self.binding_test.get_object(&utxo_object_id)?;
            ensure!(
                utxo_obj.is_some(),
                "Missing utxo object: {:?} for staking tx: {:?}",
                utxo_object_id,
                bbn_staking_tx
            );
            let utxo_obj = utxo_obj.unwrap();
            let utxo = utxo_obj.value_as::<UTXO>()?;
            let seals = utxo.seals.borrow(&MoveString::from(
                BBNStakeSeal::type_tag().to_canonical_string(),
            ));
            ensure!(
                seals.is_some(),
                "Missing seals in utxo: {:?}, staking tx: {:?}",
                utxo,
                bbn_staking_tx
            );
            let seals = seals.unwrap();
            ensure!(
                seals.contains(&stake_object_id),
                "Missing seal object id in utxo: {:?}, staking tx: {:?}",
                utxo,
                bbn_staking_tx
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
    #[serde(default)]
    pub bbn_staking_records: Vec<BBNStakingTxRecord>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BitcoinTesterGenesis {
    pub height: u64,
    pub blocks: Vec<BlockData>,
    pub utxo_store_change: ObjectChange,
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
    ord_event_dir: Option<PathBuf>,
    bbn_staking_tx_csv: Option<PathBuf>,
    blocks: Vec<BlockData>,
    block_txids: HashSet<Txid>,
    utxo_store_change: ObjectChange,
}

impl TesterGenesisBuilder {
    pub fn new<P: AsRef<Path>>(
        bitcoin_client: BitcoinClientProxy,
        ord_event_dir: Option<P>,
        bbn_staking_tx_csv: Option<P>,
    ) -> Result<Self> {
        Ok(Self {
            bitcoin_client,
            ord_event_dir: ord_event_dir.map(|p| p.as_ref().to_path_buf()),
            bbn_staking_tx_csv: bbn_staking_tx_csv.map(|p| p.as_ref().to_path_buf()),
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
        let ord_events = match &self.ord_event_dir {
            None => vec![],
            Some(ord_event_dir) => {
                let ord_events = rooch_ord::event::load_events(
                    ord_event_dir.join(format!("{}.blk", block_height)),
                )?;
                info!(
                    "Load ord events: {} in block {}",
                    ord_events.len(),
                    block_height
                );
                ord_events
            }
        };

        let bbn_staking_records = match &self.bbn_staking_tx_csv {
            None => vec![],
            Some(bbn_staking_tx_csv) => {
                let bbn_staking_txs =
                    BBNStakingTxRecord::load_bbn_staking_txs(bbn_staking_tx_csv, block_height)?;
                info!(
                    "Load bbn staking txs: {} in block {}",
                    bbn_staking_txs.len(),
                    block_height
                );
                bbn_staking_txs
            }
        };

        for tx in &block.txdata {
            self.block_txids.insert(tx.compute_txid());
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
            bbn_staking_records,
        });
        Ok(self)
    }

    pub async fn build(self) -> Result<BitcoinTesterGenesis> {
        debug!(
            "utxo store changes: {}",
            self.utxo_store_change.metadata.size
        );
        debug_assert!(
            self.utxo_store_change.metadata.size == (self.utxo_store_change.fields.len() as u64)
        );
        let block_height = self.blocks[0].height;
        let bitcoin_block_genesis = BitcoinTesterGenesis {
            height: block_height,
            blocks: self.blocks,
            utxo_store_change: self.utxo_store_change,
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

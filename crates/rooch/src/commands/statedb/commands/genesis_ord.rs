// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::mpsc::{Receiver, SyncSender};
use std::sync::{mpsc, Arc};
use std::time::SystemTime;
use std::{fs, thread};

use anyhow::Result;
use bitcoin::hashes::Hash;
use bitcoin::{OutPoint, PublicKey};
use bitcoin_move::natives::ord::inscription_id::InscriptionId;
use clap::Parser;
use move_core_types::account_address::AccountAddress;
use serde::{Deserialize, Serialize};
use sled::Db;
use tempfile::TempDir;

use moveos_store::MoveOSStore;
use moveos_types::h256::H256;
use moveos_types::move_std::option::MoveOption;
use moveos_types::move_std::string::MoveString;
use moveos_types::moveos_std::object::{
    ObjectEntity, ObjectID, GENESIS_STATE_ROOT, SHARED_OBJECT_FLAG_MASK, SYSTEM_OWNER_ADDRESS,
};
use moveos_types::state::{KeyState, MoveState, State};
use rooch_config::R_OPT_NET_HELP;
use rooch_types::address::{BitcoinAddress, MultiChainAddress, RoochAddress};
use rooch_types::addresses::BITCOIN_MOVE_ADDRESS;
use rooch_types::bitcoin::ord::{
    derive_inscription_id, Inscription, InscriptionID, InscriptionStore,
};
use rooch_types::error::RoochResult;
use rooch_types::into_address::IntoAddress;
use rooch_types::multichain_id::RoochMultiChainID;
use rooch_types::rooch_network::RoochChainID;
use smt::UpdateSet;

use crate::cli_types::WalletContextOptions;
use crate::commands::statedb::commands::genesis_utxo::{
    apply_utxo_updates_to_state, produce_utxo_updates,
};
use crate::commands::statedb::commands::import::{apply_fields, apply_nodes};
use crate::commands::statedb::commands::init_genesis_job;

pub const ADDRESS_UNBOUND: &str = "unbound";
pub const ADDRESS_NON_STANDARD: &str = "non-standard";

// import data from ord
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InscriptionSource {
    pub sequence_number: u32,
    pub inscription_number: i32,
    pub id: InscriptionId,
    // ord crate has different version of bitcoin dependency, using string for compatibility
    pub satpoint_outpoint: String, // txid:vout
    pub satpoint_offset: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<Vec<u8>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_encoding: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Vec<u8>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metaprotocol: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<Vec<InscriptionId>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pointer: Option<u64>,
    pub is_p2pk: bool, // If true, address field is script
    pub address: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rune: Option<u128>,
}

/// Genesis Import BTC(utxo, ord)
#[derive(Debug, Parser)]
pub struct GenesisOrdCommand {
    #[clap(long, short = 'i')]
    /// utxo source data file. like ~/.rooch/local/utxo.csv or utxo.csv
    /// The file format is csv, and the first line is the header, the header is as follows:
    /// count,txid,vout,height,coinbase,amount,script,type,address
    pub utxo_input: PathBuf,
    #[clap(long)]
    /// ord source data file. like ~/.rooch/local/ord or ord
    /// The file format is json, and the first line is block height info: # export at block height <N>
    /// ord range: [0, N).
    /// ord_input & utxo_input must be in the same height
    /// ord_input must be sorted by sequence_number
    pub ord_input: PathBuf,

    #[clap(long = "data-dir", short = 'd')]
    /// Path to data dir, this dir is base dir, the final data_dir is base_dir/chain_network_name
    pub base_data_dir: Option<PathBuf>,

    /// If local chainid, start the service with a temporary data store.
    /// All data will be deleted when the service is stopped.
    #[clap(long, short = 'n', help = R_OPT_NET_HELP)]
    pub chain_id: Option<RoochChainID>,

    #[clap(long, default_value = "2097152")]
    pub utxo_batch_size: Option<usize>,
    #[clap(long, default_value = "1048576")] // ord may have large body, so set a smaller batch
    pub ord_batch_size: Option<usize>,

    #[clap(flatten)]
    pub context_options: WalletContextOptions,
}

impl GenesisOrdCommand {
    // 1. init import job
    // 2. import ord (record utxo_seal)
    // 3. import utxo with utxo_seal
    // 4. update genesis
    // 5. print job stats, clean env
    pub async fn execute(self) -> RoochResult<()> {
        // 1. init import job
        let utxo_ord_map_db_path = TempDir::new().unwrap().into_path();
        let id_ord_map_db_path = TempDir::new().unwrap().into_path();
        let (root, moveos_store, start_time) =
            init_genesis_job(self.base_data_dir.clone(), self.chain_id.clone());
        let pre_root_state_root = H256::from(root.state_root.into_bytes());
        let utxo_ord_map = Arc::new(sled::open(utxo_ord_map_db_path.clone()).unwrap());
        let moveos_store = Arc::new(moveos_store);
        let startup_update_set = UpdateSet::new();

        let utxo_input_path = self.utxo_input.clone();
        let utxo_batch_size = self.utxo_batch_size.unwrap();

        // 2. import od
        self.import_ord(
            utxo_ord_map.clone(),
            moveos_store.clone(),
            id_ord_map_db_path.clone(),
            startup_update_set.clone(),
        );

        // 3. import utxo
        import_utxo(
            utxo_input_path,
            utxo_batch_size,
            utxo_ord_map.clone(),
            moveos_store.clone(),
            startup_update_set.clone(),
            root.size,
            pre_root_state_root,
            start_time,
        );
        let utxo_ord_map_db = utxo_ord_map.clone();
        drop(utxo_ord_map_db);

        fs::remove_dir_all(utxo_ord_map_db_path.clone())?;
        fs::remove_dir_all(id_ord_map_db_path.clone())?;

        Ok(())
    }

    fn import_ord(
        self,
        utxo_ord_map: Arc<sled::Db>,
        moveos_store: Arc<MoveOSStore>,
        id_ord_map_db_path: PathBuf,
        startup_update_set: UpdateSet<KeyState, State>,
    ) {
        let input_path = self.ord_input.clone();
        let batch_size = self.ord_batch_size.unwrap();

        let (tx, rx) = mpsc::sync_channel(2);
        let produce_updates_thread = thread::spawn(move || {
            produce_ord_updates(
                tx,
                input_path,
                batch_size,
                utxo_ord_map,
                id_ord_map_db_path.clone(),
            )
        });
        let apply_updates_thread = thread::spawn(move || {
            apply_ord_updates_to_state(rx, moveos_store, startup_update_set);
        });
        produce_updates_thread.join().unwrap();
        apply_updates_thread.join().unwrap();
    }
}

fn import_utxo(
    input_path: PathBuf,
    batch_size: usize,
    utxo_ord_map: Arc<sled::Db>,
    moveos_store: Arc<MoveOSStore>,
    startup_update_set: UpdateSet<KeyState, State>,
    root_size: u64,
    root_state_root: H256,
    startup_time: SystemTime,
) {
    let (tx, rx) = mpsc::sync_channel(2);
    let produce_updates_thread =
        thread::spawn(move || produce_utxo_updates(tx, input_path, batch_size, Some(utxo_ord_map)));
    let apply_updates_thread = thread::spawn(move || {
        apply_utxo_updates_to_state(
            rx,
            moveos_store,
            root_size,
            root_state_root,
            Some(startup_update_set),
            startup_time,
        );
    });
    produce_updates_thread.join().unwrap();
    apply_updates_thread.join().unwrap();
}

fn apply_ord_updates_to_state(
    rx: Receiver<BatchUpdatesOrd>,
    moveos_store: Arc<MoveOSStore>,
    startup_update_set: UpdateSet<KeyState, State>,
) {
    let mut inscription_store_state_root = *GENESIS_STATE_ROOT;
    let mut last_inscription_store_state_root = inscription_store_state_root;
    let mut ord_count = 0u32;
    let mut cursed_inscription_count = 0u32;
    let mut blessed_inscription_count = 0u32;
    let moveos_store = moveos_store.as_ref();
    while let Ok(batch) = rx.recv() {
        let loop_start_time = SystemTime::now();

        let mut nodes: BTreeMap<H256, Vec<u8>> = BTreeMap::new();

        let cnt = batch.ord_updates.len();
        let mut ord_tree_change_set = apply_fields(
            moveos_store,
            inscription_store_state_root,
            batch.ord_updates,
        )
        .unwrap();
        nodes.append(&mut ord_tree_change_set.nodes);
        inscription_store_state_root = ord_tree_change_set.state_root;
        ord_count += cnt as u32;
        cursed_inscription_count += batch.cursed_inscription_count;
        blessed_inscription_count += batch.blessed_inscription_count;
        // TODO update inscriptions table_vec by batch.inscription_ids

        apply_nodes(moveos_store, nodes).expect("failed to apply ord nodes");

        println!(
            "{} ord applied ({} cursed, {} blessed). This bacth cost: {:?}",
            // e.g. batch_size = 8192:
            // 8192 ord applied in: 1.000000000s
            // 16384 ord applied in: 1.000000000s
            ord_count,
            cursed_inscription_count,
            blessed_inscription_count,
            loop_start_time.elapsed().unwrap()
        );

        log::debug!(
            "last inscription_store_state_root: {:?}, new inscription_store_state_root: {:?}",
            last_inscription_store_state_root,
            inscription_store_state_root,
        );

        last_inscription_store_state_root = inscription_store_state_root;
    }

    update_startup_ord(
        startup_update_set,
        inscription_store_state_root,
        ord_count,
        cursed_inscription_count,
        blessed_inscription_count,
    );
}

fn update_startup_ord(
    mut startup_update_set: UpdateSet<KeyState, State>,
    ord_store_state_root: H256,
    ord_count: u32,
    cursed_inscription_count: u32,
    blessed_inscription_count: u32,
) {
    let mut genesis_inscription_store_object = create_genesis_inscription_store_object(
        cursed_inscription_count,
        blessed_inscription_count,
        ord_count,
    );
    genesis_inscription_store_object.size += ord_count as u64;
    genesis_inscription_store_object.state_root = ord_store_state_root.into_address();
    let parent_id = InscriptionStore::object_id();
    startup_update_set.put(
        parent_id.to_key(),
        genesis_inscription_store_object.into_state(),
    );
}

struct BatchUpdatesOrd {
    ord_updates: UpdateSet<KeyState, State>,
    inscription_ids: Vec<InscriptionID>,
    cursed_inscription_count: u32,
    blessed_inscription_count: u32,
}

fn produce_ord_updates(
    tx: SyncSender<BatchUpdatesOrd>,
    input: PathBuf,
    batch_size: usize,
    utxo_ord_map: Arc<sled::Db>,
    id_ord_map_db_path: PathBuf,
) {
    let id_ord_map: Db = sled::open(id_ord_map_db_path).unwrap();
    let mut reader = BufReader::new(File::open(input).unwrap());
    let mut is_title_line = true;
    loop {
        let mut updates = BatchUpdatesOrd {
            ord_updates: UpdateSet::new(),
            inscription_ids: Vec::with_capacity(batch_size),
            cursed_inscription_count: 0,
            blessed_inscription_count: 0,
        };
        for line in reader.by_ref().lines().take(batch_size) {
            let line = line.unwrap();

            if is_title_line {
                is_title_line = false;
                if line.starts_with("# export at") {
                    // skip block height info
                    continue;
                }
            }

            let source: InscriptionSource = serde_json::from_str(&line).unwrap();
            if source.inscription_number < 0 {
                updates.cursed_inscription_count += 1;
            } else {
                updates.blessed_inscription_count += 1;
            }
            let (key, state, inscription_id) =
                gen_ord_update(source, utxo_ord_map.clone(), &id_ord_map).unwrap();
            updates.ord_updates.put(key, state);
            updates.inscription_ids.push(inscription_id);
        }
        if updates.ord_updates.is_empty() {
            break;
        }
        tx.send(updates).expect("failed to send updates");
    }

    drop(tx);
    drop(id_ord_map);
}

impl InscriptionSource {
    pub fn get_rooch_address(mut self) -> Result<AccountAddress> {
        if self.address == *ADDRESS_UNBOUND.to_string()
            || self.address == *ADDRESS_NON_STANDARD.to_string()
        {
            return Ok(BITCOIN_MOVE_ADDRESS);
        }

        if self.is_p2pk {
            let pubkey = PublicKey::from_str(self.address.as_str())?;
            let pubkey_hash = pubkey.pubkey_hash();
            let bitcoin_address = BitcoinAddress::new_p2pkh(&pubkey_hash);
            self.address = bitcoin_address.to_string();
        }

        let maddress = MultiChainAddress::try_from_str_with_multichain_id(
            RoochMultiChainID::Bitcoin,
            self.address.as_str(),
        )?;
        Ok(AccountAddress::from(RoochAddress::try_from(maddress)?))
    }

    pub fn to_inscription(self, id_ord_map: &sled::Db) -> (Inscription, InscriptionId) {
        let src = self;

        let ord_id = src.id;
        let txid: AccountAddress = ord_id.txid.into_address();

        let parents = get_ords_by_ids(id_ord_map, src.parent);

        let inscription = Inscription {
            txid,
            index: ord_id.index,
            offset: src.satpoint_offset,
            sequence_number: src.sequence_number,
            inscription_number: src.inscription_number.unsigned_abs(),
            is_curse: src.inscription_number.is_negative(),
            body: src.body.unwrap_or_default(),
            content_encoding: convert_option_string_to_move_type(src.content_encoding),
            content_type: convert_option_string_to_move_type(src.content_type),
            metadata: src.metadata.unwrap_or_default(),
            metaprotocol: convert_option_string_to_move_type(src.metaprotocol),
            pointer: src.pointer.into(),
            parents,
            rune: src.rune.into(),
        };
        (inscription, ord_id)
    }
}

fn gen_ord_update(
    src: InscriptionSource,
    utxo_ord_map: Arc<sled::Db>,
    id_ord_map: &sled::Db,
) -> Result<(KeyState, State, InscriptionID)> {
    let (inscription, src_inscription_id) = src.clone().to_inscription(id_ord_map);
    let address = src.clone().get_rooch_address()?;

    let inscription_id = InscriptionID::new(inscription.txid, inscription.index);
    let obj_id = derive_inscription_id(&inscription_id);
    let ord_obj = ObjectEntity::new(
        obj_id.clone(),
        address,
        0u8,
        *GENESIS_STATE_ROOT,
        0,
        0,
        0,
        inscription,
    );

    let satpoint_output_str = src.satpoint_outpoint.clone();
    let satpoint_output = OutPoint::from_str(satpoint_output_str.as_str()).unwrap();

    _ = update_ord_map(
        utxo_ord_map,
        id_ord_map,
        src_inscription_id,
        satpoint_output,
        obj_id.clone(),
    );

    Ok((ord_obj.id.to_key(), ord_obj.into_state(), inscription_id))
}

fn convert_option_string_to_move_type(opt: Option<String>) -> MoveOption<MoveString> {
    opt.map(MoveString::from).into()
}

// update id:object for parents
// update outpoint:object for utxo
fn update_ord_map(
    utxo_ord_map: Arc<sled::Db>,
    id_ord_map: &sled::Db,
    id: InscriptionId,
    outpoint: OutPoint,
    obj_id: ObjectID,
) -> bool {
    let id_key = bcs::to_bytes(&id).unwrap();
    id_ord_map
        .insert(id_key, bcs::to_bytes(&obj_id).unwrap())
        .unwrap();

    let is_unbound = outpoint.txid == Hash::all_zeros() && outpoint.vout == 0;
    if is_unbound {
        return is_unbound; // unbound has no output
    }

    let key = bcs::to_bytes(&outpoint).unwrap();
    let value = utxo_ord_map.get(&key).unwrap();
    if let Some(value) = value {
        let mut ord_ids: Vec<ObjectID> = bcs::from_bytes(&value).unwrap();
        ord_ids.push(obj_id);
        utxo_ord_map
            .insert(&key, bcs::to_bytes(&ord_ids).unwrap())
            .unwrap();
    } else {
        utxo_ord_map
            .insert(&key, bcs::to_bytes(&vec![obj_id]).unwrap())
            .unwrap();
    }
    is_unbound
}

fn get_ords_by_ids(id_ord_map: &sled::Db, ids: Option<Vec<InscriptionId>>) -> Vec<ObjectID> {
    if let Some(ids) = ids {
        let mut obj_ids = Vec::new();
        for id in ids {
            let obj_id = get_ord_by_id(id_ord_map, id);
            obj_ids.push(obj_id)
        }
        obj_ids
    } else {
        vec![]
    }
}

fn get_ord_by_id(id_ord_map: &sled::Db, id: InscriptionId) -> ObjectID {
    let id_key = bcs::to_bytes(&id).unwrap();
    let value = id_ord_map
        .get(id_key)
        .unwrap()
        .expect("get ord object id by inscriptionId must be succeed");
    bcs::from_bytes(&value).unwrap()
}

fn create_genesis_inscription_store_object(
    cursed_inscription_count: u32,
    blessed_inscription_count: u32,
    next_sequence_number: u32,
) -> ObjectEntity<InscriptionStore> {
    let inscription_store = InscriptionStore {
        inscriptions: ObjectID::from(AccountAddress::random()),
        cursed_inscription_count,
        blessed_inscription_count,
        next_sequence_number,
    };
    let obj_id = InscriptionStore::object_id();
    ObjectEntity::new(
        obj_id,
        SYSTEM_OWNER_ADDRESS,
        SHARED_OBJECT_FLAG_MASK,
        *GENESIS_STATE_ROOT,
        0,
        0,
        0,
        inscription_store,
    )
}

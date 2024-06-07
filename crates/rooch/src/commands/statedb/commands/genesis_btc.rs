// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, SyncSender};
use std::thread;
use std::time::SystemTime;

use anyhow::Result;
use bitcoin::hashes::Hash;
use bitcoin::{OutPoint, PublicKey};
use clap::Parser;
use moveos_store::MoveOSStore;
use ord::InscriptionId;
use ord::SatPoint;
use serde::{Deserialize, Serialize};
use sled::Db;

use moveos_types::h256::H256;
use moveos_types::move_std::option::MoveOption;
use moveos_types::moveos_std::object::{
    ObjectEntity, ObjectID, RootObjectEntity, GENESIS_STATE_ROOT, SHARED_OBJECT_FLAG_MASK,
    SYSTEM_OWNER_ADDRESS,
};
use moveos_types::startup_info::StartupInfo;
use moveos_types::state::{KeyState, MoveState, State};
use rooch_config::R_OPT_NET_HELP;
use rooch_types::address::AccountAddress;
use rooch_types::address::{BitcoinAddress, MultiChainAddress, RoochAddress};
use rooch_types::addresses::BITCOIN_MOVE_ADDRESS;
use rooch_types::bitcoin::ord::{
    derive_inscription_id, BitcoinInscriptionID, Inscription, InscriptionID, InscriptionStore,
};
use rooch_types::bitcoin::utxo::BitcoinUTXOStore;
use rooch_types::error::RoochResult;
use rooch_types::into_address::IntoAddress;
use rooch_types::multichain_id::RoochMultiChainID;
use rooch_types::rooch_network::RoochChainID;
use smt::UpdateSet;

use crate::cli_types::WalletContextOptions;
use crate::commands::statedb::commands::import::{apply_fields, apply_nodes, init_import_job};

pub const ADDRESS_UNBOUND: &str = "unbound";
pub const ADDRESS_NON_STANDARD: &str = "non-standard";

// import data from ord
#[derive(Serialize, Deserialize, Debug)]
pub struct InscriptionSource {
    pub sequence_number: u32,
    pub inscription_number: i32,
    pub id: InscriptionId,
    pub sat_point: ord::SatPoint,
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
    pub is_p2pk: bool,
    pub address: String,
}

/// Genesis Import BTC(utxo, ord)
#[derive(Debug, Parser)]
pub struct GenesisBTCCommand {
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
    pub ord_input: PathBuf,

    /// Path to utxo_ord_map_db
    /// <txid_vout>:<Vec<ord_objec_id>>
    pub utxo_ord_map_db: PathBuf,

    /// Path to id_ord_map_db
    /// <inscription_id>:<ord_object_id>
    pub id_ord_map_db: PathBuf,

    #[clap(long = "data-dir", short = 'd')]
    /// Path to data dir, this dir is base dir, the final data_dir is base_dir/chain_network_name
    pub base_data_dir: Option<PathBuf>,

    /// If local chainid, start the service with a temporary data store.
    /// All data will be deleted when the service is stopped.
    #[clap(long, short = 'n', help = R_OPT_NET_HELP)]
    pub chain_id: Option<RoochChainID>,

    #[clap(long, default_value = "2097152")]
    pub utxo_batch_size: Option<usize>,
    #[clap(long, default_value = "re")]
    pub ord_batch_size: Option<usize>,

    #[clap(flatten)]
    pub context_options: WalletContextOptions,
}

const UTXO_SEAL_INSCRIPTION_PROTOCOL: &str =
    "0000000000000000000000000000000000000000000000000000000000000004::ord::Inscription"; // In present, only one protocol is supported.

impl GenesisBTCCommand {
    // 1. init import job
    // 2. import ord (record utxo_seal)
    // 3. import utxo with utxo_seal
    // 3. update genesis
    pub async fn execute(self) -> RoochResult<()> {
        let (root, moveos_store, start_time) =
            init_import_job(self.base_data_dir.clone(), self.chain_id.clone());
        let pre_root_state_root = H256::from(root.state_root.into_bytes());
        let utxo_ord_map: Db = sled::open(self.utxo_ord_map_db.clone())?;
        let mut update_set = UpdateSet::new();

        self.import_ord(&utxo_ord_map, &moveos_store, &mut update_set);

        Ok(())
    }

    fn import_utxo(
        self,
        utxo_ord_map_db: &sled::Db,
        moveos_store: &MoveOSStore,
        update_set: &mut UpdateSet<KeyState, State>,
    ) {
    }

    fn import_ord(
        self,
        utxo_ord_map: &sled::Db,
        moveos_store: &MoveOSStore,
        update_set: &mut UpdateSet<KeyState, State>,
    ) {
        let id_ord_map: Db = sled::open(self.id_ord_map_db.clone())?;
        let input_path = self.ord_input.clone();
        let batch_size = self.ord_batch_size.clone().unwrap();
        let mut inscription_store_state_root = *GENESIS_STATE_ROOT;
        let mut ord_count = 0;

        let (tx, rx) = mpsc::sync_channel(2);
        let produce_updates_thread = thread::spawn(move || {
            produce_ord_updates(tx, input_path, batch_size, utxo_ord_map, &id_ord_map)
        });
        let apply_updates_thread = thread::spawn(move || {
            apply_updates_to_state(
                rx,
                moveos_store,
                &mut ord_count,
                &mut inscription_store_state_root,
            );
        });
        produce_updates_thread.join().unwrap();
        apply_updates_thread.join().unwrap();

        add_inscription_store_update_set(update_set, *ord_count, *inscription_store_state_root);

        utxo_ord_map.flush().unwrap();
        id_ord_map.flush().unwrap();
    }
}

fn apply_updates_to_state(
    rx: Receiver<BatchUpdatesOrd>,
    moveos_store: &MoveOSStore,
    ord_count: &mut u64,
    inscription_store_state_root: &mut H256,
) {
    let mut last_inscription_store_state_root = *inscription_store_state_root;
    while let Ok(batch) = rx.recv() {
        let loop_start_time = SystemTime::now();

        let mut nodes: BTreeMap<H256, Vec<u8>> = BTreeMap::new();

        let cnt = batch.ord_updates.len();
        let mut ord_tree_change_set = apply_fields(
            moveos_store,
            *inscription_store_state_root,
            batch.ord_updates,
        )
        .unwrap();
        nodes.append(&mut ord_tree_change_set.nodes);
        *inscription_store_state_root = ord_tree_change_set.state_root;
        *ord_count += cnt as u64;

        apply_nodes(moveos_store, nodes).expect("failed to apply ord nodes");

        println!(
            "{} ord applied. This bacth cost: {:?}",
            // because we skip the first line, count result keep missing one.
            // e.g. batch_size = 8192:
            // 8191 ord applied in: 1.000000000s
            // 16383 ord applied in: 1.000000000s
            *ord_count,
            loop_start_time.elapsed().unwrap()
        );

        log::debug!(
            "last inscription_store_state_root: {:?}, new inscription_store_state_root: {:?}",
            last_inscription_store_state_root,
            *inscription_store_state_root,
        );

        last_inscription_store_state_root = *inscription_store_state_root;
    }
}

fn add_inscription_store_update_set(
    update_set: &mut UpdateSet<KeyState, State>,
    ord_count: u64,
    ord_store_state_root: H256,
) {
    let mut genesis_inscription_store_object = create_genesis_inscription_store_object();
    genesis_inscription_store_object.size += ord_count;
    genesis_inscription_store_object.state_root = ord_store_state_root.into_address();
    let parent_id = InscriptionStore::object_id();
    update_set.put(
        parent_id.to_key(),
        genesis_inscription_store_object.into_state(),
    );
}

struct BatchUpdatesOrd {
    ord_updates: UpdateSet<KeyState, State>,
}

fn produce_ord_updates(
    tx: SyncSender<BatchUpdatesOrd>,
    input: PathBuf,
    batch_size: usize,
    utxo_ord_map: &sled::Db,
    id_ord_map: &sled::Db,
) {
    let mut reader = BufReader::new(File::open(input).unwrap());
    let mut is_title_line = true;
    let mut utxo_ord_bound_count = 0;
    loop {
        let mut updates = BatchUpdatesOrd {
            ord_updates: UpdateSet::new(),
        };
        for line in reader.by_ref().lines().take(batch_size) {
            let line = line.unwrap();

            if is_title_line {
                is_title_line = false;
                if line.starts_with("# export at") {
                    continue;
                }
            }

            let source: InscriptionSource = serde_json::from_str(&line).unwrap();

            let (key, state) =
                gen_ord_update(source, utxo_ord_map, id_ord_map, &mut utxo_ord_bound_count)
                    .unwrap();
            updates.ord_updates.put(key, state);
        }
        if updates.ord_updates.is_empty() {
            break;
        }
        tx.send(updates).expect("failed to send updates");
    }

    drop(tx);

    println!("utxo ord bound count: {}", utxo_ord_bound_count)
}

fn gen_ord_update(
    mut ord_data: InscriptionSource,
    utxo_ord_map: &sled::Db,
    id_ord_map: &sled::Db,
    utxo_ord_bound_count: &mut u64,
) -> Result<(KeyState, State)> {
    let txid_raw: bitcoin::Txid = ord_data.id.txid.into();
    let txid: AccountAddress = txid_raw.into_address();

    // reserve utxo by default bitcoin and rooch address
    let address = if ord_data.address == ADDRESS_UNBOUND.to_string()
        || ord_data.address == ADDRESS_NON_STANDARD.to_string()
    {
        let _bitcoin_address = BitcoinAddress::default();
        let address = BITCOIN_MOVE_ADDRESS;
        address
    } else {
        if ord_data.is_p2pk {
            let pubkey = PublicKey::from_str(ord_data.address.as_str())?;
            let pubkey_hash = pubkey.pubkey_hash();
            let bitcoin_address = BitcoinAddress::new_p2pkh(&pubkey_hash);
            ord_data.address = bitcoin_address.to_string();
        }

        let maddress = MultiChainAddress::try_from_str_with_multichain_id(
            RoochMultiChainID::Bitcoin,
            ord_data.address.as_str(),
        )?;
        let address = AccountAddress::from(RoochAddress::try_from(maddress.clone())?);
        address
    };

    let parent = get_ords_by_ids(id_ord_map, ord_data.parent);

    let inscription = Inscription {
        txid,
        index: ord_data.id.index,
        input: 0, // TODO may remove this field
        offset: ord_data.sat_point.offset,
        sequence_number: ord_data.sequence_number,
        inscription_number: ord_data.inscription_number.unsigned_abs(),
        is_curse: ord_data.inscription_number.is_negative(),
        body: ord_data.body.unwrap_or(vec![]),
        content_encoding: ord_data.content_encoding.into(),
        content_type: ord_data.content_type.into(),
        metadata: ord_data.metadata.unwrap_or(vec![]),
        metaprotocol: ord_data.metaprotocol.into(),
        parent: parent.into(),
        pointer: ord_data.pointer.into(),
    };
    let inscription_id = InscriptionID::new(txid, ord_data.id.index);
    let obj_id = derive_inscription_id(&inscription_id);
    let ord_obj = ObjectEntity::new(obj_id, address, 0u8, *GENESIS_STATE_ROOT, 0, inscription);

    if !update_ord_map(
        utxo_ord_map,
        id_ord_map,
        ord_data.id,
        ord_data.sat_point,
        obj_id.clone(),
    ) {
        *utxo_ord_bound_count += 1;
    }

    Ok((ord_obj.id.to_key(), ord_obj.into_state()))
}

fn update_ord_map(
    utxo_ord_map: &sled::Db,
    id_ord_map: &sled::Db,
    id: InscriptionId,
    sat_point: SatPoint,
    obj_id: ObjectID,
) -> bool {
    let id_key = bcs::to_bytes(&id).unwrap();
    id_ord_map
        .insert(&id_key, bcs::to_bytes(&obj_id).unwrap())
        .unwrap();

    let is_unbound = sat_point.outpoint.txid == Hash::all_zeros() && sat_point.outpoint.vout == 0;
    if is_unbound {
        return is_unbound;
    }

    let key = bcs::to_bytes(&(sat_point.outpoint)).unwrap();
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
    return is_unbound;
}

fn get_ords_by_ids(
    id_ord_map: &sled::Db,
    ids: Option<Vec<InscriptionId>>,
) -> Option<Vec<ObjectID>> {
    if let Some(ids) = ids {
        let mut obj_ids = Vec::new();
        for id in ids {
            let obj_id = get_ord_by_id(id_ord_map, id);
            obj_ids.push(obj_id)
        }
        Some(obj_ids)
    } else {
        None
    }
}

fn get_ord_by_id(id_ord_map: &sled::Db, id: InscriptionId) -> ObjectID {
    let id_key = bcs::to_bytes(&id).unwrap();
    let value = id_ord_map
        .get(&id_key)
        .unwrap()
        .expect("get ord object id by inscriptionId must be succeed");
    bcs::from_bytes(&value).unwrap()
}

fn get_ord_by_outpoint(utxo_ord_map: &sled::Db, outpoint: OutPoint) -> Option<Vec<ObjectID>> {
    let key = bcs::to_bytes(&outpoint).unwrap();
    let value = utxo_ord_map.get(&key).unwrap();
    if let Some(value) = value {
        Some(bcs::from_bytes(&value).unwrap())
    } else {
        None
    }
}

fn create_genesis_inscription_store_object() -> ObjectEntity<InscriptionStore> {
    // TODO fixme after InscriptionStore changed
    let inscription_store = InscriptionStore {
        latest_tx_index: 0,
        inscriptions: ObjectID::from(AccountAddress::random()),
        inscription_ids: ObjectID::from(AccountAddress::random()),
    };
    let obj_id = InscriptionStore::object_id();
    let obj = ObjectEntity::new(
        obj_id,
        SYSTEM_OWNER_ADDRESS,
        SHARED_OBJECT_FLAG_MASK,
        *GENESIS_STATE_ROOT,
        0,
        inscription_store,
    );
    obj
}

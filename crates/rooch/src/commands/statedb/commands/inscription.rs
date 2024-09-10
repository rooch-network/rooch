// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use framework_types::addresses::BITCOIN_MOVE_ADDRESS;
use move_core_types::account_address::AccountAddress;
use moveos_types::moveos_std::object::{
    DynamicField, ObjectEntity, ObjectID, FROZEN_OBJECT_FLAG_MASK, SYSTEM_OWNER_ADDRESS,
};
use moveos_types::state::{FieldKey, ObjectState};
use rooch_types::address::BitcoinAddress;
use rooch_types::bitcoin::ord::{
    derive_inscription_id, Inscription, InscriptionID, InscriptionStore, SatPoint,
};
use rooch_types::into_address::IntoAddress;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use serde_with::DisplayFromStr;
use std::io::BufRead;
use std::path::PathBuf;
use std::str::FromStr;

use crate::commands::statedb::commands::convert_option_string_to_move_type;

const ADDRESS_UNBOUND: &str = "unbound";
const ADDRESS_NON_STANDARD: &str = "non-standard";

const CHARM_BURNED_FLAG: u16 = 1 << 12;

#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InscriptionSource {
    pub sequence_number: u32,
    pub inscription_number: i32,
    #[serde_as(as = "DisplayFromStr")]
    pub id: InscriptionID,
    // ord crate may have a different version of bitcoin dependency, using string for compatibility
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
    #[serde_as(as = "Option<Vec<DisplayFromStr>>")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<Vec<InscriptionID>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pointer: Option<u64>,
    pub address: String, // <address>, "unbound", "non-standard"
    #[serde_as(as = "Option<DisplayFromStr>")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rune: Option<u128>,
    pub charms: u16,
}

impl InscriptionSource {
    pub fn from_str(line: &str) -> Self {
        serde_json::from_str(line).unwrap()
    }

    // derive account address from inscription source address(unbound/non-standard/valid_address)
    pub fn derive_account_address(&self) -> anyhow::Result<AccountAddress> {
        if self.address == *ADDRESS_UNBOUND.to_string()
            || self.address == *ADDRESS_NON_STANDARD.to_string()
        {
            return Ok(BITCOIN_MOVE_ADDRESS);
        }

        let bitcoin_address = BitcoinAddress::from_str(self.address.as_str()).unwrap();
        let address = AccountAddress::from(bitcoin_address.to_rooch_address());
        Ok(address)
    }

    pub fn to_inscription(&self) -> Inscription {
        let src = self;

        let txid: AccountAddress = src.id.txid.into_address();
        let parents = src.parent.clone();
        let id = InscriptionID::new(txid, src.id.index);
        let outpoint = bitcoin::OutPoint::from_str(src.satpoint_outpoint.as_str()).unwrap();
        let location = SatPoint {
            outpoint: outpoint.into(),
            offset: src.satpoint_offset,
        };

        Inscription {
            id,
            location,
            sequence_number: src.sequence_number,
            inscription_number: src.inscription_number.unsigned_abs(),
            is_cursed: src.inscription_number.is_negative(),
            charms: src.charms,
            body: src.body.clone().unwrap_or_default(),
            content_encoding: convert_option_string_to_move_type(src.content_encoding.clone()),
            content_type: convert_option_string_to_move_type(src.content_type.clone()),
            metadata: src.metadata.clone().unwrap_or_default(),
            metaprotocol: convert_option_string_to_move_type(src.metaprotocol.clone()),
            pointer: src.pointer.into(),
            parents: parents.unwrap_or_default(),
            rune: src.rune.into(),
        }
    }

    pub(crate) fn gen_update(&self) -> (FieldKey, ObjectState, InscriptionID) {
        let inscription = self.to_inscription();
        let address = self.derive_account_address().unwrap();

        let inscription_id = inscription.id;
        let obj_id = derive_inscription_id(&inscription_id);

        let mut owner = address;
        let mut flag = 0u8;
        if self.charms == CHARM_BURNED_FLAG {
            flag = FROZEN_OBJECT_FLAG_MASK;
            owner = SYSTEM_OWNER_ADDRESS;
        }

        let ord_obj = ObjectEntity::new(obj_id.clone(), owner, flag, None, 0, 0, 0, inscription);

        (ord_obj.id.field_key(), ord_obj.into_state(), inscription_id)
    }
}

// sequence_number:inscription_id
pub(crate) fn gen_inscription_id_update(
    sequence_number: u32,
    inscription_id: InscriptionID,
) -> (FieldKey, ObjectState) {
    let parent_id = InscriptionStore::object_id();
    let field: ObjectEntity<DynamicField<u32, InscriptionID>> =
        ObjectEntity::new_dynamic_field(parent_id, sequence_number, inscription_id);
    let state = field.into_state();
    let key = state.id().field_key();
    (key, state)
}

pub(crate) fn derive_inscription_ids(ids: Option<Vec<InscriptionID>>) -> Vec<ObjectID> {
    if let Some(ids) = ids {
        let mut obj_ids = Vec::with_capacity(ids.len());
        for id in ids {
            let obj_id = derive_inscription_id(&id);
            obj_ids.push(obj_id)
        }
        obj_ids
    } else {
        vec![]
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub(crate) struct InscriptionStats {
    pub block_height: u32,
    pub cursed_inscription_count: u32,
    pub blessed_inscription_count: u32,
    pub unbound_inscription_count: u32,
    pub lost_sats: u64,
    pub next_sequence_number: u32,
}

impl InscriptionStats {
    pub(crate) fn load_from_file(file_path: PathBuf) -> Self {
        let file = std::fs::File::open(file_path).unwrap();
        let mut reader = std::io::BufReader::new(file);
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        serde_json::from_str(line.as_str()).unwrap()
    }
}

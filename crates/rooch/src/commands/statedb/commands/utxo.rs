// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use std::str::FromStr;
use std::sync::Arc;

use bitcoin::{OutPoint, PublicKey, ScriptBuf, Txid};
use move_core_types::account_address::AccountAddress;
use rustc_hash::FxHashSet;

use framework_types::addresses::{BITCOIN_MOVE_ADDRESS, ROOCH_FRAMEWORK_ADDRESS};
use moveos_types::moveos_std::object::ObjectEntity;
use moveos_types::moveos_std::simple_multimap::SimpleMultiMap;
use moveos_types::state::{FieldKey, ObjectState};
use rooch_types::address::BitcoinAddress;
use rooch_types::bitcoin::utxo::{BitcoinUTXOStore, UTXO};
use rooch_types::bitcoin::{types, utxo};
use rooch_types::framework::address_mapping::RoochToBitcoinAddressMapping;
use rooch_types::into_address::IntoAddress;

use crate::commands::statedb::commands::{derive_utxo_inscription_seal, OutpointInscriptionsMap};

const SCRIPT_TYPE_P2MS: &str = "p2ms";
const SCRIPT_TYPE_P2PK: &str = "p2pk";
const SCRIPT_TYPE_NON_STANDARD: &str = "non-standard";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UTXORawData {
    pub(crate) txid: Txid,
    pub(crate) vout: u32,
    pub(crate) amount: u64,
    script: String,
    script_type: String,
    address: String,
    pub(crate) height: u64,
}

impl UTXORawData {
    pub fn is_valid_empty_address(&self) -> bool {
        SCRIPT_TYPE_P2PK.eq(self.script_type.as_str())
            || SCRIPT_TYPE_P2MS.eq(self.script_type.as_str())
            || SCRIPT_TYPE_NON_STANDARD.eq(self.script_type.as_str())
    }

    // csv format: count,txid,vout,height,coinbase,amount,script,type,address (9 fields)
    pub fn from_str(line: &str) -> Self {
        let str_list: Vec<&str> = line.trim().split(',').collect();
        assert_eq!(str_list.len(), 9);

        let txid = str_list[1].to_string();
        let vout = str_list[2].parse::<u32>().unwrap();
        let height = str_list[3].parse::<u64>().unwrap();
        let amount = str_list[5].parse::<u64>().unwrap();
        let script = str_list[6].to_string();
        let script_type = str_list[7].to_string();
        let address = str_list[8].to_string();
        let raw_data = UTXORawData {
            txid: Txid::from_str(txid.as_str()).unwrap(),
            vout,
            amount,
            script,
            script_type,
            address: address.clone(),
            height,
        };
        assert!(
            raw_data.is_valid_empty_address() || !address.is_empty(),
            "Invalid empty address"
        );
        raw_data
    }

    pub fn gen_utxo_update(
        &mut self,
        outpoint_inscriptions_map: Option<Arc<OutpointInscriptionsMap>>,
    ) -> (FieldKey, ObjectState) {
        let (address, _address_mapping_data) = self.gen_address_mapping_data();

        let seals = match outpoint_inscriptions_map {
            Some(outpoint_inscriptions_map) => {
                let inscriptions =
                    outpoint_inscriptions_map.search(&OutPoint::new(self.txid, self.vout));
                derive_utxo_inscription_seal(inscriptions)
            }
            None => SimpleMultiMap::create(),
        };

        let txid = self.txid.into_address();
        let outpoint = types::OutPoint::new(txid, self.vout);
        let utxo_object = ObjectEntity::new(
            utxo::derive_utxo_id(&outpoint),
            address,
            0u8,
            None,
            0,
            0,
            0,
            UTXO::new(txid, self.vout, self.amount, seals),
        );
        (utxo_object.id.field_key(), utxo_object.into_state())
    }

    pub fn gen_address_mapping_data(&mut self) -> (AccountAddress, Option<AddressMappingData>) {
        let bitcoin_address = derive_bitcoin_address(
            self.address.clone(),
            self.script.clone(),
            self.script_type.clone(),
        );
        match bitcoin_address {
            Some(bitcoin_address) => {
                let address = AccountAddress::from(bitcoin_address.to_rooch_address());
                if self.address.is_empty() {
                    self.address = bitcoin_address.to_string();
                }
                let address_mapping_data = Some(AddressMappingData::new(
                    self.address.clone(),
                    bitcoin_address,
                    address,
                ));
                (address, address_mapping_data)
            }
            None => (BITCOIN_MOVE_ADDRESS, None),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddressMappingData {
    pub origin_address: String,
    pub bitcoin_address: BitcoinAddress,
    pub address: AccountAddress,
}

impl AddressMappingData {
    pub fn new(
        origin_address: String,
        bitcoin_address: BitcoinAddress,
        address: AccountAddress,
    ) -> Self {
        AddressMappingData {
            origin_address,
            bitcoin_address,
            address,
        }
    }

    pub fn into_state(self) -> ObjectState {
        let parent_id = RoochToBitcoinAddressMapping::object_id();
        // Rooch address to bitcoin address dynamic field: name is rooch address, value is bitcoin address
        ObjectEntity::new_dynamic_field(parent_id, self.address, self.bitcoin_address).into_state()
    }

    pub fn gen_update(
        self,
        added_address_set: &mut FxHashSet<String>,
    ) -> Option<(FieldKey, ObjectState)> {
        let address = self.origin_address.clone();
        if added_address_set.insert(address) {
            let state = self.into_state();
            let key = state.id().field_key();
            return Some((key, state));
        }
        None
    }
}

// derive BitcoinAddress from UTXO data source
fn derive_bitcoin_address(
    origin_address: String,
    script: String,
    script_type: String,
) -> Option<BitcoinAddress> {
    if !origin_address.is_empty() {
        return Some(BitcoinAddress::from_str(origin_address.as_str()).unwrap());
    }
    if SCRIPT_TYPE_NON_STANDARD.eq(script_type.as_str()) {
        return None;
    }
    // Try to derive address from script
    let script_buf = ScriptBuf::from_hex(script.as_str()).unwrap();
    let bitcoin_address: BitcoinAddress = BitcoinAddress::from(&script_buf);
    if bitcoin_address != BitcoinAddress::default() {
        return Some(bitcoin_address);
    }
    // Try to derive address from p2pk pubkey
    if SCRIPT_TYPE_P2PK.eq(script_type.as_str()) {
        let pubkey = match PublicKey::from_str(script.as_str()) {
            Ok(pubkey) => pubkey,
            Err(_) => {
                return None;
            }
        };
        let pubkey_hash = pubkey.pubkey_hash();
        return Some(BitcoinAddress::new_p2pkh(&pubkey_hash));
    };
    None
}

pub fn create_genesis_utxo_store_object() -> ObjectState {
    BitcoinUTXOStore::genesis_object()
}

pub fn create_genesis_rooch_to_bitcoin_address_mapping_object(
) -> ObjectEntity<RoochToBitcoinAddressMapping> {
    ObjectEntity::new(
        RoochToBitcoinAddressMapping::object_id(),
        ROOCH_FRAMEWORK_ADDRESS,
        0u8,
        None,
        0,
        0,
        0,
        RoochToBitcoinAddressMapping::default(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gen_address_mapping_update() {
        let address_mapping_data = AddressMappingData {
            origin_address: "123 Main St".to_string(),
            bitcoin_address: Default::default(),
            address: AccountAddress::random(),
        };
        let mut added_address_set: FxHashSet<String> =
            FxHashSet::with_capacity_and_hasher(60_000_000, Default::default());
        let result = address_mapping_data
            .clone()
            .gen_update(&mut added_address_set);
        assert!(result.is_some());
        let result2 = address_mapping_data.gen_update(&mut added_address_set);
        assert!(result2.is_none());
    }

    #[test]
    fn test_drive_bitcoin_address() {
        // non-empty address
        let bitcoin_address = derive_bitcoin_address(
            "bc1qsn7v0rwezflwd6pk7xxf25zhjw9wkvmympm7tk".to_string(),
            "".to_string(),
            SCRIPT_TYPE_NON_STANDARD.to_string(), // no matter what script type
        );
        assert_eq!(
            "bc1qsn7v0rwezflwd6pk7xxf25zhjw9wkvmympm7tk",
            bitcoin_address.unwrap().to_string()
        );
        // non-standard address
        let bitcoin_address = derive_bitcoin_address(
            "".to_string(),
            "".to_string(),
            SCRIPT_TYPE_NON_STANDARD.to_string(),
        );
        assert_eq!(None, bitcoin_address);
        // p2pk script
        let bitcoin_address = derive_bitcoin_address(
            "".to_string(),
            "41049434a2dd7c5b82df88f578f8d7fd14e8d36513aaa9c003eb5bd6cb56065e44b7e0227139e8a8e68e7de0a4ed32b8c90edc9673b8a7ea541b52f2a22196f7b8cfac".to_string(),
            SCRIPT_TYPE_P2PK.to_string(),
        );
        assert_eq!(
            "14vrCdzPtnHaXtDNLH4xNhceS7GV4GMw76",
            bitcoin_address.unwrap().to_string()
        );
        // p2pk pubkey
        let bitcoin_address = derive_bitcoin_address(
            "".to_string(),
            "04f254e36949ec1a7f6e9548f16d4788fb321f429b2c7d2eb44480b2ed0195cbf0c3875c767fe8abb2df6827c21392ea5cc934240b9ac46c6a56d2bd13dd0b17a9".to_string(),
            SCRIPT_TYPE_P2PK.to_string(),
        );
        let pubkey = PublicKey::from_str(
            "04f254e36949ec1a7f6e9548f16d4788fb321f429b2c7d2eb44480b2ed0195cbf0c3875c767fe8abb2df6827c21392ea5cc934240b9ac46c6a56d2bd13dd0b17a9",
        )
            .unwrap();
        assert_eq!(
            BitcoinAddress::new_p2pkh(&pubkey.pubkey_hash()),
            bitcoin_address.unwrap()
        );
        // invalid p2pk pubkey
        let bitcoin_address = derive_bitcoin_address(
            "".to_string(),
            "036c6565662c206f6e7464656b2c2067656e6965742e2e2e202020202020202020".to_string(),
            SCRIPT_TYPE_P2PK.to_string(),
        );
        assert_eq!(None, bitcoin_address);
        // invalid p2pk script
        let bitcoin_address = derive_bitcoin_address(
            "".to_string(),
            "21036c6565662c206f6e7464656b2c2067656e6965742e2e2e202020202020202020ac".to_string(),
            SCRIPT_TYPE_P2PK.to_string(),
        );
        assert_eq!(None, bitcoin_address);
        // special p2ms case: https://ordinals.com/inscription/72552729(
        // output: a353a7943a2b38318bf458b6af878b8384f48a6d10aad5b827d0550980abe3f0:0
        // script: 0014f29f9316f0f1e48116958216a8babd353b491dae
        // address: bc1q720ex9hs78jgz954sgt23w4ax5a5j8dwjj5kkm
        // )
        let script = "0014f29f9316f0f1e48116958216a8babd353b491dae";
        let bitcoin_address = derive_bitcoin_address(
            "".to_string(),
            script.to_string(),
            SCRIPT_TYPE_P2MS.to_string(),
        );
        assert_eq!(
            "bc1q720ex9hs78jgz954sgt23w4ax5a5j8dwjj5kkm",
            bitcoin_address.unwrap().to_string()
        );
        // normal p2ms (cannot get payload)
        let script = "512102047da7156b82baaed491787e77a0d94cbc00ebdbd993639382b8a41d2f8d42dd2107000000000000000000000000000000000000000000000000000000000000000052ae";
        let bitcoin_address = derive_bitcoin_address(
            "".to_string(),
            script.to_string(),
            SCRIPT_TYPE_P2MS.to_string(),
        );
        assert_eq!(None, bitcoin_address);
    }
}

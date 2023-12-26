// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::address::BitcoinAddress;
use crate::addresses::BITCOIN_MOVE_ADDRESS;
use crate::indexer::state::IndexerGlobalState;
use move_core_types::language_storage::StructTag;
use move_core_types::{account_address::AccountAddress, ident_str, identifier::IdentStr};
use moveos_types::move_std::string::MoveString;
use moveos_types::moveos_std::object::ObjectID;
use moveos_types::moveos_std::simple_multimap::SimpleMultiMap;
use moveos_types::state::{MoveState, MoveStructState, MoveStructType};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UTXO {
    /// The txid of the UTXO
    pub txid: AccountAddress,
    /// The vout of the UTXO
    pub vout: u32,
    /// The value of the UTXO
    pub value: u64,
    /// Protocol seals
    pub seals: SimpleMultiMap<MoveString, ObjectID>,
}

impl UTXO {
    pub fn new(
        txid: AccountAddress,
        vout: u32,
        value: u64,
        seals: SimpleMultiMap<MoveString, ObjectID>,
    ) -> Self {
        Self {
            txid,
            vout,
            value,
            seals,
        }
    }
}

impl MoveStructType for UTXO {
    const ADDRESS: AccountAddress = BITCOIN_MOVE_ADDRESS;
    const MODULE_NAME: &'static IdentStr = ident_str!("utxo");
    const STRUCT_NAME: &'static IdentStr = ident_str!("UTXO");
}

impl MoveStructState for UTXO {
    fn struct_layout() -> move_core_types::value::MoveStructLayout {
        move_core_types::value::MoveStructLayout::new(vec![
            AccountAddress::type_layout(),
            u32::type_layout(),
            u64::type_layout(),
            SimpleMultiMap::<MoveString, ObjectID>::type_layout(),
        ])
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UTXOState {
    pub object_id: ObjectID,
    pub owner: AccountAddress,
    pub owner_bitcoin_address: Option<BitcoinAddress>,
    pub flag: u8,
    pub value: UTXO,
    pub object_type: StructTag,
    pub tx_order: u64,
    pub state_index: u64,
    pub created_at: u64,
    pub updated_at: u64,
}

impl UTXOState {
    pub fn new_from_global_state(
        state: IndexerGlobalState,
        utxo: UTXO,
        owner_bitcoin_address: Option<BitcoinAddress>,
    ) -> Self {
        Self {
            object_id: state.object_id,
            owner: state.owner,
            owner_bitcoin_address,
            flag: state.flag,
            value: utxo,
            object_type: state.object_type,
            tx_order: state.tx_order,
            state_index: state.state_index,
            created_at: state.created_at,
            updated_at: state.updated_at,
        }
    }
}

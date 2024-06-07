// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::jsonrpc_types::address::BitcoinAddressView;
use crate::jsonrpc_types::btc::transaction::{hex_to_txid, TxidView};
use crate::jsonrpc_types::{H256View, RoochAddressView, StructTagView};
use anyhow::Result;
use bitcoin::hashes::Hash;
use bitcoin::Txid;

use moveos_types::moveos_std::object::ObjectID;
use moveos_types::state::MoveStructType;
use rooch_types::address::RoochAddress;
use rooch_types::bitcoin::utxo::{self, UTXOState, UTXO};
use rooch_types::indexer::state::ObjectStateFilter;
use rooch_types::into_address::IntoAddress;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize, Eq, JsonSchema)]
pub struct BitcoinOutPointView {
    pub txid: TxidView,
    pub vout: u32,
}

impl From<BitcoinOutPointView> for bitcoin::OutPoint {
    fn from(view: BitcoinOutPointView) -> Self {
        bitcoin::OutPoint::new(view.txid.into(), view.vout)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum UTXOFilterView {
    /// Query by owner, represent by bitcoin address
    Owner(BitcoinAddressView),
    /// Query by bitcoin outpoint, represent by bitcoin txid and vout
    OutPoint { txid: String, vout: u32 },
    /// Query by object id.
    ObjectId(ObjectID),
    /// Query all.
    All,
}

impl UTXOFilterView {
    pub fn into_global_state_filter(
        filter_opt: UTXOFilterView,
        resolve_address: RoochAddress,
    ) -> Result<ObjectStateFilter> {
        Ok(match filter_opt {
            UTXOFilterView::Owner(_owner) => ObjectStateFilter::ObjectTypeWithOwner {
                object_type: UTXO::struct_tag(),
                owner: resolve_address,
            },
            UTXOFilterView::OutPoint { txid, vout } => {
                let txid = hex_to_txid(txid.as_str())?;
                let outpoint =
                    rooch_types::bitcoin::types::OutPoint::new(txid.into_address(), vout);
                let utxo_id = utxo::derive_utxo_id(&outpoint);
                ObjectStateFilter::ObjectId(vec![utxo_id])
            }
            UTXOFilterView::ObjectId(object_id) => ObjectStateFilter::ObjectId(vec![object_id]),
            UTXOFilterView::All => ObjectStateFilter::ObjectType(UTXO::struct_tag()),
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct UTXOView {
    /// The txid of the UTXO
    txid: H256View,
    /// The txid of the UTXO
    bitcoin_txid: TxidView,
    /// The vout of the UTXO
    vout: u32,
    /// The value of the UTXO
    value: u64,
    /// Protocol seals
    seals: String,
}

impl UTXOView {
    pub fn try_new_from_utxo(utxo: UTXO) -> Result<UTXOView, anyhow::Error> {
        // reversed bytes of txid
        let bitcoin_txid = Txid::from_byte_array(utxo.txid.into_bytes());
        let seals_str = serde_json::to_string(&utxo.seals)?;

        Ok(UTXOView {
            txid: utxo.txid.into(),
            bitcoin_txid: bitcoin_txid.into(),
            vout: utxo.vout,
            value: utxo.value,
            seals: seals_str,
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct UTXOStateView {
    pub object_id: ObjectID,
    pub owner: RoochAddressView,
    pub owner_bitcoin_address: Option<String>,
    pub flag: u8,
    pub value: Option<UTXOView>,
    pub object_type: StructTagView,
    pub tx_order: u64,
    pub state_index: u64,
    pub created_at: u64,
    pub updated_at: u64,
}

impl UTXOStateView {
    pub fn try_new_from_utxo_state(
        utxo: UTXOState,
        network: u8,
    ) -> Result<UTXOStateView, anyhow::Error> {
        let owner_bitcoin_address = match utxo.owner_bitcoin_address {
            Some(baddress) => Some(baddress.format(network)?),
            None => None,
        };
        let utxo_view = match utxo.value {
            Some(utxo) => Some(UTXOView::try_new_from_utxo(utxo)?),
            None => None,
        };
        Ok(UTXOStateView {
            object_id: utxo.object_id,
            owner: utxo.owner.into(),
            owner_bitcoin_address,
            flag: utxo.flag,
            value: utxo_view,
            object_type: utxo.object_type.into(),
            tx_order: utxo.tx_order,
            state_index: utxo.state_index,
            created_at: utxo.created_at,
            updated_at: utxo.updated_at,
        })
    }
}

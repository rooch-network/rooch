// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::jsonrpc_types::btc::transaction::{hex_to_txid, TxidView};
use crate::jsonrpc_types::{
    H256View, IndexerObjectStateView, IndexerStateIDView, ObjectIDVecView, ObjectIDView,
    ObjectMetaView, StrView, UnitedAddressView,
};
use anyhow::Result;
use bitcoin::hashes::Hash;
use bitcoin::Txid;

use moveos_types::move_std::string::MoveString;
use moveos_types::state::{MoveState, MoveStructType};
use rooch_types::bitcoin::types::OutPoint;
use rooch_types::bitcoin::utxo::{self, UTXO};
use rooch_types::indexer::state::ObjectStateFilter;
use rooch_types::into_address::{FromAddress, IntoAddress};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

impl From<OutPoint> for BitcoinOutPointView {
    fn from(outpoint: OutPoint) -> Self {
        BitcoinOutPointView {
            txid: Txid::from_address(outpoint.txid).into(),
            vout: outpoint.vout,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum UTXOFilterView {
    /// Query by owner, support rooch address and bitcoin address
    Owner(UnitedAddressView),
    /// Query by bitcoin outpoint, represent by bitcoin txid and vout
    OutPoint { txid: String, vout: u32 },
    /// Query by object ids.
    ObjectId(ObjectIDVecView),
    /// Query all.
    All,
}

impl UTXOFilterView {
    pub fn owner<A: Into<UnitedAddressView>>(owner: A) -> Self {
        let addr: UnitedAddressView = owner.into();
        UTXOFilterView::Owner(addr)
    }

    pub fn into_global_state_filter(filter_opt: UTXOFilterView) -> Result<ObjectStateFilter> {
        Ok(match filter_opt {
            UTXOFilterView::Owner(owner) => ObjectStateFilter::ObjectTypeWithOwner {
                object_type: UTXO::struct_tag(),
                filter_out: false,
                owner: owner.0.rooch_address.into(),
            },
            UTXOFilterView::OutPoint { txid, vout } => {
                let txid = hex_to_txid(txid.as_str())?;
                let outpoint =
                    rooch_types::bitcoin::types::OutPoint::new(txid.into_address(), vout);
                let utxo_id = utxo::derive_utxo_id(&outpoint);
                ObjectStateFilter::ObjectId(vec![utxo_id])
            }
            UTXOFilterView::ObjectId(object_id_vec_view) => {
                ObjectStateFilter::ObjectId(object_id_vec_view.into())
            }
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
    value: StrView<u64>,
    /// Protocol seals
    seals: HashMap<String, Vec<ObjectIDView>>,
}

impl UTXOView {
    pub fn try_new_from_utxo(utxo: UTXO) -> Result<UTXOView, anyhow::Error> {
        // reversed bytes of txid
        let bitcoin_txid = Txid::from_byte_array(utxo.txid.into_bytes());

        let mut seals_view: HashMap<String, Vec<ObjectIDView>> = HashMap::new();
        utxo.seals.data.into_iter().for_each(|element| {
            seals_view.insert(
                format!("0x{}", element.key),
                element
                    .value
                    .into_iter()
                    .map(|id| id.into())
                    .collect::<Vec<_>>(),
            );
        });

        Ok(UTXOView {
            txid: utxo.txid.into(),
            bitcoin_txid: bitcoin_txid.into(),
            vout: utxo.vout,
            value: utxo.value.into(),
            seals: seals_view,
        })
    }

    pub fn get_value(&self) -> u64 {
        self.value.0
    }
}

impl From<UTXOView> for UTXO {
    fn from(view: UTXOView) -> Self {
        UTXO {
            txid: view.txid.0.into_address(),
            vout: view.vout,
            value: view.value.0,
            seals: view
                .seals
                .into_iter()
                .map(|(k, v)| {
                    (
                        MoveString::from(k),
                        v.into_iter().map(|id| id.0).collect::<Vec<_>>(),
                    )
                })
                .collect::<Vec<_>>()
                .into(),
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct UTXOStateView {
    #[serde(flatten)]
    pub metadata: ObjectMetaView,
    pub value: UTXOView,
    #[serde(flatten)]
    pub indexer_id: IndexerStateIDView,
}

impl TryFrom<IndexerObjectStateView> for UTXOStateView {
    type Error = anyhow::Error;

    fn try_from(state: IndexerObjectStateView) -> Result<Self, Self::Error> {
        let utxo = UTXO::from_bytes(&state.value.0)?;
        let utxo_view = UTXOView::try_new_from_utxo(utxo)?;
        Ok(UTXOStateView {
            metadata: state.metadata,
            value: utxo_view,
            indexer_id: state.indexer_id,
        })
    }
}

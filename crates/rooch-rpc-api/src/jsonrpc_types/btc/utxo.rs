// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::jsonrpc_types::btc::transaction::TxidView;
use crate::jsonrpc_types::{
    H256View, IndexerObjectStateView, IndexerStateIDView, ObjectIDVecView, ObjectIDView,
    ObjectMetaView, ObjectStateView, StrView, UnitedAddressView,
};
use anyhow::Result;
use bitcoin::hashes::Hash;
use bitcoin::{Amount, TxOut, Txid};

use moveos_types::move_std::string::MoveString;
use moveos_types::state::{MoveState, MoveStructType};
use rooch_types::address::BitcoinAddress;
use rooch_types::bitcoin::types::OutPoint;
use rooch_types::bitcoin::utxo::{self, UTXO};
use rooch_types::indexer::state::ObjectStateFilter;
use rooch_types::into_address::{FromAddress, IntoAddress};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize, Eq, JsonSchema)]
pub struct OutPointView {
    pub txid: TxidView,
    pub vout: u32,
}

impl From<OutPointView> for bitcoin::OutPoint {
    fn from(view: OutPointView) -> Self {
        bitcoin::OutPoint::new(view.txid.0, view.vout)
    }
}

impl From<OutPointView> for OutPoint {
    fn from(view: OutPointView) -> Self {
        OutPoint {
            txid: view.txid.0.into_address(),
            vout: view.vout,
        }
    }
}

impl From<OutPoint> for OutPointView {
    fn from(outpoint: OutPoint) -> Self {
        OutPointView {
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

    pub fn object_ids<A: Into<ObjectIDVecView>>(object_ids: A) -> Self {
        let object_ids: ObjectIDVecView = object_ids.into();
        UTXOFilterView::ObjectId(object_ids)
    }

    pub fn into_global_state_filter(filter_opt: UTXOFilterView) -> Result<ObjectStateFilter> {
        Ok(match filter_opt {
            UTXOFilterView::Owner(owner) => ObjectStateFilter::ObjectTypeWithOwner {
                object_type: UTXO::struct_tag(),
                filter_out: false,
                owner: owner.0.rooch_address.into(),
            },
            UTXOFilterView::OutPoint { txid, vout } => {
                let txid = Txid::from_str(&txid)?;
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
    pub txid: H256View,
    /// The txid of the UTXO
    pub bitcoin_txid: TxidView,
    /// The vout of the UTXO
    pub vout: u32,
    /// The value of the UTXO
    pub value: StrView<u64>,
    /// Protocol seals
    pub seals: HashMap<String, Vec<ObjectIDView>>,
}

impl UTXOView {
    pub fn try_new_from_utxo(utxo: UTXO) -> Result<UTXOView, anyhow::Error> {
        // reversed bytes of txid
        let bitcoin_txid = Txid::from_byte_array(utxo.txid.into_bytes());

        let mut seals_view: HashMap<String, Vec<ObjectIDView>> = HashMap::new();
        utxo.seals.data.into_iter().for_each(|element| {
            seals_view.insert(
                element.key.to_string(),
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

    pub fn value(&self) -> u64 {
        self.value.0
    }

    pub fn txid(&self) -> Txid {
        self.bitcoin_txid.0
    }

    pub fn amount(&self) -> Amount {
        Amount::from_sat(self.value())
    }

    pub fn outpoint(&self) -> OutPoint {
        OutPoint {
            txid: self.txid.0.into_address(),
            vout: self.vout,
        }
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

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct UTXOObjectView {
    #[serde(flatten)]
    pub metadata: ObjectMetaView,
    pub value: UTXOView,
}

impl From<UTXOStateView> for UTXOObjectView {
    fn from(state: UTXOStateView) -> Self {
        UTXOObjectView {
            metadata: state.metadata,
            value: state.value,
        }
    }
}

impl UTXOObjectView {
    pub fn owner_bitcoin_address(&self) -> Option<BitcoinAddress> {
        self.metadata
            .owner_bitcoin_address
            .as_ref()
            .and_then(|addr| BitcoinAddress::from_str(addr).ok())
    }

    pub fn outpoint(&self) -> OutPoint {
        self.value.outpoint()
    }

    pub fn amount(&self) -> Amount {
        self.value.amount()
    }

    pub fn tx_output(&self) -> Result<TxOut> {
        // Rooch UTXO does keep the original tx output script_pubkey,
        // We convert the owner bitcoin address to script_pubkey here.
        // But if the TxOut is a non-standard script_pubkey, we can not convert it back to bitcoin address.
        // Find a way to keep the original script_pubkey in UTXO.
        let script_pubkey = self
            .owner_bitcoin_address()
            .map(|addr| addr.script_pubkey())
            .transpose()?
            .ok_or_else(|| {
                anyhow::anyhow!("Can not recognize the owner of UTXO {}", self.outpoint())
            })?;
        Ok(TxOut {
            value: self.amount(),
            script_pubkey,
        })
    }
}

impl TryFrom<ObjectStateView> for UTXOObjectView {
    type Error = anyhow::Error;

    fn try_from(state: ObjectStateView) -> Result<Self, Self::Error> {
        let utxo = UTXO::from_bytes(&state.value.0)?;
        let utxo_view = UTXOView::try_new_from_utxo(utxo)?;
        Ok(UTXOObjectView {
            metadata: state.metadata,
            value: utxo_view,
        })
    }
}

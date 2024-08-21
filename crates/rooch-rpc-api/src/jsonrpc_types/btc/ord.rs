// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::jsonrpc_types::btc::transaction::{hex_to_txid, TxidView};
use crate::jsonrpc_types::{
    BytesView, H256View, IndexerObjectStateView, IndexerStateIDView, MoveStringView,
    ObjectIDVecView, ObjectMetaView, StrView, UnitedAddressView,
};
use anyhow::Result;
use bitcoin::hashes::Hash;
use bitcoin::Txid;
use moveos_types::move_std::string::MoveString;
use moveos_types::state::MoveState;
use moveos_types::state::MoveStructType;
use rooch_types::bitcoin::ord;
use rooch_types::bitcoin::ord::{BitcoinInscriptionID, Inscription};
use rooch_types::indexer::state::ObjectStateFilter;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

pub type BitcoinInscriptionIDStr = StrView<BitcoinInscriptionID>;

impl std::fmt::Display for BitcoinInscriptionIDStr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}i{}", self.0.txid, self.0.index)
    }
}

impl FromStr for BitcoinInscriptionIDStr {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> anyhow::Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('i').collect();
        if parts.len() != 2 {
            return Err(anyhow::anyhow!(
                "Invalid string format for bitcoin inscriptionID"
            ));
        }

        let txid = hex_to_txid(parts[0])?;
        let index = u32::from_str(parts[1])
            .map_err(|_| anyhow::anyhow!("Invalid index for bitcoin inscriptionID"))?;
        Ok(StrView(BitcoinInscriptionID { txid, index }))
    }
}
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize, Eq, JsonSchema)]
pub struct BitcoinInscriptionIDView {
    pub txid: TxidView,
    pub index: u32,
}

impl std::fmt::Display for BitcoinInscriptionIDView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}i{}", self.txid, self.index)
    }
}

impl FromStr for BitcoinInscriptionIDView {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> anyhow::Result<Self, Self::Err> {
        let bitcoin_inscription_id = BitcoinInscriptionIDStr::from_str(s)?;
        Ok(BitcoinInscriptionIDView {
            txid: StrView(bitcoin_inscription_id.0.txid),
            index: bitcoin_inscription_id.0.index,
        })
    }
}

impl From<BitcoinInscriptionIDView> for BitcoinInscriptionID {
    fn from(inscription: BitcoinInscriptionIDView) -> Self {
        BitcoinInscriptionID {
            txid: inscription.txid.into(),
            index: inscription.index,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum InscriptionFilterView {
    /// Query by owner, support rooch address and bitcoin address
    Owner(UnitedAddressView),
    /// Query by inscription id, represent by bitcoin {{txid}i{index}}
    InscriptionId(BitcoinInscriptionIDStr),
    /// Query by object ids.
    ObjectId(ObjectIDVecView),
    /// Query all.
    All,
}

impl InscriptionFilterView {
    pub fn into_global_state_filter(filter: InscriptionFilterView) -> Result<ObjectStateFilter> {
        Ok(match filter {
            InscriptionFilterView::Owner(owner) => ObjectStateFilter::ObjectTypeWithOwner {
                object_type: Inscription::struct_tag(),
                filter_out: false,
                owner: owner.0.rooch_address.into(),
            },
            InscriptionFilterView::InscriptionId(inscription_id) => {
                let obj_id = ord::derive_inscription_id(&inscription_id.0.into());
                ObjectStateFilter::ObjectId(vec![obj_id])
            }
            InscriptionFilterView::ObjectId(object_id_vec_view) => {
                ObjectStateFilter::ObjectId(object_id_vec_view.into())
            }
            InscriptionFilterView::All => ObjectStateFilter::ObjectType(Inscription::struct_tag()),
        })
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct InscriptionIDView {
    pub txid: H256View,
    pub index: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct InscriptionView {
    pub txid: H256View,
    pub bitcoin_txid: TxidView,
    pub index: u32,
    pub offset: StrView<u64>,
    pub sequence_number: u32,
    pub inscription_number: u32,
    pub is_curse: bool,
    pub body: BytesView,
    pub content_encoding: Option<MoveStringView>,
    pub content_type: Option<MoveStringView>,
    pub metadata: BytesView,
    pub metaprotocol: Option<MoveStringView>,
    pub parents: ObjectIDVecView,
    pub pointer: Option<StrView<u64>>,
}

impl From<Inscription> for InscriptionView {
    fn from(inscription: Inscription) -> Self {
        InscriptionView {
            txid: inscription.txid.into(),
            bitcoin_txid: StrView(Txid::from_byte_array(inscription.txid.into_bytes())),
            index: inscription.index,
            offset: inscription.offset.into(),
            sequence_number: inscription.sequence_number,
            inscription_number: inscription.inscription_number,
            is_curse: inscription.is_curse,
            body: StrView(inscription.body),
            content_encoding: Option::<MoveString>::from(inscription.content_encoding).map(StrView),
            content_type: Option::<MoveString>::from(inscription.content_type).map(StrView),
            metadata: StrView(inscription.metadata),
            metaprotocol: Option::<MoveString>::from(inscription.metaprotocol).map(StrView),
            parents: inscription.parents.into(),
            pointer: Option::<u64>::from(inscription.pointer).map(StrView),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct InscriptionStateView {
    #[serde(flatten)]
    pub metadata: ObjectMetaView,
    pub value: InscriptionView,
    #[serde(flatten)]
    pub indexer_id: IndexerStateIDView,
}

impl TryFrom<IndexerObjectStateView> for InscriptionStateView {
    type Error = anyhow::Error;

    fn try_from(state: IndexerObjectStateView) -> Result<Self, Self::Error> {
        let inscription = Inscription::from_bytes(&state.value.0)?;
        let inscription_view = InscriptionView::from(inscription);
        Ok(InscriptionStateView {
            metadata: state.metadata,
            value: inscription_view,
            indexer_id: state.indexer_id,
        })
    }
}

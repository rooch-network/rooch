// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::jsonrpc_types::btc::transaction::{hex_to_txid, TxidView};
use crate::jsonrpc_types::{
    BytesView, H256View, IndexerObjectStateView, IndexerStateIDView, MoveStringView,
    ObjectIDVecView, ObjectMetaView, RoochAddressView, StrView,
};
use anyhow::Result;
use bitcoin::hashes::Hash;
use bitcoin::Txid;
use moveos_types::move_std::string::MoveString;
use moveos_types::state::MoveState;
use moveos_types::{moveos_std::object::ObjectID, state::MoveStructType};
use rooch_types::bitcoin::ord;
use rooch_types::bitcoin::ord::{BitcoinInscriptionID, Inscription, InscriptionID};
use rooch_types::indexer::state::ObjectStateFilter;
use rooch_types::into_address::IntoAddress;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize, Eq, JsonSchema)]
pub struct BitcoinInscriptionIDView {
    pub txid: TxidView,
    pub index: u32,
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
    Owner(RoochAddressView),
    /// Query by inscription id, represent by bitcoin txid and index
    InscriptionId { txid: String, index: u32 },
    /// Query by object id.
    ObjectId(ObjectID),
    /// Query all.
    All,
}

impl InscriptionFilterView {
    pub fn into_global_state_filter(filter: InscriptionFilterView) -> Result<ObjectStateFilter> {
        Ok(match filter {
            InscriptionFilterView::Owner(owner) => ObjectStateFilter::ObjectTypeWithOwner {
                object_type: Inscription::struct_tag(),
                owner: owner.0,
            },
            InscriptionFilterView::InscriptionId { txid, index } => {
                let txid = hex_to_txid(txid.as_str())?;
                let inscription_id = InscriptionID::new(txid.into_address(), index);
                let obj_id = ord::derive_inscription_id(&inscription_id);
                ObjectStateFilter::ObjectId(vec![obj_id])
            }
            InscriptionFilterView::ObjectId(object_id) => {
                ObjectStateFilter::ObjectId(vec![object_id])
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

// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::utxo::BitcoinOutPointView;
use crate::jsonrpc_types::{
    BytesView, H256View, IndexerObjectStateView, IndexerStateIDView, MoveStringView,
    ObjectIDVecView, ObjectMetaView, StrView, UnitedAddressView,
};
use anyhow::Result;
use moveos_types::move_std::string::MoveString;
use moveos_types::state::MoveState;
use moveos_types::state::MoveStructType;
use rooch_types::bitcoin::ord::{self, SatPoint};
use rooch_types::bitcoin::ord::{BitcoinInscriptionID, Inscription, InscriptionID};
use rooch_types::indexer::state::ObjectStateFilter;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::str::FromStr;

pub type BitcoinInscriptionIDView = StrView<BitcoinInscriptionID>;

impl FromStr for BitcoinInscriptionIDView {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(StrView(BitcoinInscriptionID::from_str(s)?))
    }
}

impl Display for BitcoinInscriptionIDView {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<InscriptionID> for BitcoinInscriptionIDView {
    fn from(inscription_id: InscriptionID) -> Self {
        StrView(BitcoinInscriptionID::from(inscription_id))
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum InscriptionFilterView {
    /// Query by owner, support rooch address and bitcoin address
    Owner(UnitedAddressView),
    /// Query by inscription id, represent by bitcoin {{txid}i{index}}
    InscriptionId(BitcoinInscriptionIDView),
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
pub struct SatPointView {
    pub output: BitcoinOutPointView,
    pub offset: StrView<u64>,
}

impl From<SatPoint> for SatPointView {
    fn from(sat_point: SatPoint) -> Self {
        SatPointView {
            output: sat_point.outpoint.into(),
            offset: StrView(sat_point.offset),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct InscriptionView {
    pub id: BitcoinInscriptionIDView,
    pub location: SatPointView,
    pub sequence_number: u32,
    pub inscription_number: u32,
    pub is_curse: bool,
    pub charms: u16,
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
            id: inscription.id.into(),
            location: inscription.location.into(),
            sequence_number: inscription.sequence_number,
            inscription_number: inscription.inscription_number,
            is_curse: inscription.is_curse,
            charms: inscription.charms,
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

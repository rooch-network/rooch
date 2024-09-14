// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::utxo::OutPointView;
use crate::jsonrpc_types::ObjectStateView;
use crate::jsonrpc_types::{
    BytesView, IndexerObjectStateView, IndexerStateIDView, MoveStringView, ObjectIDVecView,
    ObjectMetaView, StrView, UnitedAddressView,
};
use anyhow::Result;
use moveos_types::move_std::option::MoveOption;
use moveos_types::move_std::string::MoveString;
use moveos_types::state::MoveState;
use moveos_types::state::MoveStructType;
use rooch_types::bitcoin::ord::{self, SatPoint};
use rooch_types::bitcoin::ord::{Inscription, InscriptionID};
use rooch_types::indexer::state::ObjectStateFilter;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::str::FromStr;

pub type InscriptionIDView = StrView<InscriptionID>;

impl FromStr for InscriptionIDView {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(StrView(InscriptionID::from_str(s)?))
    }
}

impl Display for InscriptionIDView {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<InscriptionIDView> for InscriptionID {
    fn from(view: InscriptionIDView) -> Self {
        view.0
    }
}


#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum InscriptionFilterView {
    /// Query by owner, support rooch address and bitcoin address
    Owner(UnitedAddressView),
    /// Query by inscription id, represent by bitcoin {{txid}i{index}}
    InscriptionId(InscriptionIDView),
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
                let obj_id = ord::derive_inscription_id(&inscription_id.0);
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
pub struct SatPointView {
    pub output: OutPointView,
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

impl From<SatPointView> for SatPoint {
    fn from(view: SatPointView) -> Self {
        SatPoint {
            outpoint: view.output.into(),
            offset: view.offset.0,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct InscriptionView {
    pub id: InscriptionIDView,
    pub location: SatPointView,
    pub sequence_number: u32,
    pub inscription_number: i32,
    pub charms: u16,
    pub body: BytesView,
    pub content_encoding: Option<MoveStringView>,
    pub content_type: Option<MoveStringView>,
    pub metadata: BytesView,
    pub metaprotocol: Option<MoveStringView>,
    pub parents: Vec<InscriptionIDView>,
    pub pointer: Option<StrView<u64>>,
}

impl From<Inscription> for InscriptionView {
    fn from(inscription: Inscription) -> Self {
        let inscription_number = inscription.inscription_number();
        InscriptionView {
            id: inscription.id.into(),
            location: inscription.location.into(),
            sequence_number: inscription.sequence_number,
            inscription_number,
            charms: inscription.charms,
            body: StrView(inscription.body),
            content_encoding: Option::<MoveString>::from(inscription.content_encoding).map(StrView),
            content_type: Option::<MoveString>::from(inscription.content_type).map(StrView),
            metadata: StrView(inscription.metadata),
            metaprotocol: Option::<MoveString>::from(inscription.metaprotocol).map(StrView),
            parents: inscription.parents.into_iter().map(Into::into).collect(),
            pointer: Option::<u64>::from(inscription.pointer).map(StrView),
        }
    }
}

impl From<InscriptionView> for Inscription {
    fn from(view: InscriptionView) -> Self {
        let inscription_number = view.inscription_number;
        Inscription {
            id: view.id.0,
            location: view.location.into(),
            sequence_number: view.sequence_number,
            inscription_number: inscription_number.abs() as u32,
            is_cursed: inscription_number < 0,
            charms: view.charms,
            body: view.body.0,
            content_encoding: view.content_encoding.map(|v| v.0).into(),
            content_type: view.content_type.map(|v| v.0).into(),
            metadata: view.metadata.0,
            metaprotocol: view.metaprotocol.map(|v| v.0).into(),
            parents: view.parents.into_iter().map(Into::into).collect(),
            pointer: view.pointer.map(|v| v.0).into(),
            rune: MoveOption::none(),
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

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct InscriptionObjectView {
    #[serde(flatten)]
    pub metadata: ObjectMetaView,
    pub value: InscriptionView, 
}


impl InscriptionObjectView {

    pub fn location(&self) -> SatPoint {
        self.value.location.clone().into()
    }

}

impl From<InscriptionStateView> for InscriptionObjectView {
    fn from(state: InscriptionStateView) -> Self {
        InscriptionObjectView {
            metadata: state.metadata,
            value: state.value,
        }
    }
}

impl TryFrom<ObjectStateView> for InscriptionObjectView {
    type Error = anyhow::Error;

    fn try_from(state: ObjectStateView) -> Result<Self, Self::Error> {
        let inscription = Inscription::from_bytes(&state.value.0)?;
        let inscription_view = InscriptionView::from(inscription);
        Ok(InscriptionObjectView {
            metadata: state.metadata,
            value: inscription_view,
        })
    }
}
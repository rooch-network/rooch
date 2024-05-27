// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::jsonrpc_types::address::BitcoinAddressView;
use crate::jsonrpc_types::btc::transaction::{hex_to_txid, TxidView};
use crate::jsonrpc_types::{
    BytesView, H256View, MoveStringView, RoochAddressView, StrView, StructTagView,
};
use anyhow::Result;
use bitcoin::hashes::Hash;
use bitcoin::Txid;

use moveos_types::move_std::string::MoveString;
use moveos_types::{moveos_std::object::ObjectID, state::MoveStructType};
use rooch_types::address::RoochAddress;
use rooch_types::bitcoin::ord;
use rooch_types::bitcoin::ord::{
    BitcoinInscriptionID, Inscription, InscriptionID, InscriptionState,
};
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
    /// Query by owner, represent by bitcoin address
    Owner(BitcoinAddressView),
    /// Query by inscription id, represent by bitcoin txid and index
    InscriptionId { txid: String, index: u32 },
    /// Query by object id.
    ObjectId(ObjectID),
    /// Query all.
    All,
}

impl InscriptionFilterView {
    pub fn into_global_state_filter(
        filter: InscriptionFilterView,
        resolve_address: RoochAddress,
    ) -> Result<ObjectStateFilter> {
        Ok(match filter {
            InscriptionFilterView::Owner(_owner) => ObjectStateFilter::ObjectTypeWithOwner {
                object_type: Inscription::struct_tag(),
                owner: resolve_address,
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
    pub offset: u64,
    pub body: BytesView,
    pub content_encoding: Option<MoveStringView>,
    pub content_type: Option<MoveStringView>,
    pub metadata: BytesView,
    pub metaprotocol: Option<MoveStringView>,
    pub parent: Option<ObjectID>,
    pub pointer: Option<u64>,
}

impl From<Inscription> for InscriptionView {
    fn from(inscription: Inscription) -> Self {
        InscriptionView {
            txid: inscription.txid.into(),
            bitcoin_txid: StrView(Txid::from_byte_array(inscription.txid.into_bytes())),
            index: inscription.index,
            offset: inscription.offset,
            body: StrView(inscription.body),
            content_encoding: Option::<MoveString>::from(inscription.content_encoding).map(StrView),
            content_type: Option::<MoveString>::from(inscription.content_type).map(StrView),
            metadata: StrView(inscription.metadata),
            metaprotocol: Option::<MoveString>::from(inscription.metaprotocol).map(StrView),
            parent: Option::<ObjectID>::from(inscription.parent),
            pointer: Option::<u64>::from(inscription.pointer),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct InscriptionStateView {
    pub object_id: ObjectID,
    pub owner: RoochAddressView,
    pub owner_bitcoin_address: Option<String>,
    pub flag: u8,
    pub value: InscriptionView,
    pub object_type: StructTagView,
    pub tx_order: u64,
    pub state_index: u64,
    pub created_at: u64,
    pub updated_at: u64,
}

impl InscriptionStateView {
    pub fn try_new_from_inscription_state(
        inscription: InscriptionState,
        network: u8,
    ) -> Result<InscriptionStateView, anyhow::Error> {
        let owner_bitcoin_address = match inscription.owner_bitcoin_address {
            Some(baddress) => Some(baddress.format(network)?),
            None => None,
        };
        Ok(InscriptionStateView {
            object_id: inscription.object_id,
            owner: inscription.owner.into(),
            owner_bitcoin_address,
            flag: inscription.flag,
            value: inscription.value.into(),
            object_type: inscription.object_type.into(),
            tx_order: inscription.tx_order,
            state_index: inscription.state_index,
            created_at: inscription.created_at,
            updated_at: inscription.updated_at,
        })
    }
}

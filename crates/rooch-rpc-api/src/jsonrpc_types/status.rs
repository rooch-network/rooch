// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::StrView;
use accumulator::accumulator_info::AccumulatorInfo;
use bitcoin::BlockHash;
use moveos_types::h256::H256;
use moveos_types::{startup_info::StartupInfo, state::ObjectState};
use rooch_types::da::status::DAServerStatus;
use rooch_types::into_address::FromAddress;
use rooch_types::{
    bitcoin::types::BlockHeightHash, sequencer::SequencerInfo, service_status::ServiceStatus,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BlockHeightHashView {
    pub block_height: StrView<u64>,
    pub block_hash: String,
}

impl From<BlockHeightHash> for BlockHeightHashView {
    fn from(info: BlockHeightHash) -> Self {
        BlockHeightHashView {
            block_height: StrView::from(info.block_height),
            block_hash: BlockHash::from_address(info.block_hash).to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BitcoinStatus {
    pub confirmed_block: Option<BlockHeightHashView>,
    pub pending_block: Option<BlockHeightHashView>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AccumulatorInfoView {
    pub accumulator_root: StrView<H256>,
    pub frozen_subtree_roots: Vec<StrView<H256>>,
    pub num_leaves: StrView<u64>,
    pub num_nodes: StrView<u64>,
}

impl From<AccumulatorInfo> for AccumulatorInfoView {
    fn from(info: AccumulatorInfo) -> Self {
        AccumulatorInfoView {
            accumulator_root: StrView::from(info.accumulator_root),
            frozen_subtree_roots: info
                .frozen_subtree_roots
                .into_iter()
                .map(StrView::from)
                .collect(),
            num_leaves: StrView::from(info.num_leaves),
            num_nodes: StrView::from(info.num_nodes),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SequencerInfoView {
    pub last_order: StrView<u64>,
    pub last_accumulator_info: AccumulatorInfoView,
}

impl From<SequencerInfo> for SequencerInfoView {
    fn from(info: SequencerInfo) -> Self {
        SequencerInfoView {
            last_order: StrView::from(info.last_order),
            last_accumulator_info: AccumulatorInfoView::from(info.last_accumulator_info),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DAInfoView {
    pub last_block_number: Option<StrView<u128>>,
    pub last_tx_order: Option<StrView<u64>>,
    pub last_block_update_time: Option<StrView<u64>>,
    pub last_avail_block_number: Option<StrView<u128>>,
    pub last_avail_tx_order: Option<StrView<u64>>,
    pub last_avail_block_update_time: Option<StrView<u64>>,
    pub avail_backends: Vec<(String, StrView<u128>)>,
}

impl From<DAServerStatus> for DAInfoView {
    fn from(info: DAServerStatus) -> Self {
        DAInfoView {
            last_block_number: info.last_block_number.map(Into::into),
            last_tx_order: info.last_tx_order.map(Into::into),
            last_block_update_time: info.last_block_update_time.map(Into::into),
            last_avail_block_number: info.last_avail_block_number.map(Into::into),
            last_avail_tx_order: info.last_avail_tx_order.map(Into::into),
            last_avail_block_update_time: info.last_avail_block_update_time.map(Into::into),
            avail_backends: info
                .avail_backends
                .into_iter()
                .map(|(k, v)| (k, v.into()))
                .collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RootStateView {
    pub state_root: StrView<H256>,
    pub size: StrView<u64>,
}

impl From<StartupInfo> for RootStateView {
    fn from(info: StartupInfo) -> Self {
        RootStateView {
            state_root: StrView::from(info.state_root),
            size: StrView::from(info.size),
        }
    }
}

impl From<ObjectState> for RootStateView {
    fn from(info: ObjectState) -> Self {
        RootStateView {
            state_root: StrView::from(info.metadata.state_root()),
            size: StrView::from(info.metadata.size),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RoochStatus {
    pub sequencer_info: SequencerInfoView,
    pub da_info: DAInfoView,
    pub root_state: RootStateView,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Status {
    /// The status of the rpc service
    pub service_status: ServiceStatus,
    /// The status of the Rooch chain
    pub rooch_status: RoochStatus,
    /// The status of the Bitcoin chain
    pub bitcoin_status: BitcoinStatus,
}

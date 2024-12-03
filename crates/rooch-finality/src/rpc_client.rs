// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use tonic::{transport::Channel, Request};

use crate::proto::proto::finality_gadget_client::FinalityGadgetClient;
use crate::proto::proto::{
    BlockInfo, QueryBlockRangeBabylonFinalizedRequest, QueryBtcStakingActivatedTimestampRequest,
    QueryIsBlockBabylonFinalizedRequest, QueryIsBlockFinalizedByHashRequest,
    QueryIsBlockFinalizedByHeightRequest, QueryLatestFinalizedBlockRequest,
};
use anyhow::{anyhow, Result};
use rooch_types::finality_block::Block;

pub struct FinalityGadgetGrpcClient {
    client: FinalityGadgetClient<Channel>,
}

impl FinalityGadgetGrpcClient {
    pub async fn new(remote_addr: String) -> Result<Self> {
        let channel = Channel::from_shared(remote_addr)?
            .connect()
            .await
            .map_err(|e| anyhow!(format!("new error: {:?}", e)))?;

        let client = FinalityGadgetClient::new(channel);

        Ok(Self { client })
    }

    pub async fn query_is_block_babylon_finalized(&mut self, block: &Block) -> Result<bool> {
        let req = Request::new(QueryIsBlockBabylonFinalizedRequest {
            block: Some(BlockInfo {
                block_hash: block.block_hash.clone(),
                block_height: block.block_height,
                block_timestamp: block.block_timestamp,
            }),
        });

        let response = self
            .client
            .query_is_block_babylon_finalized(req)
            .await
            .map_err(|e| anyhow!(format!("query_is_block_babylon_finalized error: {:?}", e)))?;
        Ok(response.into_inner().is_finalized)
    }

    pub async fn query_block_range_babylon_finalized(
        &mut self,
        blocks: &[Block],
    ) -> Result<Option<u64>> {
        let block_infos: Vec<BlockInfo> = blocks
            .iter()
            .map(|block| BlockInfo {
                block_hash: block.block_hash.clone(),
                block_height: block.block_height,
                block_timestamp: block.block_timestamp,
            })
            .collect();

        let req = Request::new(QueryBlockRangeBabylonFinalizedRequest {
            blocks: block_infos,
        });

        let response = self
            .client
            .query_block_range_babylon_finalized(req)
            .await
            .map_err(|e| {
                anyhow!(format!(
                    "query_block_range_babylon_finalized error: {:?}",
                    e
                ))
            })?;
        let height = response.into_inner().last_finalized_block_height;

        if height == 0 {
            Ok(None)
        } else {
            Ok(Some(height))
        }
    }

    pub async fn query_btc_staking_activated_timestamp(&mut self) -> Result<u64> {
        let req = Request::new(QueryBtcStakingActivatedTimestampRequest {});

        let response = self
            .client
            .query_btc_staking_activated_timestamp(req)
            .await
            .map_err(|e| {
                anyhow!(format!(
                    "query_btc_staking_activated_timestamp error: {:?}",
                    e
                ))
            })?;
        Ok(response.into_inner().activated_timestamp)
    }

    pub async fn query_is_block_finalized_by_height(&mut self, height: u64) -> Result<bool> {
        let req = Request::new(QueryIsBlockFinalizedByHeightRequest {
            block_height: height,
        });

        let response = self
            .client
            .query_is_block_finalized_by_height(req)
            .await
            .map_err(|e| anyhow!(format!("query_is_block_finalized_by_height error: {:?}", e)))?;
        Ok(response.into_inner().is_finalized)
    }

    pub async fn query_is_block_finalized_by_hash(&mut self, hash: String) -> Result<bool> {
        let req = Request::new(QueryIsBlockFinalizedByHashRequest { block_hash: hash });

        let response = self
            .client
            .query_is_block_finalized_by_hash(req)
            .await
            .map_err(|e| anyhow!(format!("query_is_block_finalized_by_hash error: {:?}", e)))?;
        Ok(response.into_inner().is_finalized)
    }

    pub async fn query_latest_finalized_block(&mut self) -> Result<Block> {
        let req = Request::new(QueryLatestFinalizedBlockRequest {});

        let response = self
            .client
            .query_latest_finalized_block(req)
            .await
            .map_err(|e| anyhow!(format!("query_latest_finalized_block error: {:?}", e)))?;
        let block = response.into_inner().block.unwrap();

        Ok(Block {
            block_hash: block.block_hash,
            block_height: block.block_height,
            block_timestamp: block.block_timestamp,
        })
    }
}

impl Drop for FinalityGadgetGrpcClient {
    fn drop(&mut self) {
        // Channel cleanup is handled automatically by tonic
    }
}

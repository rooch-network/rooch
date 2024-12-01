// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use rooch_types::finality_block::Block;
// This should be updated to match the gRPC interface of the Babylon finality gadget client
// https://github.com/babylonlabs-io/finality-gadget

/// Trait defining the interface for the Babylon finality gadget client
pub trait FinalityGadgetClient {
    /// Checks if the given L2 block is finalized by the Babylon finality gadget
    fn query_is_block_babylon_finalized(&self, block: &Block) -> Result<bool>;

    /// Searches for a row of consecutive finalized blocks in the block range
    fn query_block_range_babylon_finalized(&self, blocks: &[Block]) -> Result<Option<u64>>;

    /// Returns the timestamp when the BTC staking is activated
    fn query_btc_staking_activated_timestamp(&self) -> Result<u64>;

    /// Returns the btc finalization status of a block at given height by querying the local db
    fn query_is_block_finalized_by_height(&self, height: u64) -> Result<bool>;

    /// Returns the btc finalization status of a block at given hash by querying the local db
    fn query_is_block_finalized_by_hash(&self, hash: String) -> Result<bool>;

    /// Returns the latest finalized block by querying the local db
    fn query_latest_finalized_block(&self) -> Result<Block>;

    /// Closes the client
    fn close(&self) -> Result<()>;
}

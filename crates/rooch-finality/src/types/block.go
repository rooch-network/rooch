package types

type Block struct {
	BlockHash      string `json:"block_hash" description:"block hash"`
	BlockHeight    uint64 `json:"block_height" description:"block height"`
	BlockTimestamp uint64 `json:"block_timestamp" description:"block timestamp"`
}

type ChainSyncStatus struct {
	LatestBlockHeight               uint64 `json:"latest_block"`
	LatestBtcFinalizedBlockHeight   uint64 `json:"latest_btc_finalized_block"`
	EarliestBtcFinalizedBlockHeight uint64 `json:"earliest_btc_finalized_block"`
	LatestEthFinalizedBlockHeight   uint64 `json:"latest_eth_finalized_block"`
}

package types

type TransactionInfo struct {
	TxHash           string         `json:"txHash"`
	BlockHash        string         `json:"blockHash"`
	Status           FinalityStatus `json:"status"`
	BlockTimestamp   uint64         `json:"blockTimestamp"`
	BlockHeight      uint64         `json:"blockHeight"`
	BabylonFinalized bool           `json:"babylonFinalized"`
}

type FinalityStatus string

const (
	FinalityStatusPending          FinalityStatus = "pending"
	FinalityStatusUnsafe           FinalityStatus = "unsafe"
	FinalityStatusBitcoinFinalized FinalityStatus = "btc finalized"
	FinalityStatusSafe             FinalityStatus = "safe"
	FinalityStatusFinalized        FinalityStatus = "finalized"
)

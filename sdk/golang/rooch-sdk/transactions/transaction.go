// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

package transactions

import (
	"errors"

	"github.com/rooch-network/rooch/sdk/golang/rooch-sdk/bcs"
	"github.com/rooch-network/rooch/sdk/golang/rooch-sdk/crypto"
	"github.com/rooch-network/rooch/sdk/golang/rooch-sdk/types"
)

// Transaction represents a Rooch transaction
type Transaction struct {
	data *TransactionData
	auth *crypto.Authenticator
	info *string
}

// CallFunction initializes the transaction with function call data
func (t *Transaction) CallFunction(input CallFunctionArgs, info *string) {
	t.info = info
	t.data = NewTransactionData(NewMoveActionCallFunction(input))
}

// GetInfo returns the transaction info
func (t *Transaction) GetInfo() *string {
	return t.info
}

// SetSender sets the sender address for the transaction
func (t *Transaction) SetSender(input types.Address) {
	t.getData().Sender = input
}

// SetAuth sets the authenticator for the transaction
func (t *Transaction) SetAuth(input *crypto.Authenticator) {
	t.auth = input
}

// SetChainId sets the chain ID for the transaction
func (t *Transaction) SetChainId(input uint64) {
	t.getData().ChainId = input
}

// SetSeqNumber sets the sequence number for the transaction
func (t *Transaction) SetSeqNumber(input uint64) {
	t.getData().SequenceNumber = input
}

// HashData returns the hash of the transaction data
func (t *Transaction) HashData() []byte {
	return t.getData().Hash()
}

// Encode serializes the transaction using BCS
func (t *Transaction) Encode() ([]byte, error) {
	if t.data == nil || t.auth == nil {
		return nil, errors.New("transaction data or auth is not initialized")
	}
	
	data, err := t.data.Encode()
	if err != nil {
		return nil, err
	}
	
	auth, err := t.auth.Encode()
	if err != nil {
		return nil, err
	}
	
	return bcs.SerializeRoochTransaction(data, auth)
}

// getData returns the transaction data after validation
func (t *Transaction) getData() *TransactionData {
	t.isValid()
	return t.data
}

// isValid checks if the transaction data is initialized
func (t *Transaction) isValid() error {
	if t.data == nil {
		return errors.New("transaction data is not initialized. Call action first")
	}
	return nil
} 
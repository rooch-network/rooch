package transactions

import (
	"testing"

	"github.com/rooch-network/rooch/sdk/golang/bcs"
	"github.com/rooch-network/rooch/sdk/golang/crypto"
	"github.com/rooch-network/rooch/sdk/golang/keypairs"
	"github.com/stretchr/testify/assert"
)

func TestTransactionVerification(t *testing.T) {
	// Create a new keypair
	signer, err := keypairs.NewSecp256k1Keypair()
	assert.NoError(t, err)

	// Create and configure transaction
	tx := NewTransaction()
	tx.CallFunction(&FunctionCall{
		Target: "0x3::empty::empty_with_signer",
	})

	tx.SetSender(signer.GetRoochAddress().ToHexAddress())
	tx.SetSeqNumber(0)
	tx.SetChainId(4)

	// Sign the transaction
	auth, err := signer.SignTransaction(tx)
	assert.NoError(t, err)

	// Parse the auth payload
	payload, err := bcs.ParseBitcoinAuthPayload(auth.Payload)
	assert.NoError(t, err)

	// Create bitcoin message
	bitcoinMessage := crypto.NewBitcoinSignMessage(tx.HashData(), []byte(payload.MessageInfo))

	// Verify signature
	result, err := signer.GetPublicKey().Verify(bitcoinMessage.Hash(), payload.Signature)
	assert.NoError(t, err)
	assert.True(t, result)
} 
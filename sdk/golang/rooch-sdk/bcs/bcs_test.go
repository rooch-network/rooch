package bcs

import (
	"testing"

	"github.com/rooch-network/rooch/sdk/golang/keypairs"
	"github.com/rooch-network/rooch/sdk/golang/utils"
	"github.com/stretchr/testify/assert"
)

func TestBCS(t *testing.T) {
	t.Run("Address", func(t *testing.T) {
		// Test Ed25519 address serialization
		ed25519Keypair := keypairs.NewEd25519Keypair()
		roochAddress := ed25519Keypair.GetRoochAddress()

		bcs1 := BCS.Address.Serialize(roochAddress).ToBytes()
		bcs2 := BCS.Address.Serialize(roochAddress.ToHexAddress()).ToBytes()
		bcs3 := BCS.Address.Serialize(roochAddress.ToBech32Address()).ToBytes()

		assert.True(t, utils.BytesEqual(bcs1, bcs2))
		assert.True(t, utils.BytesEqual(bcs1, bcs3))

		// Test Secp256k1 address serialization
		secp256k1Keypair := keypairs.NewSecp256k1Keypair()
		bitcoinAddress := secp256k1Keypair.GetBitcoinAddress()

		bcs4 := BCS.Address.Serialize(bitcoinAddress).ToBytes()
		bcs5 := BCS.Address.Serialize(bitcoinAddress.GenRoochAddress()).ToBytes()

		assert.True(t, utils.BytesEqual(bcs4, bcs5))
	})
} 
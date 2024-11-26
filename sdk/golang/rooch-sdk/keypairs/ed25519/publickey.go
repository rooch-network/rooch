package ed25519

import (
	"crypto/ed25519"
	"encoding/base64"
	"errors"
	"github.com/rooch-network/rooch/sdk/golang/rooch-sdk/address"
	"github.com/rooch-network/rooch/sdk/golang/rooch-sdk/crypto"
	"github.com/rooch-network/rooch/sdk/golang/rooch-sdk/utils"
)

const PUBLIC_KEY_SIZE = 32

// Ed25519PublicKey represents an Ed25519 public key
type Ed25519PublicKey struct {
	data []byte
}

// NewEd25519PublicKey creates a new Ed25519PublicKey object
// value can be either a base64 encoded string or a byte slice
func NewEd25519PublicKey(value interface{}) (*Ed25519PublicKey, error) {
	var data []byte

	switch v := value.(type) {
	case string:
		var err error
		data, err = base64.StdEncoding.DecodeString(v)
		if err != nil {
			return nil, err
		}
	case []byte:
		data = v
	default:
		return nil, errors.New("invalid public key input type")
	}

	if len(data) != PUBLIC_KEY_SIZE {
		return nil, errors.New("invalid public key size")
	}

	return &Ed25519PublicKey{
		data: data,
	}, nil
}

// Equals checks if two Ed25519 public keys are equal
func (pk *Ed25519PublicKey) Equals(other *Ed25519PublicKey) bool {
	if pk == nil || other == nil {
		return false
	}
	return crypto.BytesEqual(pk.data, other.data)
}

// ToBytes returns the byte array representation of the Ed25519 public key
func (pk *Ed25519PublicKey) ToBytes() []byte {
	return pk.data
}

// Flag returns the signature scheme flag for Ed25519
func (pk *Ed25519PublicKey) Flag() uint8 {
	return crypto.ED25519_FLAG
}

// Verify verifies that the signature is valid for the provided message
func (pk *Ed25519PublicKey) Verify(message, signature []byte) bool {
	return ed25519.Verify(pk.data, message, signature)
}

// ToAddress returns the Rooch address associated with this Ed25519 public key
func (pk *Ed25519PublicKey) ToAddress() *address.RoochAddress {
	tmp := make([]byte, PUBLIC_KEY_SIZE+1)
	tmp[0] = crypto.ED25519_FLAG
	copy(tmp[1:], pk.data)

	hash := utils.Blake2b(tmp, 32)
	addressBytes := hash[:address.ROOCH_ADDRESS_LENGTH*2]
	return address.NewRoochAddress(addressBytes)
} 
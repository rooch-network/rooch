package secp256k1

import (
	"encoding/base64"
	"encoding/hex"
	"errors"
	"github.com/decred/dcrd/dcrec/secp256k1/v4"
	"github.com/rooch-network/rooch/sdk/golang/address"
	"github.com/rooch-network/rooch/sdk/golang/crypto"
	"github.com/rooch-network/rooch/sdk/golang/utils"
)

const (
	SchnorrPublicKeySize = 32
)

// Secp256k1PublicKey represents a Secp256k1 public key
type Secp256k1PublicKey struct {
	data []byte
}

// NewSecp256k1PublicKey creates a new Secp256k1PublicKey object
func NewSecp256k1PublicKey(value interface{}) (*Secp256k1PublicKey, error) {
	var data []byte

	switch v := value.(type) {
	case string:
		// Assume base64 encoded string
		decoded, err := base64.StdEncoding.DecodeString(v)
		if err != nil {
			return nil, err
		}
		data = decoded
	case []byte:
		data = v
	default:
		return nil, errors.New("unsupported public key input type")
	}

	if len(data) != SchnorrPublicKeySize && len(data) != 33 {
		return nil, errors.New("invalid public key input size")
	}

	return &Secp256k1PublicKey{
		data: data,
	}, nil
}

// Equals checks if two Secp256k1 public keys are equal
func (pk *Secp256k1PublicKey) Equals(other *Secp256k1PublicKey) bool {
	return crypto.BytesEqual(pk.data, other.data)
}

// ToBytes returns the byte array representation of the Secp256k1 public key
func (pk *Secp256k1PublicKey) ToBytes() []byte {
	return pk.data
}

// String returns the hex string representation of the public key
func (pk *Secp256k1PublicKey) String() string {
	return hex.EncodeToString(pk.data)
}

// ToAddress returns the Bitcoin address associated with this Secp256k1 public key
func (pk *Secp256k1PublicKey) ToAddress() *address.AddressView {
	return address.NewAddressView(pk.data)
}

// ToAddressWith returns the Bitcoin address with specified network type
func (pk *Secp256k1PublicKey) ToAddressWith(network address.BitcoinNetworkType) *address.AddressView {
	return address.NewAddressViewWithNetwork(pk.data, network)
}

// Flag returns the signature scheme flag for Secp256k1
func (pk *Secp256k1PublicKey) Flag() int {
	return crypto.SignatureSchemeToFlag["Secp256k1"]
}

// Verify verifies that the signature is valid for the provided message
func (pk *Secp256k1PublicKey) Verify(message []byte, signature []byte) (bool, error) {
	// Create hash of the message
	messageHash := utils.Sha256(message)
	
	// Parse the signature
	sig, err := secp256k1.SignatureFromBytes(signature)
	if err != nil {
		return false, err
	}

	// Parse the public key
	pubKey, err := secp256k1.ParsePubKey(pk.data)
	if err != nil {
		return false, err
	}

	return sig.Verify(messageHash, pubKey), nil
} 
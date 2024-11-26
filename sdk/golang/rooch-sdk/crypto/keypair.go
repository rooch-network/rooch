package crypto

import (
	"errors"
	"github.com/btcsuite/btcutil/bech32"
)

const (
	PrivateKeySize       = 32
	LegacyPrivateKeySize = 64
	RoochSecretKeyPrefix = "roochsecretkey"
)

// ParsedKeypair represents a parsed keypair with its signature scheme and secret key
type ParsedKeypair struct {
	Schema    SignatureScheme
	SecretKey []byte
}

// Keypair is an abstract base struct that extends Signer
type Keypair struct {
	Signer
}

// GetSecretKey returns the Bech32 secret key string for this keypair
func (k *Keypair) GetSecretKey() string {
	// This is abstract and should be implemented by concrete types
	panic("GetSecretKey must be implemented by concrete types")
}

// DecodeRoochSecretKey decodes a Bech32 encoded secret key string
func DecodeRoochSecretKey(value string) (*ParsedKeypair, error) {
	// Decode the Bech32 string
	prefix, words, err := bech32.Decode(value)
	if err != nil {
		return nil, err
	}

	if prefix != RoochSecretKeyPrefix {
		return nil, errors.New("invalid private key prefix")
	}

	// Convert words to bytes
	extendedSecretKey, err := bech32.ConvertBits(words, 5, 8, false)
	if err != nil {
		return nil, err
	}

	// Extract flag and secret key
	if len(extendedSecretKey) < 2 {
		return nil, errors.New("invalid secret key length")
	}

	flag := extendedSecretKey[0]
	secretKey := extendedSecretKey[1:]

	// Get signature scheme from flag
	scheme, ok := SignatureFlagToScheme[flag]
	if !ok {
		return nil, errors.New("invalid signature scheme flag")
	}

	return &ParsedKeypair{
		Schema:    scheme,
		SecretKey: secretKey,
	}, nil
}

// EncodeRoochSecretKey encodes a private key and signature scheme to a Bech32 string
func EncodeRoochSecretKey(bytes []byte, scheme SignatureScheme) (string, error) {
	if len(bytes) != PrivateKeySize {
		return "", errors.New("invalid bytes length")
	}

	flag, ok := SignatureSchemeToFlag[scheme]
	if !ok {
		return "", errors.New("invalid signature scheme")
	}

	// Combine flag and private key bytes
	privKeyBytes := make([]byte, len(bytes)+1)
	privKeyBytes[0] = flag
	copy(privKeyBytes[1:], bytes)

	// Encode the combined bytes to Bech32
	encoded, err := bech32.Encode(RoochSecretKeyPrefix, privKeyBytes)
	if err != nil {
		return "", err
	}

	return encoded, nil
} 
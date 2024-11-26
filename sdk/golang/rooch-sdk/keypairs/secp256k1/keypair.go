package secp256k1

import (
	"crypto/sha256"
	"encoding/hex"
	"errors"
	"github.com/btcsuite/btcd/btcec/v2"
	"github.com/btcsuite/btcd/btcec/v2/schnorr"
	"github.com/tyler-smith/go-bip32"
	"github.com/tyler-smith/go-bip39"
)

const DefaultSecp256k1DerivationPath = "m/54'/784'/0'/0/0"

// Secp256k1KeypairData represents the keypair data structure
type Secp256k1KeypairData struct {
	PublicKey []byte
	SecretKey []byte
}

// Secp256k1Keypair represents a Secp256k1 keypair
type Secp256k1Keypair struct {
	keypair Secp256k1KeypairData
}

// NewSecp256k1Keypair creates a new keypair instance
func NewSecp256k1Keypair(keypair *Secp256k1KeypairData) (*Secp256k1Keypair, error) {
	if keypair != nil {
		return &Secp256k1Keypair{keypair: *keypair}, nil
	}

	// Generate random keypair
	privateKey, err := btcec.NewPrivateKey()
	if err != nil {
		return nil, err
	}

	return &Secp256k1Keypair{
		keypair: Secp256k1KeypairData{
			PublicKey: privateKey.PubKey().SerializeCompressed(),
			SecretKey: privateKey.Serialize(),
		},
	}, nil
}

// Generate generates a new random keypair
func Generate() (*Secp256k1Keypair, error) {
	return NewSecp256k1Keypair(nil)
}

// FromSecretKey creates a keypair from a raw secret key byte array
func FromSecretKey(secretKey []byte, skipValidation bool) (*Secp256k1Keypair, error) {
	privateKey, publicKey := btcec.PrivKeyFromBytes(secretKey)

	if !skipValidation {
		// Perform validation similar to TypeScript implementation
		msg := []byte("rooch validation")
		hash := sha256.Sum256(msg)
		
		signature, err := privateKey.Sign(hash[:])
		if err != nil {
			return nil, err
		}

		if !signature.Verify(hash[:], publicKey) {
			return nil, errors.New("provided secretKey is invalid")
		}
	}

	return &Secp256k1Keypair{
		keypair: Secp256k1KeypairData{
			PublicKey: publicKey.SerializeCompressed(),
			SecretKey: privateKey.Serialize(),
		},
	}, nil
}

// FromSeed generates a keypair from a 32 byte seed
func FromSeed(seed []byte) (*Secp256k1Keypair, error) {
	privateKey, _ := btcec.PrivKeyFromBytes(seed)
	
	return &Secp256k1Keypair{
		keypair: Secp256k1KeypairData{
			PublicKey: privateKey.PubKey().SerializeCompressed(),
			SecretKey: privateKey.Serialize(),
		},
	}, nil
}

// GetPublicKey returns the public key
func (kp *Secp256k1Keypair) GetPublicKey() []byte {
	return kp.keypair.PublicKey
}

// GetSchnorrPublicKey returns the Schnorr public key
func (kp *Secp256k1Keypair) GetSchnorrPublicKey() []byte {
	privateKey, _ := btcec.PrivKeyFromBytes(kp.keypair.SecretKey)
	return schnorr.SerializePubKey(privateKey.PubKey())
}

// GetSecretKey returns the secret key
func (kp *Secp256k1Keypair) GetSecretKey() []byte {
	return kp.keypair.SecretKey
}

// Sign signs the provided data
func (kp *Secp256k1Keypair) Sign(input []byte) ([]byte, error) {
	hash := sha256.Sum256(input)
	privateKey, _ := btcec.PrivKeyFromBytes(kp.keypair.SecretKey)
	
	signature, err := privateKey.Sign(hash[:])
	if err != nil {
		return nil, err
	}
	
	return signature.Serialize(), nil
}

// DeriveKeypair derives a keypair from mnemonics and path
func DeriveKeypair(mnemonics string, path string) (*Secp256k1Keypair, error) {
	if path == "" {
		path = DefaultSecp256k1DerivationPath
	}

	seed := bip39.NewSeed(mnemonics, "")
	masterKey, err := bip32.NewMasterKey(seed)
	if err != nil {
		return nil, err
	}

	// Derive the key using the path
	derivedKey := masterKey
	// Implementation of path derivation would go here
	// You'll need to parse the path and derive each level

	return &Secp256k1Keypair{
		keypair: Secp256k1KeypairData{
			PublicKey: derivedKey.PublicKey().Key,
			SecretKey: derivedKey.Key,
		},
	}, nil
} 
package address

import (
	"encoding/hex"
	"strings"

	"github.com/btcsuite/btcutil/bech32"
)

// RoochAddress implements the Address interface
type RoochAddress struct {
	address []byte
}

// NewRoochAddress creates a new RoochAddress from either a byte slice or string
func NewRoochAddress(addr interface{}) (*RoochAddress, error) {
	switch v := addr.(type) {
	case string:
		if isHex(v) {
			// Handle hex string
			bytes, err := fromHex(v)
			if err != nil {
				return nil, err
			}
			return &RoochAddress{address: bytes}, nil
		}
		// Handle bech32 string
		_, data, err := bech32.Decode(v)
		if err != nil {
			return nil, err
		}
		converted, err := bech32.ConvertBits(data, 5, 8, false)
		if err != nil {
			return nil, err
		}
		return &RoochAddress{address: converted}, nil
	case []byte:
		return &RoochAddress{address: v}, nil
	default:
		return nil, ErrInvalidAddressType
	}
}

// ToStr returns the bech32 string representation
func (r *RoochAddress) ToStr() string {
	return r.ToBech32Address()
}

// ToBytes returns the raw byte representation
func (r *RoochAddress) ToBytes() []byte {
	return r.address
}

// ToHexAddress returns the normalized hex string representation
func (r *RoochAddress) ToHexAddress() string {
	return normalizeRoochAddress(toHex(r.address))
}

// ToBech32Address returns the bech32 encoded address
func (r *RoochAddress) ToBech32Address() string {
	converted, err := bech32.ConvertBits(r.address, 8, 5, true)
	if err != nil {
		return "" // In practice, you might want to handle this error differently
	}
	encoded, err := bech32.Encode(ROOCH_BECH32_PREFIX, converted)
	if err != nil {
		return "" // In practice, you might want to handle this error differently
	}
	return encoded
}

// Helper functions

func isHex(s string) bool {
	s = strings.TrimPrefix(strings.ToLower(s), "0x")
	_, err := hex.DecodeString(s)
	return err == nil
}

func fromHex(s string) ([]byte, error) {
	s = strings.TrimPrefix(strings.ToLower(s), "0x")
	return hex.DecodeString(s)
}

func toHex(b []byte) string {
	return "0x" + hex.EncodeToString(b)
} 
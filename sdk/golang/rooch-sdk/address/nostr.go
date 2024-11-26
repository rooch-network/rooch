// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

package address

import (
	"github.com/btcsuite/btcutil/bech32"
)

const PREFIX_BECH32_PUBLIC_KEY = "npub"

type NoStrAddress struct {
	str   string
	bytes []byte
}

// NewNoStrAddress creates a new NoStrAddress from either a string or bytes
func NewNoStrAddress(input interface{}) (*NoStrAddress, error) {
	switch v := input.(type) {
	case string:
		// Decode bech32 string
		_, data, err := bech32.Decode(v)
		if err != nil {
			return nil, err
		}
		
		// Convert from 5-bit to 8-bit
		converted, err := bech32.ConvertBits(data, 5, 8, false)
		if err != nil {
			return nil, err
		}

		return &NoStrAddress{
			str:   v,
			bytes: converted,
		}, nil

	case []byte:
		// Convert from 8-bit to 5-bit
		converted, err := bech32.ConvertBits(v, 8, 5, true)
		if err != nil {
			return nil, err
		}

		// Encode to bech32
		str, err := bech32.Encode(PREFIX_BECH32_PUBLIC_KEY, converted)
		if err != nil {
			return nil, err
		}

		return &NoStrAddress{
			str:   str,
			bytes: v,
		}, nil

	default:
		return nil, fmt.Errorf("invalid input type")
	}
}

// GenRoochAddress generates a RoochAddress from the NoStrAddress
func (n *NoStrAddress) GenRoochAddress() (*RoochAddress, error) {
	btcAddr, err := BitcoinAddressFromPublicKey(n.bytes)
	if err != nil {
		return nil, err
	}
	return btcAddr.GenRoochAddress()
}

// ToStr returns the string representation
func (n *NoStrAddress) ToStr() string {
	return n.str
}

// ToBytes returns the byte representation
func (n *NoStrAddress) ToBytes() []byte {
	return n.bytes
} 
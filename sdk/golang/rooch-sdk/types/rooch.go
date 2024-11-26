// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

package types

import (
	"math/big"
	"github.com/rooch-network/rooch/sdk/golang/rooch-sdk/address"
)

// Basic numeric types
type (
	U8  uint8
	U16 uint16
	U32 uint32
	U64 uint64
)

// Large numeric types using big.Int for u128 and u256
type (
	U128 *big.Int
	U256 *big.Int
)

// Boolean type
type Bool bool

// Object and address types
type (
	ObjectID string
	Address  interface{} // Can be string, *address.Address, or []byte
)

// Identifier type
type Identifier string 
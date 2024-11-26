// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

package types

// Bytes is a type alias for []byte
type Bytes []byte

// EmptyBytes represents an empty byte slice
var EmptyBytes = make([]byte, 0)

// NullBytes represents a one-element byte slice containing zero
var NullBytes = []byte{0} 
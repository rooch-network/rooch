// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

package utils

import (
	"encoding/base64"
)

// FromB64 converts a base64 string to a byte slice
func FromB64(base64String string) ([]byte, error) {
	return base64.StdEncoding.DecodeString(base64String)
}

// ToB64 converts a byte slice to a base64 string
func ToB64(bytes []byte) string {
	return base64.StdEncoding.EncodeToString(bytes)
} 
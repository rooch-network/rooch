// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

package crypto

import (
	"bytes"
	"encoding/base64"
	"errors"
)

// PublicKeyInitData represents data that can be converted into a public key
// In Go, we'll typically handle this through constructor functions rather than a union type
type PublicKeyInitData []byte

// PublicKey interface defines the common behavior for public keys
type PublicKey[T any] interface {
	// Equals checks if two public keys are equal
	Equals(publicKey PublicKey[T]) bool

	// ToBase64 returns the base-64 representation of the public key
	ToBase64() string

	// String returns string representation of the public key
	String() string

	// ToBytes returns the byte array representation of the public key
	ToBytes() []byte

	// Flag returns signature scheme flag of the public key
	Flag() uint8

	// ToAddress converts the public key to its corresponding address
	ToAddress() T

	// Verify checks if the signature is valid for the provided message
	Verify(data []byte, signature []byte) (bool, error)
}

// BasePublicKey provides a basic implementation of the PublicKey interface
type BasePublicKey[T any] struct {
	bytes []byte
}

// Equals implements the equality check for public keys
func (pk *BasePublicKey[T]) Equals(other PublicKey[T]) bool {
	return bytes.Equal(pk.ToBytes(), other.ToBytes())
}

// ToBase64 returns the base-64 representation
func (pk *BasePublicKey[T]) ToBase64() string {
	return base64.StdEncoding.EncodeToString(pk.bytes)
}

// String returns an error as it should be implemented by concrete types
func (pk *BasePublicKey[T]) String() string {
	return "use ToBase64() or ToBytes() instead"
}

// ToBytes returns the underlying bytes
func (pk *BasePublicKey[T]) ToBytes() []byte {
	return pk.bytes
}

// Flag should be implemented by concrete types
func (pk *BasePublicKey[T]) Flag() uint8 {
	panic("Flag() must be implemented by concrete public key types")
}

// ToAddress should be implemented by concrete types
func (pk *BasePublicKey[T]) ToAddress() T {
	panic("ToAddress() must be implemented by concrete public key types")
}

// Verify should be implemented by concrete types
func (pk *BasePublicKey[T]) Verify(data []byte, signature []byte) (bool, error) {
	return false, errors.New("Verify() must be implemented by concrete public key types")
} 
// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

package types

// TypeTag represents a Move type identifier
type TypeTag interface {
    isTypeTag() // marker method to ensure type safety
}

// Basic type tags
type (
    U8TypeTag      struct{}
    U16TypeTag     struct{}
    U32TypeTag     struct{}
    U64TypeTag     struct{}
    U128TypeTag    struct{}
    U256TypeTag    struct{}
    BoolTypeTag    struct{}
    AddressTypeTag struct{}
    SignerTypeTag  struct{}
)

// Implement TypeTag interface for basic types
func (U8TypeTag) isTypeTag()      {}
func (U16TypeTag) isTypeTag()     {}
func (U32TypeTag) isTypeTag()     {}
func (U64TypeTag) isTypeTag()     {}
func (U128TypeTag) isTypeTag()    {}
func (U256TypeTag) isTypeTag()    {}
func (BoolTypeTag) isTypeTag()    {}
func (AddressTypeTag) isTypeTag() {}
func (SignerTypeTag) isTypeTag()  {}

// VectorTypeTag represents a vector type
type VectorTypeTag struct {
    ElementType TypeTag
}

func (VectorTypeTag) isTypeTag() {}

// StructTag represents a Move struct type
type StructTag struct {
    Address    string
    Module     string
    Name       string
    TypeParams []TypeTag
}

func (StructTag) isTypeTag() {}

// BcsStructTag represents a BCS-specific struct type
type BcsStructTag struct {
    Address    Address
    Module     string
    Name       string
    TypeParams []BcsTypeTag
}

// BcsTypeTag represents BCS-specific type tags
type BcsTypeTag struct {
    Bool    *bool        `json:"bool,omitempty"`
    U8      *bool        `json:"u8,omitempty"`
    U16     *bool        `json:"u16,omitempty"`
    U32     *bool        `json:"u32,omitempty"`
    U64     *bool        `json:"u64,omitempty"`
    U128    *bool        `json:"u128,omitempty"`
    U256    *bool        `json:"u256,omitempty"`
    Address *bool        `json:"address,omitempty"`
    Signer  *bool        `json:"signer,omitempty"`
    Vector  *BcsTypeTag  `json:"vector,omitempty"`
    Struct  *BcsStructTag `json:"struct,omitempty"`
} 
package bcs

import (
	"encoding/hex"
	"errors"
	"fmt"
	"math/big"

	"github.com/rooch-network/rooch/sdk/golang/address"
	"github.com/rooch-network/rooch/sdk/golang/types"
)

// BCS type definitions
type MultiChainAddress struct {
	MultiChainId *big.Int
	RawAddress   []byte
}

type StructTag struct {
	Address    string
	Module     string
	Name       string
	TypeParams []TypeTag
}

type TypeTag interface {
	isTypeTag()
}

// TypeTag implementations
type (
	BoolType    struct{}
	U8Type      struct{}
	U16Type     struct{}
	U32Type     struct{}
	U64Type     struct{}
	U128Type    struct{}
	U256Type    struct{}
	AddressType struct{}
	SignerType  struct{}
	VectorType  struct {
		ElementType TypeTag
	}
	StructType struct {
		StructTag StructTag
	}
)

// Implement isTypeTag for all types
func (BoolType) isTypeTag()    {}
func (U8Type) isTypeTag()      {}
func (U16Type) isTypeTag()     {}
func (U32Type) isTypeTag()     {}
func (U64Type) isTypeTag()     {}
func (U128Type) isTypeTag()    {}
func (U256Type) isTypeTag()    {}
func (AddressType) isTypeTag() {}
func (SignerType) isTypeTag()  {}
func (VectorType) isTypeTag()  {}
func (StructType) isTypeTag()  {}

type ModuleId struct {
	Address string
	Name    string
}

type FunctionId struct {
	ModuleId ModuleId
	Name     string
}

type ScriptCall struct {
	Code     []byte
	Args     []byte
	TypeArgs []TypeTag
}

type CallFunction struct {
	FunctionId FunctionId
	TypeArgs   []TypeTag
	Args       [][]byte
}

type MoveAction struct {
	Type        string // "ScriptCall" or "CallFunction"
	ScriptCall  *ScriptCall
	CallFunction *CallFunction
}

type RoochTransactionData struct {
	Sender         string
	SequenceNumber uint64
	ChainId        uint64
	MaxGas         uint64
	Action         MoveAction
}

type Authenticator struct {
	AuthValidatorId uint64
	Payload         []byte
}

type RoochTransaction struct {
	Data []byte
	Auth []byte
}

type BitcoinAuthPayload struct {
	Signature     []byte
	MessagePrefix []byte
	MessageInfo   []byte
	PublicKey     []byte
	FromAddress   []byte
}

// Helper functions for type conversions and validations
func ValidateAddress(addr string) error {
	if !address.IsValidAddress(addr) {
		return fmt.Errorf("invalid address %s", addr)
	}
	return nil
}

func ConvertToBytes(input string, encoding string) ([]byte, error) {
	switch encoding {
	case "hex":
		return hex.DecodeString(input)
	case "utf8":
		return []byte(input), nil
	default:
		return nil, fmt.Errorf("unsupported encoding: %s", encoding)
	}
}

func Vector(input interface{}, encoding string) ([]byte, error) {
	switch v := input.(type) {
	case string:
		return ConvertToBytes(v, encoding)
	case []byte:
		return v, nil
	default:
		return nil, errors.New("invalid input type for Vector")
	}
}

func RawBytes(input interface{}, encoding string) ([]byte, error) {
	return Vector(input, encoding)
}

// Serializer interface for BCS encoding/decoding
type Serializer interface {
	Serialize() ([]byte, error)
	Deserialize([]byte) error
} 
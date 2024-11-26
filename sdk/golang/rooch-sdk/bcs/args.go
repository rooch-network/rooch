package bcs

import (
	"encoding/hex"

	"github.com/rooch-network/rooch/sdk/golang/rooch-sdk/types"
	"github.com/rooch-network/rooch/sdk/golang/rooch-sdk/utils"
)

type ArgType string

const (
	ArgTypeU8       ArgType = "u8"
	ArgTypeU16      ArgType = "u16"
	ArgTypeU32      ArgType = "u32"
	ArgTypeU64      ArgType = "u64"
	ArgTypeU128     ArgType = "u128"
	ArgTypeU256     ArgType = "u256"
	ArgTypeBool     ArgType = "bool"
	ArgTypeString   ArgType = "string"
	ArgTypeObject   ArgType = "object"
	ArgTypeObjectId ArgType = "objectId"
	ArgTypeAddress  ArgType = "address"
)

type Args struct {
	value []byte
}

func NewArgs(input []byte) *Args {
	return &Args{value: input}
}

func (a *Args) EncodeWithHex() string {
	return hex.EncodeToString(a.value)
}

func (a *Args) Encode() []byte {
	return a.value
}

func U8(input uint8) *Args {
	return NewArgs(utils.SerializeU8(input))
}

func U16(input uint16) *Args {
	return NewArgs(utils.SerializeU16(input))
}

func U32(input uint32) *Args {
	return NewArgs(utils.SerializeU32(input))
}

func U64(input uint64) *Args {
	return NewArgs(utils.SerializeU64(input))
}

func U128(input *types.U128) *Args {
	return NewArgs(utils.SerializeU128(input))
}

func U256(input *types.U256) *Args {
	return NewArgs(utils.SerializeU256(input))
}

func Bool(input bool) *Args {
	return NewArgs(utils.SerializeBool(input))
}

func String(input string) *Args {
	return NewArgs(utils.SerializeString(input))
}

func Address(input string) *Args {
	return NewArgs(utils.SerializeAddress(input))
}

func Object(input *StructTag) *Args {
	objectID := StructTagToObjectID(input)
	return ObjectId(objectID)
}

func ObjectId(input string) *Args {
	return NewArgs(utils.SerializeObjectID(input))
}

func Struct(input []byte) *Args {
	return NewArgs(input)
}

func Vec(argType ArgType, input interface{}) *Args {
	var serialized []byte

	switch argType {
	case ArgTypeU8:
		serialized = utils.SerializeVecU8(input.([]uint8))
	case ArgTypeU16:
		serialized = utils.SerializeVecU16(input.([]uint16))
	case ArgTypeU32:
		serialized = utils.SerializeVecU32(input.([]uint32))
	case ArgTypeU64:
		serialized = utils.SerializeVecU64(input.([]uint64))
	case ArgTypeU128:
		serialized = utils.SerializeVecU128(input.([]*types.U128))
	case ArgTypeU256:
		serialized = utils.SerializeVecU256(input.([]*types.U256))
	case ArgTypeBool:
		serialized = utils.SerializeVecBool(input.([]bool))
	case ArgTypeString:
		serialized = utils.SerializeVecString(input.([]string))
	case ArgTypeObject:
		structTags := input.([]*StructTag)
		objectIDs := make([]string, len(structTags))
		for i, tag := range structTags {
			objectIDs[i] = StructTagToObjectID(tag)
		}
		serialized = utils.SerializeVecObjectID(objectIDs)
	case ArgTypeObjectId:
		serialized = utils.SerializeVecObjectID(input.([]string))
	case ArgTypeAddress:
		serialized = utils.SerializeVecAddress(input.([]string))
	}

	return NewArgs(serialized)
} 
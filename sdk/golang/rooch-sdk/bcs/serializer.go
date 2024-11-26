package bcs

import (
	"fmt"
	"regexp"
	"strings"

	"github.com/rooch-network/rooch/sdk/golang/rooch-sdk/address"
	"github.com/rooch-network/rooch/sdk/golang/rooch-sdk/utils"
)

var (
	vectorRegex = regexp.MustCompile(`^vector<(.+)>$`)
	structRegex = regexp.MustCompile(`^([^:]+)::([^:]+)::([^<]+)(<(.+)>)?`)
)

type Serializer struct{}

func (s *Serializer) StructTagToObjectID(input StructTag) string {
	return "0x" + utils.ToHex(utils.Sha3256(s.StructTagToCanonicalString(input)))
}

func (s *Serializer) StructTagToCanonicalString(input StructTag) string {
	result := fmt.Sprintf("%s::%s::%s", address.CanonicalRoochAddress(input.Address), input.Module, input.Name)

	if len(input.TypeParams) > 0 {
		typeParams := make([]string, len(input.TypeParams))
		for i, param := range input.TypeParams {
			typeParams[i] = s.TypeTagToString(param)
		}
		result += fmt.Sprintf("<%s>", strings.Join(typeParams, ","))
	}

	return result
}

func (s *Serializer) TypeTagToString(input TypeTag) string {
	if str, ok := input.(string); ok {
		return str
	}

	if vector, ok := input.(map[string]TypeTag); ok {
		if val, exists := vector["Vector"]; exists {
			return fmt.Sprintf("vector<%s>", s.TypeTagToString(val))
		}
	}

	if structTag, ok := input.(map[string]StructTag); ok {
		if val, exists := structTag["Struct"]; exists {
			return s.StructTagToCanonicalString(val)
		}
	}

	panic("Invalid TypeTag")
}

func (s *Serializer) TypeTagParseFromStr(str string, normalizeAddress bool) BcsTypeTag {
	switch str {
	case "address":
		return BcsTypeTag{Address: &struct{}{}}
	case "bool":
		return BcsTypeTag{Bool: &struct{}{}}
	case "u8":
		return BcsTypeTag{U8: &struct{}{}}
	case "u16":
		return BcsTypeTag{U16: &struct{}{}}
	case "u32":
		return BcsTypeTag{U32: &struct{}{}}
	case "u64":
		return BcsTypeTag{U64: &struct{}{}}
	case "u128":
		return BcsTypeTag{U128: &struct{}{}}
	case "u256":
		return BcsTypeTag{U256: &struct{}{}}
	case "signer":
		return BcsTypeTag{Signer: &struct{}{}}
	}

	if matches := vectorRegex.FindStringSubmatch(str); matches != nil {
		return BcsTypeTag{
			Vector: s.TypeTagParseFromStr(matches[1], normalizeAddress),
		}
	}

	if matches := structRegex.FindStringSubmatch(str); matches != nil {
		addr := matches[1]
		if normalizeAddress {
			addr = address.NormalizeRoochAddress(addr)
		}
		
		var typeParams []BcsTypeTag
		if matches[5] != "" {
			typeParams = s.ParseStructTypeArgs(matches[5], normalizeAddress)
		}

		return BcsTypeTag{
			Struct: &StructTag{
				Address:    addr,
				Module:     matches[2],
				Name:       matches[3],
				TypeParams: typeParams,
			},
		}
	}

	panic(fmt.Sprintf("Encountered unexpected token when parsing type args for %s", str))
}

func (s *Serializer) ParseStructTypeArgs(str string, normalizeAddress bool) []BcsTypeTag {
	tokens := SplitGenericParameters(str)
	result := make([]BcsTypeTag, len(tokens))
	for i, token := range tokens {
		result[i] = s.TypeTagParseFromStr(token, normalizeAddress)
	}
	return result
}

func (s *Serializer) TagToString(tag BcsTypeTag) string {
	switch {
	case tag.Bool != nil:
		return "bool"
	case tag.U8 != nil:
		return "u8"
	case tag.U16 != nil:
		return "u16"
	case tag.U32 != nil:
		return "u32"
	case tag.U64 != nil:
		return "u64"
	case tag.U128 != nil:
		return "u128"
	case tag.U256 != nil:
		return "u256"
	case tag.Address != nil:
		return "address"
	case tag.Signer != nil:
		return "signer"
	case tag.Vector != nil:
		return fmt.Sprintf("vector<%s>", s.TagToString(*tag.Vector))
	case tag.Struct != nil:
		struct_ := tag.Struct
		typeParams := make([]string, len(struct_.TypeParams))
		for i, param := range struct_.TypeParams {
			typeParams[i] = s.TagToString(param)
		}
		typeParamsStr := ""
		if len(typeParams) > 0 {
			typeParamsStr = fmt.Sprintf("<%s>", strings.Join(typeParams, ", "))
		}
		return fmt.Sprintf("%s::%s::%s%s", struct_.Address, struct_.Module, struct_.Name, typeParamsStr)
	}
	panic("Invalid TypeTag")
} 
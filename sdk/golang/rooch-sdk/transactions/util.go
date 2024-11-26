// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

package transactions

import (
	"fmt"
	"strings"
)

// TypeArgs represents the input structure for type arguments
type TypeArgs struct {
	Target  string `json:"target,omitempty"`
	Address string `json:"address,omitempty"`
	Module  string `json:"module,omitempty"`
	Name    string `json:"name,omitempty"`
}

// NormalizeTypeArgs converts TypeArgs into a slice of strings containing [address, module, name]
func NormalizeTypeArgs(input TypeArgs) ([]string, error) {
	if input.Target != "" {
		data := strings.Split(input.Target, "::")
		if len(data) != 3 {
			return nil, fmt.Errorf("invalid type")
		}
		return data, nil
	}

	return []string{input.Address, input.Module, input.Name}, nil
}

// NormalizeTypeArgsToStr converts TypeArgs into a string in the format "address::module::name"
func NormalizeTypeArgsToStr(input TypeArgs) (string, error) {
	if input.Target != "" {
		if len(strings.Split(input.Target, "::")) != 3 {
			return "", fmt.Errorf("invalid type")
		}
		return input.Target, nil
	}

	return fmt.Sprintf("%s::%s::%s", input.Address, input.Module, input.Name), nil
} 
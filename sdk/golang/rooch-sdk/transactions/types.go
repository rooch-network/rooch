// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

package transactions

import (
	"github.com/rooch-network/rooch/sdk/golang/rooch-sdk/bcs"
)

// CallScript represents a script to be executed
type CallScript struct {
	Code     string      `json:"code"`
	Args     []bcs.Args  `json:"args"`
	TypeArgs []bcs.TypeTag `json:"typeArgs"`
}

// FunctionArgs represents either a full function path or a target string
type FunctionArgs struct {
	// These fields are used when specifying full path
	Address string `json:"address,omitempty"`
	Module  string `json:"module,omitempty"`
	Function string `json:"function,omitempty"`
	
	// This field is used when specifying target directly
	Target  string `json:"target,omitempty"`
}

// CallFunctionArgs combines function arguments with optional Args and TypeArgs
type CallFunctionArgs struct {
	FunctionArgs
	Args     []bcs.Args    `json:"args,omitempty"`
	TypeArgs []bcs.TypeTag `json:"typeArgs,omitempty"`
}

// TypeArgs represents either a full type path or a target string
type TypeArgs struct {
	// These fields are used when specifying full path
	Address string `json:"address,omitempty"`
	Module  string `json:"module,omitempty"`
	Name    string `json:"name,omitempty"`
	
	// This field is used when specifying target directly
	Target  string `json:"target,omitempty"`
} 
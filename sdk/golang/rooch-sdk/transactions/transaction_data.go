// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

package transactions

import (
	"fmt"
	"strings"

	"github.com/rooch-network/rooch/sdk/golang/address"
	"github.com/rooch-network/rooch/sdk/golang/bcs"
	"github.com/rooch-network/rooch/sdk/golang/types"
	"github.com/rooch-network/rooch/sdk/golang/utils"
)

const DEFAULT_GAS = uint64(50000000)

type CallFunctionArgs struct {
	Target   string
	Address  string
	Module   string
	Function string
	Args     []bcs.Args
	TypeArgs []string
}

type CallFunction struct {
	Address  string
	Module   types.Identifier
	Function types.Identifier
	Args     []bcs.Args
	TypeArgs []string
}

func NewCallFunction(input CallFunctionArgs) *CallFunction {
	var pkg, mod, fn string
	if input.Target != "" {
		parts := strings.Split(input.Target, "::")
		pkg, mod, fn = parts[0], parts[1], parts[2]
	} else {
		pkg, mod, fn = input.Address, input.Module, input.Function
	}

	if input.Args == nil {
		input.Args = make([]bcs.Args, 0)
	}
	if input.TypeArgs == nil {
		input.TypeArgs = make([]string, 0)
	}

	return &CallFunction{
		Address:  pkg,
		Module:   mod,
		Function: fn,
		Args:     input.Args,
		TypeArgs: input.TypeArgs,
	}
}

func (c *CallFunction) FunctionId() string {
	return fmt.Sprintf("%s::%s::%s", 
		address.NormalizeRoochAddress(c.Address), 
		c.Module, 
		c.Function)
}

func (c *CallFunction) EncodeArgs() []string {
	result := make([]string, len(c.Args))
	for i, arg := range c.Args {
		result[i] = arg.EncodeWithHex()
	}
	return result
}

func (c *CallFunction) EncodeArgsWithUtf8() string {
	return ""
}

func (c *CallFunction) EncodeArgsToByteArrays() [][]uint8 {
	result := make([][]uint8, len(c.Args))
	for i, arg := range c.Args {
		result[i] = arg.Encode()
	}
	return result
}

type MoveActionType interface {
	isActionType()
}

func (*CallFunction) isActionType() {}
func (*CallScript) isActionType()   {}

type MoveAction struct {
	Scheme int
	Val    MoveActionType
}

func NewCallFunctionAction(input CallFunctionArgs) *MoveAction {
	return &MoveAction{
		Scheme: 1,
		Val:    NewCallFunction(input),
	}
}

func NewCallScriptAction(input *CallScript) *MoveAction {
	return &MoveAction{
		Scheme: 2,
		Val:    input,
	}
}

type TransactionData struct {
	Sender         *types.Address
	SequenceNumber *uint64
	ChainId        *uint64
	MaxGas         uint64
	Action         *MoveAction
}

func NewTransactionData(
	action *MoveAction,
	sender string,
	sequenceNumber uint64,
	chainId uint64,
	maxGas uint64,
) *TransactionData {
	if maxGas == 0 {
		maxGas = DEFAULT_GAS
	}

	var senderAddr *types.Address
	if sender != "" {
		addr := types.Address(sender)
		senderAddr = &addr
	}

	return &TransactionData{
		Sender:         senderAddr,
		SequenceNumber: &sequenceNumber,
		ChainId:        &chainId,
		MaxGas:         maxGas,
		Action:         action,
	}
}

func (t *TransactionData) Encode() []byte {
	call := t.Action.Val.(*CallFunction)

	data := bcs.RoochTransactionData{
		Sender:         *t.Sender,
		SequenceNumber: *t.SequenceNumber,
		ChainId:        *t.ChainId,
		MaxGas:         t.MaxGas,
		Action: bcs.TransactionAction{
			Kind: "CallFunction",
			FunctionId: bcs.FunctionId{
				ModuleId: bcs.ModuleId{
					Address: call.Address,
					Name:    call.Module,
				},
				Name: call.Function,
			},
			Args:     call.EncodeArgsToByteArrays(),
			TypeArgs: call.TypeArgs,
		},
	}

	return data.Serialize()
}

func (t *TransactionData) Hash() []byte {
	return utils.Sha3_256(t.Encode())
} 
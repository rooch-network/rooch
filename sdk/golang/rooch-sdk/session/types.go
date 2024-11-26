// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

package session

import (
	"github.com/rooch-network/rooch/sdk/golang/keypairs"
)

// Scope represents the permission scope for a session
type Scope struct {
	Address  string `json:"address"`
	Module   string `json:"module"`
	Function string `json:"function"`
}

// CreateSessionArgs represents the arguments needed to create a new session
type CreateSessionArgs struct {
	AppName             string           `json:"appName"`
	AppURL              string           `json:"appUrl"`
	Scopes             []Scope          `json:"scopes"`
	Keypair            *keypairs.Ed25519Keypair `json:"keypair,omitempty"`
	MaxInactiveInterval *int             `json:"maxInactiveInterval,omitempty"`
} 
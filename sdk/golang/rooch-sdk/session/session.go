package session

import (
	"encoding/json"
	"fmt"
	"strings"
	"time"

	"github.com/rooch-network/rooch/sdk/golang/rooch-sdk/address"
	"github.com/rooch-network/rooch/sdk/golang/rooch-sdk/client"
	"github.com/rooch-network/rooch/sdk/golang/rooch-sdk/crypto"
	"github.com/rooch-network/rooch/sdk/golang/rooch-sdk/crypto/ed25519"
	"github.com/rooch-network/rooch/sdk/golang/rooch-sdk/transactions"
	"github.com/rooch-network/rooch/sdk/golang/rooch-sdk/types"
)

const (
	DefaultMaxInactiveInterval = 1200 // seconds
	RequiredScope             = "0x3::session_key::remove_session_key_entry"
)

type CreateSessionArgs struct {
	AppName            string
	AppUrl             string
	Scopes             []string
	Keypair            *ed25519.Ed25519Keypair
	MaxInactiveInterval int64
}

type Session struct {
	appName             string
	appUrl              string
	scopes              []string
	keypair             *ed25519.Ed25519Keypair
	maxInactiveInterval int64
	bitcoinAddress      *address.BitcoinAddress
	roochAddress        *address.RoochAddress
	localCreateSessionTime int64
	lastActiveTime      int64
}

func CREATE(client *client.RoochClient, signer crypto.Signer, args CreateSessionArgs) (*Session, error) {
	parsedScopes := make([]string, 0)
	for _, scope := range args.Scopes {
		if !strings.Contains(scope, "::") {
			return nil, fmt.Errorf("invalid scope format")
		}
		parsedScopes = append(parsedScopes, scope)
	}

	allOx3 := "0x3::*::*"
	hasRequiredScope := false
	for _, scope := range parsedScopes {
		if scope == allOx3 || scope == RequiredScope {
			hasRequiredScope = true
			break
		}
	}
	if !hasRequiredScope {
		parsedScopes = append(parsedScopes, RequiredScope)
	}

	maxInactiveInterval := args.MaxInactiveInterval
	if maxInactiveInterval == 0 {
		maxInactiveInterval = DefaultMaxInactiveInterval
	}

	keypair := args.Keypair
	if keypair == nil {
		var err error
		keypair, err = ed25519.GenerateKeypair()
		if err != nil {
			return nil, err
		}
	}

	session := &Session{
		appName:             args.AppName,
		appUrl:              args.AppUrl,
		scopes:              parsedScopes,
		keypair:             keypair,
		maxInactiveInterval: maxInactiveInterval,
		bitcoinAddress:      signer.GetBitcoinAddress(),
		roochAddress:        signer.GetRoochAddress(),
		localCreateSessionTime: time.Now().UnixMilli(),
		lastActiveTime:      time.Now().UnixMilli(),
	}

	return session.build(client, signer)
}

func FromJson(jsonData []byte) (*Session, error) {
	var jsonObj map[string]interface{}
	if err := json.Unmarshal(jsonData, &jsonObj); err != nil {
		return nil, err
	}

	keypair, err := ed25519.FromSecretKey([]byte(jsonObj["secretKey"].(string)))
	if err != nil {
		return nil, err
	}

	bitcoinAddr, err := address.NewBitcoinAddress(jsonObj["bitcoinAddress"].(string))
	if err != nil {
		return nil, err
	}

	roochAddr, err := address.NewRoochAddress(jsonObj["roochAddress"].(string))
	if err != nil {
		return nil, err
	}

	scopes := make([]string, 0)
	for _, scope := range jsonObj["scopes"].([]interface{}) {
		scopes = append(scopes, scope.(string))
	}

	return &Session{
		appName:             jsonObj["appName"].(string),
		appUrl:              jsonObj["appUrl"].(string),
		scopes:              scopes,
		keypair:             keypair,
		maxInactiveInterval: int64(jsonObj["maxInactiveInterval"].(float64)),
		bitcoinAddress:      bitcoinAddr,
		roochAddress:        roochAddr,
		localCreateSessionTime: int64(jsonObj["localCreateSessionTime"].(float64)),
		lastActiveTime:      int64(jsonObj["lastActiveTime"].(float64)),
	}, nil
}

func (s *Session) Sign(input []byte) ([]byte, error) {
	s.lastActiveTime = time.Now().UnixMilli()
	return s.keypair.Sign(input)
}

func (s *Session) SignTransaction(tx *transactions.Transaction) (*crypto.Authenticator, error) {
	hashData, err := tx.HashData()
	if err != nil {
		return nil, err
	}
	return crypto.NewRoochAuthenticator(hashData, s)
}

func (s *Session) GetRoochAddress() *address.RoochAddress {
	return s.roochAddress
}

func (s *Session) GetBitcoinAddress() *address.BitcoinAddress {
	return s.bitcoinAddress
}

func (s *Session) GetKeyScheme() crypto.SignatureScheme {
	return s.keypair.GetKeyScheme()
}

func (s *Session) GetPublicKey() crypto.PublicKey {
	return s.keypair.GetPublicKey()
}

func (s *Session) GetCreateTime() int64 {
	return s.localCreateSessionTime
}

func (s *Session) GetAuthKey() string {
	return s.keypair.GetRoochAddress().ToHexAddress()
}

func (s *Session) build(client *client.RoochClient, signer crypto.Signer) (*Session, error) {
	addrs := make([]string, 0)
	mods := make([]string, 0)
	fns := make([]string, 0)

	for _, scope := range s.scopes {
		parts := strings.Split(scope, "::")
		if len(parts) != 3 {
			return nil, fmt.Errorf("invalid scope format")
		}
		addrs = append(addrs, parts[0])
		mods = append(mods, parts[1])
		fns = append(fns, parts[2])
	}

	authKey := s.GetAuthKey()
	authKeyBytes, err := types.FromHex(authKey)
	if err != nil {
		return nil, err
	}

	tx := transactions.NewTransaction()
	info := fmt.Sprintf("Welcome to %s\nYou will authorize session:\nScope:\n%s\nTimeOut:%d",
		s.appName, strings.Join(s.scopes, "\n"), s.maxInactiveInterval)

	tx.CallFunction(transactions.FunctionCall{
		Target: "0x3::session_key::create_session_key_with_multi_scope_entry",
		Args: []interface{}{
			s.appName,
			s.appUrl,
			authKeyBytes,
			addrs,
			mods,
			fns,
			s.maxInactiveInterval,
		},
		Info: info,
	})

	result, err := client.SignAndExecuteTransaction(tx, signer)
	if err != nil {
		return nil, err
	}

	if result.ExecutionInfo.Status.Type != "executed" {
		return nil, fmt.Errorf("create session failed %v", result.ExecutionInfo.Status)
	}

	return s, nil
}

func (s *Session) ToJSON() ([]byte, error) {
	jsonObj := map[string]interface{}{
		"appName":             s.appName,
		"appUrl":              s.appUrl,
		"scopes":              s.scopes,
		"secretKey":           s.keypair.GetSecretKey(),
		"maxInactiveInterval": s.maxInactiveInterval,
		"bitcoinAddress":      s.bitcoinAddress.ToStr(),
		"roochAddress":        s.roochAddress.ToStr(),
		"localCreateSessionTime": s.localCreateSessionTime,
		"lastActiveTime":      s.lastActiveTime,
	}
	return json.Marshal(jsonObj)
} 
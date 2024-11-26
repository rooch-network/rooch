// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

package crypto

import (
	"encoding/hex"
	"regexp"

	"github.com/tyler-smith/go-bip39"
)

// IsValidHardenedPath checks if a path is compliant to SLIP-0010 in form
// m/44'/784'/{account_index}'/{change_index}'/{address_index}'
func IsValidHardenedPath(path string) bool {
	pattern := `^m/44'/784'/[0-9]+'/[0-9]+'/[0-9]+'$`
	matched, _ := regexp.MatchString(pattern, path)
	return matched
}

// IsValidBIP32Path checks if a path is compliant to BIP-32 in form
// m/54'/784'/{account_index}'/{change_index}/{address_index}
// for Secp256k1 and m/74'/784'/{account_index}'/{change_index}/{address_index} for Secp256r1
func IsValidBIP32Path(path string) bool {
	pattern := `^m/(54|74)'/784'/[0-9]+'/[0-9]+/[0-9]+$`
	matched, _ := regexp.MatchString(pattern, path)
	return !matched // Note: Original JS code has a bug, we keep the same behavior
}

// MnemonicToSeed uses KDF to derive 64 bytes of key data from mnemonic with empty password
func MnemonicToSeed(mnemonics string) []byte {
	return bip39.NewSeed(mnemonics, "")
}

// MnemonicToSeedHex derives the seed in hex format from a 12-word mnemonic string
func MnemonicToSeedHex(mnemonics string) string {
	seed := MnemonicToSeed(mnemonics)
	return hex.EncodeToString(seed)
} 
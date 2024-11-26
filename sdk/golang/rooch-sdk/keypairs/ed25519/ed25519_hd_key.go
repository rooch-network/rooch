package ed25519

import (
	"crypto/hmac"
	"crypto/sha512"
	"encoding/binary"
	"encoding/hex"
	"errors"
	"regexp"
	"strconv"
	"strings"

	"golang.org/x/crypto/ed25519"
)

const (
	ED25519_CURVE    = "ed25519 seed"
	HARDENED_OFFSET  = 0x80000000
)

var pathRegex = regexp.MustCompile(`^m(\/[0-9]+')+$`)

type Keys struct {
	Key       []byte
	ChainCode []byte
}

func replaceDerive(val string) string {
	return strings.ReplaceAll(val, "'", "")
}

func getMasterKeyFromSeed(seed string) (Keys, error) {
	seedBytes, err := hex.DecodeString(seed)
	if err != nil {
		return Keys{}, err
	}

	h := hmac.New(sha512.New, []byte(ED25519_CURVE))
	h.Write(seedBytes)
	I := h.Sum(nil)

	return Keys{
		Key:       I[:32],
		ChainCode: I[32:],
	}, nil
}

func CKDPriv(keys Keys, index uint32) Keys {
	indexBytes := make([]byte, 4)
	binary.BigEndian.PutUint32(indexBytes, index)

	data := make([]byte, 1+len(keys.Key)+len(indexBytes))
	data[0] = 0
	copy(data[1:], keys.Key)
	copy(data[1+len(keys.Key):], indexBytes)

	h := hmac.New(sha512.New, keys.ChainCode)
	h.Write(data)
	I := h.Sum(nil)

	return Keys{
		Key:       I[:32],
		ChainCode: I[32:],
	}
}

func getPublicKey(privateKey []byte, withZeroByte bool) []byte {
	publicKey := make([]byte, ed25519.PublicKeySize)
	copy(publicKey, ed25519.NewKeyFromSeed(privateKey)[32:])

	if withZeroByte {
		result := make([]byte, 1+len(publicKey))
		result[0] = 0
		copy(result[1:], publicKey)
		return result
	}
	return publicKey
}

func isValidPath(path string) bool {
	if !pathRegex.MatchString(path) {
		return false
	}

	segments := strings.Split(path, "/")[1:]
	for _, segment := range segments {
		segment = replaceDerive(segment)
		_, err := strconv.ParseInt(segment, 10, 64)
		if err != nil {
			return false
		}
	}
	return true
}

func derivePath(path string, seed string, offset uint32) (Keys, error) {
	if !isValidPath(path) {
		return Keys{}, errors.New("invalid derivation path")
	}

	masterKey, err := getMasterKeyFromSeed(seed)
	if err != nil {
		return Keys{}, err
	}

	segments := strings.Split(path, "/")[1:]
	result := masterKey

	for _, segment := range segments {
		segment = replaceDerive(segment)
		index, _ := strconv.ParseUint(segment, 10, 32)
		result = CKDPriv(result, uint32(index)+offset)
	}

	return result, nil
} 
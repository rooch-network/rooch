package crypto

import (
	"bytes"
	"testing"

	"github.com/btcsuite/btcutil/bech32"
)

const ROOCH_SECRET_KEY_PREFIX = "roochsecretkey"

func TestDecodeRoochSecretKey(t *testing.T) {
	t.Run("should correctly decode a valid Bech32 encoded string", func(t *testing.T) {
		secretKey := make([]byte, 32)
		signatureFlag := byte(0x00)
		data := append([]byte{signatureFlag}, secretKey...)
		
		words, err := bech32.ConvertBits(data, 8, 5, true)
		if err != nil {
			t.Fatal(err)
		}
		
		value, err := bech32.Encode(ROOCH_SECRET_KEY_PREFIX, words)
		if err != nil {
			t.Fatal(err)
		}

		decoded, err := DecodeRoochSecretKey(value)
		if err != nil {
			t.Fatal(err)
		}

		if decoded.Schema != "ED25519" {
			t.Errorf("expected ED25519 schema, got %s", decoded.Schema)
		}
		if !bytes.Equal(decoded.SecretKey, secretKey) {
			t.Error("secret keys don't match")
		}
	})

	t.Run("should throw an error for invalid prefix", func(t *testing.T) {
		secretKey := make([]byte, 32)
		signatureFlag := byte(0x00)
		data := append([]byte{signatureFlag}, secretKey...)
		
		words, _ := bech32.ConvertBits(data, 8, 5, true)
		value, _ := bech32.Encode("invalidprefix", words)

		_, err := DecodeRoochSecretKey(value)
		if err == nil || err.Error() != "invalid private key prefix" {
			t.Error("expected invalid private key prefix error")
		}
	})

	// Add more test cases similar to the TypeScript version...
}

func TestEncodeRoochSecretKey(t *testing.T) {
	t.Run("should encode correctly when given a valid 32-byte private key with ED25519 scheme", func(t *testing.T) {
		privateKey := bytes.Repeat([]byte{1}, 32)
		scheme := "ED25519"
		
		encoded, err := EncodeRoochSecretKey(privateKey, scheme)
		if err != nil {
			t.Fatal(err)
		}

		data := append([]byte{0x00}, privateKey...)
		words, _ := bech32.ConvertBits(data, 8, 5, true)
		expected, _ := bech32.Encode(ROOCH_SECRET_KEY_PREFIX, words)

		if encoded != expected {
			t.Errorf("encoded value doesn't match expected value")
		}
	})

	t.Run("should throw an error when the private key length is not 32 bytes", func(t *testing.T) {
		invalidPrivateKey := make([]byte, 31)
		scheme := "ED25519"
		
		_, err := EncodeRoochSecretKey(invalidPrivateKey, scheme)
		if err == nil || err.Error() != "Invalid bytes length" {
			t.Error("expected Invalid bytes length error")
		}
	})

	// Add more test cases similar to the TypeScript version...
} 
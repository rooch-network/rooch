package ed25519

import (
	"encoding/base64"
	"testing"

	"github.com/stretchr/testify/assert"
)

var testCases = []struct {
	sk string
	pk string
}{
	{
		sk: "roochsecretkey1qqzztph49dkdl3vyw5t6fecvtuesrt9f5f2lw8ndpvqael6rr42mwulf8v7",
		pk: "3z8zMjDk70frh7I0ZF269ROuM5PeMKsYxwgFgTZEH9s=",
	},
}

func TestEd25519Keypair(t *testing.T) {
	t.Run("create ed25519 keypair", func(t *testing.T) {
		kp := GenerateEd25519Keypair()
		assert.Equal(t, 32, len(kp.GetPublicKey().ToBytes()))
	})

	t.Run("export ed25519 keypair", func(t *testing.T) {
		kp := GenerateEd25519Keypair()
		secret := kp.GetSecretKey()
		assert.True(t, strings.HasPrefix(secret, ROOCH_SECRET_KEY_PREFIX))
	})

	t.Run("Create ed25519 keypair from secret key", func(t *testing.T) {
		// valid secret key is provided by rooch keystore
		testCase := testCases[0]
		
		key, err := DecodeRoochSecretKey(testCase.sk)
		assert.NoError(t, err)
		
		keypair := Ed25519KeypairFromSecretKey(key.SecretKey)
		assert.Equal(t, testCase.pk, keypair.GetPublicKey().ToBase64())

		keypair1 := Ed25519KeypairFromSecretKey(testCase.sk)
		assert.Equal(t, testCase.pk, keypair1.GetPublicKey().ToBase64())
	})

	t.Run("Invalid mnemonics to derive ed25519 keypair", func(t *testing.T) {
		assert.Panics(t, func() {
			DeriveEd25519Keypair("rooch")
		}, "Invalid mnemonic")
	})

	t.Run("Recover ed25519 keypair by mnemonics", func(t *testing.T) {
		// TODO: Implement when mnemonics functionality is ready
		t.Skip("Not implemented yet")
	})

	t.Run("Sign data", func(t *testing.T) {
		keypair := NewEd25519Keypair()
		message := []byte("hello rooch")
		signature, err := keypair.Sign(message)
		assert.NoError(t, err)
		
		isValid := keypair.GetPublicKey().Verify(message, signature)
		assert.True(t, isValid)
	})

	t.Run("Sign data same as rooch cli", func(t *testing.T) {
		// TODO: Implement CLI signature verification
		t.Skip("Not implemented yet")
	})
} 
package utils

import (
	"bytes"
	"testing"
)

func TestToB64(t *testing.T) {
	t.Run("should convert small byte slice to base64 string correctly", func(t *testing.T) {
		input := []byte("Hello")
		expectedOutput := "SGVsbG8="
		
		if result := ToB64(input); result != expectedOutput {
			t.Errorf("Expected %s but got %s", expectedOutput, result)
		}
	})

	t.Run("should return an empty string when input is empty", func(t *testing.T) {
		input := []byte{}
		expectedOutput := ""
		
		if result := ToB64(input); result != expectedOutput {
			t.Errorf("Expected empty string but got %s", result)
		}
	})

	t.Run("should process large byte slice correctly", func(t *testing.T) {
		input := bytes.Repeat([]byte("A"), 8191)
		result := ToB64(input)
		
		// Decode the result back to verify correctness
		decoded, err := base64.StdEncoding.DecodeString(result)
		if err != nil {
			t.Errorf("Failed to decode result: %v", err)
		}
		
		if !bytes.Equal(decoded, input) {
			t.Error("Decoded result doesn't match input")
		}
	})

	t.Run("should handle non-ASCII characters correctly", func(t *testing.T) {
		input := []byte{195, 164, 195, 182, 195, 188} // "äöü" in UTF-8
		expectedOutput := "w6TDtsO8"
		
		if result := ToB64(input); result != expectedOutput {
			t.Errorf("Expected %s but got %s", expectedOutput, result)
		}
	})
} 
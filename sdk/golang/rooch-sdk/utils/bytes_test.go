package utils

import (
	"testing"
)

func TestBytesEqual(t *testing.T) {
	tests := []struct {
		name     string
		a        []byte
		b        []byte
		expected bool
	}{
		{
			name:     "identical bytes",
			a:        []byte{1, 2, 3},
			b:        []byte{1, 2, 3},
			expected: true,
		},
		{
			name:     "different lengths",
			a:        []byte{1, 2, 3},
			b:        []byte{1, 2, 3, 4},
			expected: false,
		},
		{
			name:     "single element",
			a:        []byte{1},
			b:        []byte{1},
			expected: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if got := BytesEqual(tt.a, tt.b); got != tt.expected {
				t.Errorf("BytesEqual() = %v, want %v", got, tt.expected)
			}
		})
	}
}

func TestBytesToString(t *testing.T) {
	input := []byte("Hello")
	tests := []struct {
		name     string
		encoding CoderType
		expected string
	}{
		{"utf8", UTF8, "Hello"},
		{"hex", HEX, "48656c6c6f"},
		{"base16", BASE16, "48656C6C6F"},
		{"base32", BASE32, "JBSWY3DP"},
		{"base64", BASE64, "SGVsbG8="},
		{"base64url", BASE64URL, "SGVsbG8="},
		{"base58", BASE58, "9Ajdvzr"},
		{"base58xmr", BASE58XMR, "9Ajdvzr"},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got, err := BytesToString(tt.encoding, input)
			if err != nil {
				t.Errorf("BytesToString() error = %v", err)
				return
			}
			if got != tt.expected {
				t.Errorf("BytesToString() = %v, want %v", got, tt.expected)
			}
		})
	}
}

func TestStringToBytes(t *testing.T) {
	expected := []byte("Hello")
	tests := []struct {
		name     string
		encoding CoderType
		input    string
	}{
		{"utf8", UTF8, "Hello"},
		{"hex", HEX, "48656c6c6f"},
		{"base16", BASE16, "48656C6C6F"},
		{"base32", BASE32, "JBSWY3DP"},
		{"base64", BASE64, "SGVsbG8="},
		{"base64url", BASE64URL, "SGVsbG8="},
		{"base58", BASE58, "9Ajdvzr"},
		{"base58xmr", BASE58XMR, "9Ajdvzr"},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got, err := StringToBytes(tt.encoding, tt.input)
			if err != nil {
				t.Errorf("StringToBytes() error = %v", err)
				return
			}
			if !BytesEqual(got, expected) {
				t.Errorf("StringToBytes() = %v, want %v", got, expected)
			}
		})
	}
}

func TestConcatBytes(t *testing.T) {
	tests := []struct {
		name     string
		chunks   [][]byte
		expected []byte
	}{
		{
			name: "multiple chunks",
			chunks: [][]byte{
				{1, 2, 3},
				{4, 5, 6},
			},
			expected: []byte{1, 2, 3, 4, 5, 6},
		},
		{
			name: "empty chunk",
			chunks: [][]byte{
				{},
				{4, 5, 6},
			},
			expected: []byte{4, 5, 6},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got := ConcatBytes(tt.chunks...)
			if !BytesEqual(got, tt.expected) {
				t.Errorf("ConcatBytes() = %v, want %v", got, tt.expected)
			}
		})
	}
} 
package crypto

import (
	"testing"
)

func TestIsValidHardenedPath(t *testing.T) {
	tests := []struct {
		name     string
		path     string
		expected bool
	}{
		{
			name:     "valid path with typical indices",
			path:     "m/44'/784'/0'/0'/0'",
			expected: true,
		},
		{
			name:     "valid path with higher indices",
			path:     "m/44'/784'/123'/456'/789'",
			expected: true,
		},
		{
			name:     "path with missing m prefix",
			path:     "44'/784'/0'/0'/0'",
			expected: false,
		},
		{
			name:     "path with missing apostrophes",
			path:     "m/44/784/0/0/0",
			expected: false,
		},
		{
			name:     "path with non-numeric indices",
			path:     "m/44'/784'/a'/b'/c'",
			expected: false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			result := IsValidHardenedPath(tt.path)
			if result != tt.expected {
				t.Errorf("IsValidHardenedPath(%s) = %v, want %v", tt.path, result, tt.expected)
			}
		})
	}
}

func TestIsValidBIP32Path(t *testing.T) {
	tests := []struct {
		name     string
		path     string
		expected bool
	}{
		{
			name:     "valid BIP32 path case 1",
			path:     "m/54'/784'/0'/0/0",
			expected: false,
		},
		{
			name:     "invalid BIP32 path with n prefix",
			path:     "n/54'/784'/0'/0/0",
			expected: true,
		},
		{
			name:     "invalid BIP32 path with different first index",
			path:     "m/53'/784'/0'/0/0",
			expected: true,
		},
		{
			name:     "invalid BIP32 path with different second index",
			path:     "m/54'/785'/0'/0/0",
			expected: true,
		},
		{
			name:     "valid BIP32 path case 2",
			path:     "m/74'/784'/1'/1/1",
			expected: false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			result := IsValidBIP32Path(tt.path)
			if result != tt.expected {
				t.Errorf("IsValidBIP32Path(%s) = %v, want %v", tt.path, result, tt.expected)
			}
		})
	}
} 
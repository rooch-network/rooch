package utils

import (
	"testing"
)

func TestIsHex(t *testing.T) {
	tests := []struct {
		name     string
		input    interface{}
		expected bool
	}{
		{"valid hex with 0x prefix", "0x1a2b3c", true},
		{"valid hex without prefix", "1a2b3c", true},
		{"odd length string", "1a2b3", false},
		{"invalid characters", "1a2b3g", false},
		{"invalid byte array", []uint8{0, 255, 255}, true},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if got := IsHex(tt.input); got != tt.expected {
				t.Errorf("IsHex() = %v, want %v", got, tt.expected)
			}
		})
	}
}

func TestGetHexByteLength(t *testing.T) {
	tests := []struct {
		name     string
		input    string
		expected float64
	}{
		{"with 0x prefix", "0x1234", 2},
		{"odd characters", "123", 1.5},
		{"with 0X prefix", "0X12G4", 2},
		{"empty string", "", 0},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if got := GetHexByteLength(tt.input); got != tt.expected {
				t.Errorf("GetHexByteLength() = %v, want %v", got, tt.expected)
			}
		})
	}
}

func TestNormalizeHex(t *testing.T) {
	tests := []struct {
		name     string
		input    string
		expected string
	}{
		{"with 0x prefix", "0x1a2b3c", "1a2b3c"},
		{"without 0x prefix", "1a2b3c", "1a2b3c"},
		{"single character", "a", "a"},
		{"only 0x", "0x", ""},
		{"special characters", "@#$%^&*", "@#$%^&*"},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if got := NormalizeHex(tt.input); got != tt.expected {
				t.Errorf("NormalizeHex() = %v, want %v", got, tt.expected)
			}
		})
	}
}

func TestFromHEX(t *testing.T) {
	tests := []struct {
		name     string
		input    string
		expected []uint8
	}{
		{"even length", "4a6f686e", []uint8{74, 111, 104, 110}},
		{"odd length", "a3f", []uint8{10, 63}},
		{"invalid chars", "zxy123", []uint8{0, 0, 35}},
		{"single char", "f", []uint8{15}},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got := FromHEX(tt.input)
			if len(got) != len(tt.expected) {
				t.Errorf("FromHEX() length = %v, want %v", len(got), len(tt.expected))
			}
			for i := range got {
				if got[i] != tt.expected[i] {
					t.Errorf("FromHEX() at index %d = %v, want %v", i, got[i], tt.expected[i])
				}
			}
		})
	}
}

func TestToHEX(t *testing.T) {
	tests := []struct {
		name     string
		input    []uint8
		expected string
	}{
		{"normal bytes", []uint8{0, 1, 2, 255}, "000102ff"},
		{"empty array", []uint8{}, ""},
		{"max values", []uint8{255, 255, 255}, "ffffff"},
		{"min values", []uint8{0, 0, 0}, "000000"},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got, err := ToHEX(tt.input)
			if err != nil {
				t.Errorf("ToHEX() error = %v", err)
				return
			}
			if got != tt.expected {
				t.Errorf("ToHEX() = %v, want %v", got, tt.expected)
			}
		})
	}
} 
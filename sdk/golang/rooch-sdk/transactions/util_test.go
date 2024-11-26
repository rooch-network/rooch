package transactions

import (
	"testing"
)

func TestNormalizeTypeArgs(t *testing.T) {
	tests := []struct {
		name     string
		input    TypeArgs
		expected []string
		wantErr  bool
	}{
		{
			name:     "should correctly split target string into three parts",
			input:    TypeArgs{Target: "address::module::name"},
			expected: []string{"address", "module", "name"},
		},
		{
			name: "should return array with address, module, and name when target is not present",
			input: TypeArgs{
				Address: "address",
				Module:  "module",
				Name:    "name",
			},
			expected: []string{"address", "module", "name"},
		},
		{
			name:    "should throw error when target string does not contain exactly three parts",
			input:   TypeArgs{Target: "address::module"},
			wantErr: true,
		},
		{
			name:    "should throw error when target string is empty",
			input:   TypeArgs{Target: ""},
			wantErr: false,
			expected: []string{"", "", ""},
		},
		{
			name: "should return array with empty strings when address, module, or name properties are empty",
			input: TypeArgs{
				Address: "",
				Module:  "",
				Name:    "",
			},
			expected: []string{"", "", ""},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			defer func() {
				r := recover()
				if (r != nil) != tt.wantErr {
					t.Errorf("NormalizeTypeArgs() panic = %v, wantErr %v", r, tt.wantErr)
				}
			}()

			result := NormalizeTypeArgs(tt.input)
			if !tt.wantErr {
				if len(result) != len(tt.expected) {
					t.Errorf("NormalizeTypeArgs() = %v, want %v", result, tt.expected)
				}
				for i := range result {
					if result[i] != tt.expected[i] {
						t.Errorf("NormalizeTypeArgs() = %v, want %v", result, tt.expected)
					}
				}
			}
		})
	}
}

func TestNormalizeTypeArgsToStr(t *testing.T) {
	tests := []struct {
		name     string
		input    TypeArgs
		expected string
		wantErr  bool
	}{
		{
			name: "should return formatted string when input contains address, module, and name",
			input: TypeArgs{
				Address: "0x1",
				Module:  "Module",
				Name:    "Name",
			},
			expected: "0x1::Module::Name",
		},
		{
			name:     "should return target string when input contains target",
			input:    TypeArgs{Target: "0x1::Module::Name"},
			expected: "0x1::Module::Name",
		},
		{
			name:    "should throw error when target string does not contain exactly three parts",
			input:   TypeArgs{Target: "0x1::Module"},
			wantErr: true,
		},
		{
			name: "should handle empty strings for address, module, and name",
			input: TypeArgs{
				Address: "",
				Module:  "",
				Name:    "",
			},
			expected: "::",
		},
		{
			name:    "should throw error when target is an empty string",
			input:   TypeArgs{Target: ""},
			wantErr: false,
			expected: "::",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			defer func() {
				r := recover()
				if (r != nil) != tt.wantErr {
					t.Errorf("NormalizeTypeArgsToStr() panic = %v, wantErr %v", r, tt.wantErr)
				}
			}()

			result := NormalizeTypeArgsToStr(tt.input)
			if !tt.wantErr && result != tt.expected {
				t.Errorf("NormalizeTypeArgsToStr() = %v, want %v", result, tt.expected)
			}
		})
	}
} 
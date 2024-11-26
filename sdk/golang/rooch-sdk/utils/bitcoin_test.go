package utils

import (
	"testing"
)

func TestValidateWitness(t *testing.T) {
	t.Run("should accept data with length between 2 and 40 inclusive", func(t *testing.T) {
		validData := make([]byte, 20)
		if err := ValidateWitness(1, validData); err != nil {
			t.Errorf("Expected no error, got %v", err)
		}
	})

	t.Run("should accept version numbers between 0 and 16 inclusive", func(t *testing.T) {
		validData := make([]byte, 20)
		for version := 0; version <= 16; version++ {
			if err := ValidateWitness(version, validData); err != nil {
				t.Errorf("Expected no error for version %d, got %v", version, err)
			}
		}
	})

	t.Run("should throw error for data length less than 2", func(t *testing.T) {
		invalidData := make([]byte, 1)
		if err := ValidateWitness(1, invalidData); err == nil {
			t.Error("Expected error for invalid length, got nil")
		} else if err.Error() != "Witness: invalid length" {
			t.Errorf("Expected 'Witness: invalid length' error, got %v", err)
		}
	})

	t.Run("should throw error for data length greater than 40", func(t *testing.T) {
		invalidData := make([]byte, 41)
		if err := ValidateWitness(1, invalidData); err == nil {
			t.Error("Expected error for invalid length, got nil")
		} else if err.Error() != "Witness: invalid length" {
			t.Errorf("Expected 'Witness: invalid length' error, got %v", err)
		}
	})

	t.Run("should throw error for version numbers greater than 16", func(t *testing.T) {
		validData := make([]byte, 20)
		if err := ValidateWitness(17, validData); err == nil {
			t.Error("Expected error for invalid version, got nil")
		} else if err.Error() != "Witness: invalid version" {
			t.Errorf("Expected 'Witness: invalid version' error, got %v", err)
		}
	})

	t.Run("should throw error for version 0 with data length not equal to 20 or 32", func(t *testing.T) {
		invalidData := make([]byte, 25)
		if err := ValidateWitness(0, invalidData); err == nil {
			t.Error("Expected error for invalid length for version, got nil")
		} else if err.Error() != "Witness: invalid length for version" {
			t.Errorf("Expected 'Witness: invalid length for version' error, got %v", err)
		}
	})
} 
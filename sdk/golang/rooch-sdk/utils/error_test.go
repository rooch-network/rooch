package utils

import (
	"testing"
)

func TestParseRoochErrorCode(t *testing.T) {
	t.Run("should return the correct sub status", func(t *testing.T) {
		errorMessage := "status ABORTED of type Execution with sub status 66537"
		result := ParseRoochErrorCode(errorMessage)
		if result == nil || *result != 66537 {
			t.Errorf("Expected 66537, got %v", result)
		}
	})

	t.Run("should return nil if no sub status is found", func(t *testing.T) {
		errorMessage := "status ABORTED of type Execution with no sub status"
		result := ParseRoochErrorCode(errorMessage)
		if result != nil {
			t.Errorf("Expected nil, got %v", result)
		}
	})

	t.Run("should return nil if input is nil", func(t *testing.T) {
		var errorMessage string
		result := ParseRoochErrorCode(errorMessage)
		if result != nil {
			t.Errorf("Expected nil, got %v", result)
		}
	})
}

func TestParseRoochErrorSubStatus(t *testing.T) {
	t.Run("should return the correct sub status", func(t *testing.T) {
		errorMessage := "status ABORTED of type Execution with sub status 66537"
		subStatus := ParseRoochErrorSubStatus(errorMessage)
		if subStatus == nil {
			t.Fatal("Expected non-nil subStatus")
		}
		if subStatus.Category != ErrorCategoryInvalidArgument {
			t.Errorf("Expected category INVALID_ARGUMENT, got %v", subStatus.Category)
		}
		if subStatus.Reason != 1001 {
			t.Errorf("Expected reason 1001, got %v", subStatus.Reason)
		}
	})

	t.Run("should return nil if no sub status is found", func(t *testing.T) {
		errorMessage := "status ABORTED of type Execution with no sub status"
		result := ParseRoochErrorSubStatus(errorMessage)
		if result != nil {
			t.Errorf("Expected nil, got %v", result)
		}
	})

	t.Run("should return nil if input is nil", func(t *testing.T) {
		var errorMessage string
		result := ParseRoochErrorSubStatus(errorMessage)
		if result != nil {
			t.Errorf("Expected nil, got %v", result)
		}
	})
}

func TestGetErrorCategoryName(t *testing.T) {
	t.Run("should return the correct string representation of the enum", func(t *testing.T) {
		testCases := []struct {
			category ErrorCategory
			expected string
		}{
			{ErrorCategoryInvalidArgument, "INVALID_ARGUMENT"},
			{ErrorCategoryOutOfRange, "OUT_OF_RANGE"},
			{ErrorCategoryInvalidState, "INVALID_STATE"},
			{ErrorCategoryUnauthenticated, "UNAUTHENTICATED"},
			{ErrorCategoryPermissionDenied, "PERMISSION_DENIED"},
			{ErrorCategoryNotFound, "NOT_FOUND"},
			{ErrorCategoryAborted, "ABORTED"},
			{ErrorCategoryAlreadyExists, "ALREADY_EXISTS"},
			{ErrorCategoryResourceExhausted, "RESOURCE_EXHAUSTED"},
			{ErrorCategoryCancelled, "CANCELLED"},
			{ErrorCategoryInternal, "INTERNAL"},
			{ErrorCategoryNotImplemented, "NOT_IMPLEMENTED"},
			{ErrorCategoryUnavailable, "UNAVAILABLE"},
		}

		for _, tc := range testCases {
			result := GetErrorCategoryName(tc.category)
			if result != tc.expected {
				t.Errorf("For category %v, expected %s, got %s", tc.category, tc.expected, result)
			}
		}
	})
} 
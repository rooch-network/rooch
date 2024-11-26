// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

package utils

// ErrorCategory represents different types of errors with their corresponding codes
type ErrorCategory uint16

const (
	InvalidArgument    ErrorCategory = 0x1 // Caller specified an invalid argument (http: 400)
	OutOfRange        ErrorCategory = 0x2 // An input or result of a computation is out of range (http: 400)
	InvalidState      ErrorCategory = 0x3 // The system is not in a state where the operation can be performed (http: 400)
	Unauthenticated   ErrorCategory = 0x4 // Request not authenticated due to missing, invalid, or expired auth token (http: 401)
	PermissionDenied  ErrorCategory = 0x5 // client does not have sufficient permission (http: 403)
	NotFound          ErrorCategory = 0x6 // A specified resource is not found (http: 404)
	Aborted           ErrorCategory = 0x7 // Concurrency conflict, such as read-modify-write conflict (http: 409)
	AlreadyExists     ErrorCategory = 0x8 // The resource that a client tried to create already exists (http: 409)
	ResourceExhausted ErrorCategory = 0x9 // Out of gas or other forms of quota (http: 429)
	Cancelled         ErrorCategory = 0xa // Request cancelled by the client (http: 499)
	Internal          ErrorCategory = 0xb // Internal error (http: 500)
	NotImplemented    ErrorCategory = 0xc // Feature not implemented (http: 501)
	Unavailable       ErrorCategory = 0xd // The service is currently unavailable. Indicates that a retry could solve the issue (http: 503)
)

// SubStatus represents the error category and reason
type SubStatus struct {
	Category ErrorCategory
	Reason   uint16
}

// ParseRoochErrorCode parses the error code from a Rooch RPC error message
func ParseRoochErrorCode(errorMessage string) *uint32 {
	if errorMessage == "" {
		return nil
	}

	re := regexp.MustCompile(`sub status (\d+)`)
	match := re.FindStringSubmatch(errorMessage)
	if len(match) < 2 {
		return nil
	}

	code, err := strconv.ParseUint(match[1], 10, 32)
	if err != nil {
		return nil
	}

	result := uint32(code)
	return &result
}

// ParseRoochErrorSubStatus parses the SubStatus from a Rooch RPC error message
func ParseRoochErrorSubStatus(errorMessage string) *SubStatus {
	errorCode := ParseRoochErrorCode(errorMessage)
	if errorCode == nil {
		return nil
	}

	return &SubStatus{
		Category: ErrorCategory(*errorCode >> 16),
		Reason:   uint16(*errorCode & 0xffff),
	}
}

// GetErrorCategoryName returns the string representation of an ErrorCategory
func GetErrorCategoryName(code ErrorCategory) string {
	categoryNames := map[ErrorCategory]string{
		InvalidArgument:    "INVALID_ARGUMENT",
		OutOfRange:        "OUT_OF_RANGE",
		InvalidState:      "INVALID_STATE",
		Unauthenticated:   "UNAUTHENTICATED",
		PermissionDenied:  "PERMISSION_DENIED",
		NotFound:          "NOT_FOUND",
		Aborted:           "ABORTED",
		AlreadyExists:     "ALREADY_EXISTS",
		ResourceExhausted: "RESOURCE_EXHAUSTED",
		Cancelled:         "CANCELLED",
		Internal:          "INTERNAL",
		NotImplemented:    "NOT_IMPLEMENTED",
		Unavailable:       "UNAVAILABLE",
	}

	if name, ok := categoryNames[code]; ok {
		return name
	}
	return "UNKNOWN"
} 
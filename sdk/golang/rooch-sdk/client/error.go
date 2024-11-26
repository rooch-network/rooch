package client

import (
	"regexp"
	"strconv"
)

const (
	ErrorValidateSequenceNuberTooOld            = 1001
	ErrorValidateSequenceNumberTooNew           = 1002
	ErrorValidateAccountDoesNotExist            = 1003
	ErrorValidateCantPayGasDeposit              = 1004
	ErrorValidateTransactionExpired             = 1005
	ErrorValidateBadChainId                     = 1006
	ErrorValidateSequenceNumberTooBig           = 1007
	ErrorValidateMaxGasAmountExceeded           = 1008
	ErrorValidateInvalidAccountAuthKey          = 1009
	ErrorValidateInvalidAuthenticator           = 1010
	ErrorValidateNotInstalledAuthValidator      = 1011
	ErrorValidateSessionIsExpired               = 1012
	ErrorValidateFunctionCallBeyondSessionScope = 1013
)

var codeToErrorType = map[int]string{
	1001: "SequenceNuberTooOld",
	1002: "SequenceNuberTooNew",
	1003: "AccountDoesNotExist",
	1004: "CantPayGasDeposit",
	1005: "TransactionExpired",
	1006: "BadChainId",
	1007: "SequenceNumberTooBig",
	1008: "MaxGasAmountExceeded",
	1009: "InvalidAccountAuthKey",
	1010: "InvalidAuthenticator",
	1011: "NotInstalledAuthValidator",
	1012: "SessionIsExpired",
	1013: "CallFunctionBeyondSessionScop",
}

// RoochHTTPTransportError represents a base error type for Rooch HTTP transport
type RoochHTTPTransportError struct {
	message string
}

func (e *RoochHTTPTransportError) Error() string {
	return e.message
}

// JsonRpcError represents a JSON-RPC specific error
type JsonRpcError struct {
	RoochHTTPTransportError
	Code int
	Type string
}

func NewJsonRpcError(message string, code int) *JsonRpcError {
	err := &JsonRpcError{
		RoochHTTPTransportError: RoochHTTPTransportError{message: message},
		Code:                    code,
	}
	
	if parsedCode := err.ParseSubStatus(); parsedCode != nil {
		err.Code = *parsedCode
	}
	
	if errorType, ok := codeToErrorType[err.Code]; ok {
		err.Type = errorType
	} else {
		err.Type = "ServerError"
	}
	
	return err
}

func (e *JsonRpcError) ParseSubStatus() *int {
	matches := regexp.MustCompile(`sub status (\d+)`).FindStringSubmatch(e.message)
	if len(matches) < 2 {
		return nil
	}
	
	code, err := strconv.Atoi(matches[1])
	if err != nil {
		return nil
	}
	
	result := code & 0xffff
	return &result
}

// RoochHTTPStatusError represents an HTTP status error
type RoochHTTPStatusError struct {
	RoochHTTPTransportError
	Status     int
	StatusText string
}

func NewRoochHTTPStatusError(message string, status int, statusText string) *RoochHTTPStatusError {
	return &RoochHTTPStatusError{
		RoochHTTPTransportError: RoochHTTPTransportError{message: message},
		Status:                  status,
		StatusText:             statusText,
	}
} 
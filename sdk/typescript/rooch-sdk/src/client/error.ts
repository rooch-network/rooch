// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

export const ErrorValidateSequenceNuberTooOld = 1001
export const ErrorValidateSequenceNumberTooNew = 1002
export const ErrorValidateAccountDoesNotExist = 1003
export const ErrorValidateCantPayGasDeposit = 1004
export const ErrorValidateTransactionExpired = 1005
export const ErrorValidateBadChainId = 1006
export const ErrorValidateSequenceNumberTooBig = 1007
export const ErrorValidateMaxGasAmountExceeded = 1008
export const ErrorValidateInvalidAccountAuthKey = 1009
export const ErrorValidateInvalidAuthenticator = 1010
export const ErrorValidateNotInstalledAuthValidator = 1011
export const ErrorValidateSessionIsExpired = 1012
export const ErrorValidateFunctionCallBeyondSessionScope = 1013

const CODE_TO_ERROR_TYPE: Record<number, string> = {
  1001: 'SequenceNuberTooOld',
  1002: 'SequenceNuberTooNew',
  1003: 'AccountDoesNotExist',
  1004: 'CantPayGasDeposit',
  1005: 'TransactionExpired',
  1006: 'BadChainId',
  1007: 'SequenceNumberTooBig',
  1008: 'MaxGasAmountExceeded',
  1009: 'InvalidAccountAuthKey',
  1010: 'InvalidAuthenticator',
  1011: 'NotInstalledAuthValidator',
  1012: 'SessionIsExpired',
  1013: 'CallFunctionBeyondSessionScop',
}

export class RoochHTTPTransportError extends Error {}

export class JsonRpcError extends RoochHTTPTransportError {
  code: number
  type: string

  constructor(message: string, code: number) {
    super(message)
    const parse = this.parseSubStatus()
    this.code = parse || code
    this.type = CODE_TO_ERROR_TYPE[this.code] ?? 'ServerError'
  }

  // Parse rooch RPC error sub status from `status ABORTED of type Execution with sub status 66537`
  parse() {}

  parseSubStatus(): number | null {
    const regex = /sub status (\d+)/
    const match = this.message.match(regex)
    const code = match ? parseInt(match[1]) : null

    return code ? code & 0xffff : null
  }
}

export class RoochHTTPStatusError extends RoochHTTPTransportError {
  status: number
  statusText: string

  constructor(message: string, status: number, statusText: string) {
    super(message)
    this.status = status
    this.statusText = statusText
  }
}

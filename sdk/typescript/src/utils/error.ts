// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

export enum ErrorCategory {
  INVALID_ARGUMENT = 0x1, // Caller specified an invalid argument (http: 400)
  OUT_OF_RANGE = 0x2, // An input or result of a computation is out of range (http: 400)
  INVALID_STATE = 0x3, // The system is not in a state where the operation can be performed (http: 400)
  UNAUTHENTICATED = 0x4, // Request not authenticated due to missing, invalid, or expired auth token (http: 401)
  PERMISSION_DENIED = 0x5, // client does not have sufficient permission (http: 403)
  NOT_FOUND = 0x6, // A specified resource is not found (http: 404)
  ABORTED = 0x7, // Concurrency conflict, such as read-modify-write conflict (http: 409)
  ALREADY_EXISTS = 0x8, // The resource that a client tried to create already exists (http: 409)
  RESOURCE_EXHAUSTED = 0x9, // Out of gas or other forms of quota (http: 429)
  CANCELLED = 0xa, // Request cancelled by the client (http: 499)
  INTERNAL = 0xb, // Internal error (http: 500)
  NOT_IMPLEMENTED = 0xc, // Feature not implemented (http: 501)
  UNAVAILABLE = 0xd, // The service is currently unavailable. Indicates that a retry could solve the issue (http: 503)
}

export interface SubStatus {
  category: ErrorCategory
  reason: number
}

// Parse rooch RPC error sub status from `status ABORTED of type Execution with sub status 66537`
export function parseRoochErrorCode(errorMessage: string | null) {
  if (!errorMessage) {
    return null
  }

  const regex = /sub status (\d+)/
  const match = errorMessage.match(regex)
  return match ? parseInt(match[1]) : null
}

// Parse rooch RPC error sub status from `status ABORTED of type Execution with sub status 66537`
export function parseRoochErrorSubStatus(errorMessage: string | null): SubStatus | null {
  const errorCode = parseRoochErrorCode(errorMessage)
  if (!errorCode) {
    return null
  }

  return {
    category: errorCode >> 16,
    reason: errorCode & 0xffff,
  }
}

// Get the string representation of an enumeration
export function getErrorCategoryName(code: ErrorCategory): string {
  return ErrorCategory[code]
}

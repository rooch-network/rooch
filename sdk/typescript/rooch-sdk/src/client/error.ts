// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { SubStatus } from '@/utils'

const CODE_TO_ERROR_TYPE: Record<number, string> = {
  // 0x1: 'INVALID_ARGUMENT', // Caller specified an invalid argument (http: 400)
  // 0x2: 'OUT_OF_RANGE', // An input or result of a computation is out of range (http: 400)
  // 0x3: 'INVALID_STATE', // The system is not in a state where the operation can be performed (http: 400)
  // 0x4: 'UNAUTHENTICATED', // Request not authenticated due to missing, invalid, or expired auth token (http: 401)
  // 0x5: 'PERMISSION_DENIED', // Client does not have sufficient permission (http: 403)
  // 0x6: 'NOT_FOUND', // A specified resource is not found (http: 404)
  // 0x7: 'ABORTED', // Concurrency conflict, such as read-modify-write conflict (http: 409)
  // 0x8: 'ALREADY_EXISTS', // The resource that a client tried to create already exists (http: 409)
  // 0x9: 'RESOURCE_EXHAUSTED', // Out of gas or other forms of quota (http: 429)
  // 0xa: 'CANCELLED', // Request cancelled by the client (http: 499)
  // 0xb: 'INTERNAL', // Internal error (http: 500)
  // 0xc: 'NOT_IMPLEMENTED', // Feature not implemented (http: 501)
  // 0xd: 'UNAVAILABLE', // The service is currently unavailable. Indicates that a retry could solve the issue (http: 503)
}

export class RoochHTTPTransportError extends Error {}

export class JsonRpcError extends RoochHTTPTransportError {
  code: number
  type: string

  constructor(message: string, code: number) {
    super(message)
    this.code = code
    this.type = CODE_TO_ERROR_TYPE[code] ?? 'ServerError'
  }

  // Parse rooch RPC error sub status from `status ABORTED of type Execution with sub status 66537`
  parse() {}

  parseSubStatus(): SubStatus | null {
    const regex = /sub status (\d+)/
    const match = this.message.match(regex)
    const code = match ? parseInt(match[1]) : null

    return code
      ? {
          category: code >> 16,
          reason: code & 0xffff,
        }
      : null
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

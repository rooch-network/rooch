// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

export interface JsonRpcRequest {
  method: string
  params?: any[] | object
  id?: string | number | null
}

export interface JsonRpcResponse {
  result?: any
  error?: {
    code: number
    message: string
    data?: any
  }
  id: string | number | null
}

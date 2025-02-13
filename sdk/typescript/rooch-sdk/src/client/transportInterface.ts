// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

export interface RoochTransportRequestOptions {
  method: string
  params: unknown[]
}

export interface RoochTransport {
  request<T = unknown>(input: RoochTransportRequestOptions): Promise<T>
}

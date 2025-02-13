// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// import { PACKAGE_VERSION, TARGETED_RPC_VERSION } from '../version.js'

import { JsonRpcError, RoochHTTPStatusError } from './error.js'
import { RoochTransport, RoochTransportRequestOptions } from './transportInterface.js'

export type HttpHeaders = { [header: string]: string }

export interface RoochHTTPTransportOptions {
  fetch?: typeof fetch
  url: string
  rpc?: {
    headers?: HttpHeaders
    url?: string
  }
}

export class RoochHTTPTransport implements RoochTransport {
  #requestId = 0
  #options: RoochHTTPTransportOptions

  constructor(options: RoochHTTPTransportOptions) {
    this.#options = options
  }

  fetch(input: RequestInfo, init?: RequestInit): Promise<Response> {
    const fetchFn = this.#options.fetch ?? fetch

    if (!fetchFn) {
      throw new Error(
        'The current environment does not support fetch, you can provide a fetch implementation in the options for RoochHTTPTransport.',
      )
    }

    return fetchFn(input, init)
  }

  async request<T>(input: RoochTransportRequestOptions): Promise<T> {
    this.#requestId += 1

    const res = await this.fetch(this.#options.rpc?.url ?? this.#options.url, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        ...this.#options.rpc?.headers,
      },
      body: JSON.stringify({
        jsonrpc: '2.0',
        id: this.#requestId,
        method: input.method,
        params: input.params,
      }),
    })

    if (!res.ok) {
      throw new RoochHTTPStatusError(
        `Unexpected status code: ${res.status}`,
        res.status,
        res.statusText,
      )
    }

    const data = await res.json()

    if ('error' in data && data.error != null) {
      throw new JsonRpcError(data.error.message, data.error.code)
    }

    return data.result
  }
}

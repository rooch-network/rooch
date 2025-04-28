// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// import { PACKAGE_VERSION, TARGETED_RPC_VERSION } from '../version.js'

import { JsonRpcError, RoochHTTPStatusError } from './error.js'
import { SSEClient } from './sseTransport.js'
import {
  RoochSSETransportSubscribeOptions,
  RoochTransport,
  RoochTransportRequestOptions,
  RoochTransportSubscribeOptions,
} from './transportInterface.js'
import { WebsocketClient, WebsocketClientOptions } from './wsTransport.js'

export type HttpHeaders = { [header: string]: string }

export interface RoochHTTPTransportOptions {
  fetch?: typeof fetch
  WebSocketConstructor?: typeof WebSocket
  url: string
  rpc?: {
    headers?: HttpHeaders
    url?: string
  }
  websocket?: WebsocketClientOptions & {
    url?: string
  }
}

export class RoochHTTPTransport implements RoochTransport {
  #requestId = 0
  #options: RoochHTTPTransportOptions
  #websocketClient?: WebsocketClient
  #sseClient?: SSEClient

  constructor(options: RoochHTTPTransportOptions) {
    this.#options = options
  }

  #getRoochSSETransport(): SSEClient {
    if (!this.#sseClient) {
      this.#sseClient = new SSEClient(this.#options.url)
    }

    return this.#sseClient
  }

  #getWebsocketClient(): WebsocketClient {
    if (!this.#websocketClient) {
      const WebSocketConstructor = this.#options.WebSocketConstructor ?? WebSocket
      if (!WebSocketConstructor) {
        throw new Error(
          'The current environment does not support WebSocket, you can provide a WebSocketConstructor in the options for SuiHTTPTransport.',
        )
      }

      this.#websocketClient = new WebsocketClient(
        this.#options.websocket?.url ?? this.#options.url,
        {
          WebSocketConstructor,
          ...this.#options.websocket,
        },
      )
    }

    return this.#websocketClient
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

  async subscribeWithSSE<T>(
    input: RoochSSETransportSubscribeOptions<T>,
  ): Promise<() => Promise<boolean>> {
    const unsubscribe = await this.#getRoochSSETransport().subscribe(input)

    return async () => !!(await unsubscribe())
  }

  async subscribe<T>(input: RoochTransportSubscribeOptions<T>): Promise<() => Promise<boolean>> {
    const unsubscribe = await this.#getWebsocketClient().subscribe(input)

    if (input.signal) {
      input.signal.throwIfAborted()
      input.signal.addEventListener('abort', () => {
        unsubscribe()
      })
    }

    return async () => !!(await unsubscribe())
  }

  destroy(): void {
    // HTTP is stateless, no cleanup needed
  }
}

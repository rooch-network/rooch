// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { JsonRpcError } from './error.js'
import { RoochTransport, RoochTransportRequestOptions } from './transportInterface.js'

export interface RoochWebSocketTransportOptions {
  url: string
  WebSocket?: typeof WebSocket
  protocols?: string | string[]
  reconnectDelay?: number
  maxReconnectAttempts?: number
  requestTimeout?: number
}

interface WsRequest {
  resolve: (value: any) => void
  reject: (error: Error) => void
  method: string
  params: unknown[]
  timestamp: number
}

export class RoochWebSocketTransport implements RoochTransport {
  #ws: WebSocket | null = null
  #requestId = 0
  #options: RoochWebSocketTransportOptions
  #pendingRequests = new Map<number, WsRequest>()
  #reconnectAttempts = 0
  readonly #maxReconnectAttempts: number
  readonly #reconnectDelay: number
  readonly #requestTimeout: number
  readonly #WebSocketImpl: typeof WebSocket

  constructor(options: RoochWebSocketTransportOptions) {
    this.#options = options
    this.#maxReconnectAttempts = options.maxReconnectAttempts ?? 5
    this.#reconnectDelay = options.reconnectDelay ?? 1000
    this.#requestTimeout = options.requestTimeout ?? 30000
    this.#WebSocketImpl = options.WebSocket ?? WebSocket
    this.#connect()
  }

  #connect(): void {
    if (this.#ws?.readyState === WebSocket.CONNECTING || this.#ws?.readyState === WebSocket.OPEN)
      return

    this.#ws = new this.#WebSocketImpl(this.#options.url, this.#options.protocols)

    this.#ws.onopen = () => {
      this.#reconnectAttempts = 0
    }

    this.#ws.onclose = () => {
      this.#handleReconnect()
    }

    this.#ws.onmessage = (event) => {
      try {
        const response = JSON.parse(event.data)
        const request = this.#pendingRequests.get(response.id)
        if (!request) return

        this.#pendingRequests.delete(response.id)
        if ('error' in response && response.error != null) {
          request.reject(new JsonRpcError(response.error.message, response.error.code))
        } else {
          request.resolve(response.result)
        }
      } catch (error) {
        console.error('Failed to parse WebSocket message:', error)
      }
    }
  }

  #handleReconnect(): void {
    if (this.#reconnectAttempts >= this.#maxReconnectAttempts) {
      this.#rejectAllPending(new Error('WebSocket connection failed'))
      return
    }

    this.#reconnectAttempts++
    setTimeout(() => this.#connect(), this.#reconnectDelay * this.#reconnectAttempts)
  }

  #rejectAllPending(error: Error): void {
    for (const request of this.#pendingRequests.values()) {
      request.reject(error)
    }
    this.#pendingRequests.clear()
  }

  async request<T>(input: RoochTransportRequestOptions): Promise<T> {
    if (!this.#ws || this.#ws.readyState !== WebSocket.OPEN) {
      throw new Error('WebSocket is not connected')
    }

    return new Promise((resolve, reject) => {
      const id = ++this.#requestId
      const request: WsRequest = {
        resolve,
        reject,
        method: input.method,
        params: input.params,
        timestamp: Date.now(),
      }

      this.#pendingRequests.set(id, request)
      this.#ws!.send(
        JSON.stringify({
          jsonrpc: '2.0',
          id,
          method: input.method,
          params: input.params,
        }),
      )

      setTimeout(() => {
        if (this.#pendingRequests.has(id)) {
          this.#pendingRequests.delete(id)
          reject(new Error('Request timeout'))
        }
      }, this.#requestTimeout)
    })
  }

  disconnect(): void {
    this.#ws?.close()
    this.#rejectAllPending(new Error('WebSocket disconnected'))
  }
}

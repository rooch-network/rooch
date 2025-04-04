// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import WebSocket from 'ws'
import { EventEmitter } from 'events'
import { JsonRpcRequest } from './types/jsonRpc.js'
import { Subscription, RoochSubscriptionTransport } from './subscriptionTransportInterface.js'
import { RoochTransport, RoochTransportRequestOptions } from './transportInterface.js'
import { JsonRpcError } from './error.js'

export interface RoochWebSocketTransportOptions {
  url: string
  WebSocket?: typeof WebSocket
  protocols?: string | string[]
  reconnectDelay?: number
  maxReconnectAttempts?: number
  requestTimeout?: number
  connectionReadyTimeout?: number
}

interface WsRequest {
  resolve: (value: any) => void
  reject: (error: Error) => void
  method: string
  params: any
  timestamp: number
}

export class RoochWebSocketTransport implements RoochTransport, RoochSubscriptionTransport {
  #ws: WebSocket | null = null
  #requestId = 0
  #options: RoochWebSocketTransportOptions
  #pendingRequests = new Map<number, WsRequest>()
  #reconnectAttempts = 0
  #subscriptions = new Map<string, { request: JsonRpcRequest; id: string }>()
  #eventEmitter = new EventEmitter()
  readonly #maxReconnectAttempts: number
  readonly #reconnectDelay: number
  readonly #requestTimeout: number
  readonly #connectionReadyTimeout: number
  readonly #WebSocketImpl: typeof WebSocket

  constructor(options: RoochWebSocketTransportOptions) {
    this.#options = options
    this.#maxReconnectAttempts = this.#validateMaxReconnectAttempts(
      options.maxReconnectAttempts ?? 5,
    )
    this.#reconnectDelay = options.reconnectDelay ?? 1000
    this.#requestTimeout = options.requestTimeout ?? 30000
    this.#connectionReadyTimeout = options.connectionReadyTimeout ?? 5000
    this.#WebSocketImpl = options.WebSocket ?? WebSocket
    this.#connect()
  }

  #validateMaxReconnectAttempts(attempts: number): number {
    if (attempts < 0 || attempts > 10) {
      throw new Error('maxReconnectAttempts must be between 0 and 10')
    }
    return attempts
  }

  #connect(): void {
    if (this.#ws?.readyState === WebSocket.CONNECTING || this.#ws?.readyState === WebSocket.OPEN)
      return

    this.#ws = new this.#WebSocketImpl(this.#options.url, this.#options.protocols)

    this.#ws.onopen = () => {
      this.#reconnectAttempts = 0
      this.#eventEmitter.emit('reconnected')
      this.#resubscribeAll()
    }

    this.#ws.onmessage = (event: any) => {
      try {
        const data = JSON.parse(event.data)

        console.log('onmessage data:', data)

        // Handle subscription events
        if (data.method && data.method.startsWith('rooch_subscribe') && data.params) {
          this.#eventEmitter.emit('subscription', data)
          return
        }

        // Handle regular RPC responses
        const request = this.#pendingRequests.get(data.id)
        if (request) {
          this.#pendingRequests.delete(data.id)
          if ('error' in data && data.error != null) {
            request.reject(new JsonRpcError(data.error.message, data.error.code))
          } else {
            request.resolve(data.result)
          }
        }
      } catch (error) {
        console.error('Failed to parse WebSocket message:', error)
      }
    }

    this.#ws.onclose = () => {
      this.#handleReconnect()
    }

    this.#ws.onerror = (event: WebSocket.ErrorEvent) => {
      console.error(`websocket_error:`, event)
      this.#handleReconnect()
    }
  }

  #handleReconnect(): void {
    if (this.#reconnectAttempts >= this.#maxReconnectAttempts) {
      this.#rejectAllPending(
        new Error('WebSocket connection failed after maximum reconnection attempts'),
      )
      this.#eventEmitter.emit(
        'error',
        new Error('WebSocket connection failed after maximum reconnection attempts'),
      )
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

  #resubscribeAll(): void {
    for (const [subscriptionId, { request }] of this.#subscriptions.entries()) {
      // Don't wait for the promise since we're just restoring subscriptions
      this.subscribe(request).catch((error) => {
        console.error(`Failed to resubscribe to ${subscriptionId}:`, error)
        this.#eventEmitter.emit('error', error)
      })
    }
  }

  async #ensureConnection(): Promise<void> {
    if (!this.#ws || this.#ws.readyState !== WebSocket.OPEN) {
      const startTime = Date.now()
      while (true) {
        if (this.#ws?.readyState === WebSocket.OPEN) break

        if (Date.now() - startTime >= this.#connectionReadyTimeout) {
          throw new Error(`WebSocket connection not ready within ${this.#connectionReadyTimeout}ms`)
        }

        await new Promise((resolve) => setTimeout(resolve, 100))

        if (this.#ws?.readyState === WebSocket.CLOSED) {
          this.#connect()
        }
      }
    }
  }

  // RoochTransport implementation
  async request<T>(input: RoochTransportRequestOptions): Promise<T> {
    try {
      await this.#ensureConnection()

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
    } catch (error) {
      throw error
    }
  }

  // RoochSubscriptionTransport implementation
  async subscribe(request: JsonRpcRequest): Promise<Subscription> {
    try {
      await this.#ensureConnection()

      const id = ++this.#requestId

      return new Promise((resolve, reject) => {
        const wsRequest: WsRequest = {
          resolve: (result) => {
            const subscriptionId = result
            this.#subscriptions.set(subscriptionId, {
              request,
              id: subscriptionId,
            })

            resolve({
              id: subscriptionId,
              unsubscribe: () => this.unsubscribe(subscriptionId),
            })
          },
          reject,
          method: request.method,
          params: request.params || [],
          timestamp: Date.now(),
        }

        this.#pendingRequests.set(id, wsRequest)

        this.#ws!.send(
          JSON.stringify({
            jsonrpc: '2.0',
            id,
            method: request.method,
            params: request.params || [],
          }),
        )

        setTimeout(() => {
          if (this.#pendingRequests.has(id)) {
            this.#pendingRequests.delete(id)
            reject(new Error('Subscription request timeout'))
          }
        }, this.#requestTimeout)
      })
    } catch (error) {
      this.#eventEmitter.emit('error', error)
      throw error
    }
  }

  unsubscribe(subscriptionId: string): void {
    if (!this.#subscriptions.has(subscriptionId)) return

    if (this.#ws?.readyState === WebSocket.OPEN) {
      const id = ++this.#requestId
      this.#ws.send(
        JSON.stringify({
          jsonrpc: '2.0',
          id,
          method: 'rooch_unsubscribe',
          params: [subscriptionId],
        }),
      )
    }

    this.#subscriptions.delete(subscriptionId)
  }

  onMessage(callback: (msg: any) => void): void {
    this.#eventEmitter.on('subscription', callback)
  }

  onReconnected(callback: () => void): void {
    this.#eventEmitter.on('reconnected', callback)
  }

  onError(callback: (error: Error) => void): void {
    this.#eventEmitter.on('error', callback)
  }

  destroy(): void {
    if (this.#ws) {
      this.#ws.close()
      this.#ws = null
    }

    this.#rejectAllPending(new Error('WebSocket disconnected'))
    this.#eventEmitter.removeAllListeners()
    this.#subscriptions.clear()
    this.#pendingRequests.clear()
  }
}

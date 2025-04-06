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
  heartbeatInterval?: number // Interval between ping frames in ms
  heartbeatTimeout?: number // Time to wait for pong response in ms
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
  #eventEmitter = new EventEmitter()
  #heartbeatTimer: NodeJS.Timeout | null = null
  #awaitingPong: boolean = false
  #pongTimeoutTimer: NodeJS.Timeout | null = null
  #isDestroying: boolean = false
  readonly #maxReconnectAttempts: number
  readonly #reconnectDelay: number
  readonly #requestTimeout: number
  readonly #connectionReadyTimeout: number
  readonly #WebSocketImpl: typeof WebSocket
  readonly #heartbeatInterval: number
  readonly #heartbeatTimeout: number

  constructor(options: RoochWebSocketTransportOptions) {
    this.#options = options
    this.#maxReconnectAttempts = this.#validateMaxReconnectAttempts(
      options.maxReconnectAttempts ?? 5,
    )
    this.#reconnectDelay = options.reconnectDelay ?? 1000
    this.#requestTimeout = options.requestTimeout ?? 30000
    this.#connectionReadyTimeout = options.connectionReadyTimeout ?? 5000
    this.#WebSocketImpl = options.WebSocket ?? WebSocket
    this.#heartbeatInterval = options.heartbeatInterval ?? 30000
    this.#heartbeatTimeout = options.heartbeatTimeout ?? 5000
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
      this.#startHeartbeat()
    }

    // Add pong handler to reset awaiting state
    this.#ws.on('pong', () => {
      this.#awaitingPong = false
      if (this.#pongTimeoutTimer) {
        clearTimeout(this.#pongTimeoutTimer)
        this.#pongTimeoutTimer = null
      }
    })

    this.#ws.onmessage = (event: any) => {
      try {
        const data = JSON.parse(event.data)

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

    this.#ws.onclose = (event: WebSocket.CloseEvent) => {
      console.error(`websocket_close:`, event.code, event.reason)
      this.#stopHeartbeat()
      this.#handleReconnect()
    }

    this.#ws.onerror = (event: WebSocket.ErrorEvent) => {
      console.error(`websocket_error:`, event.message, event.error)
      this.#stopHeartbeat()
      this.#handleReconnect()
    }
  }

  #handleReconnect(): void {
    console.error(`handleReconnect: reconnecting...`)

    // Skip reconnection if we're intentionally destroying the transport
    if (this.#isDestroying) {
      console.log('Skipping reconnect during intentional shutdown')
      return
    }

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

    // Ensure WebSocket is cleaned up before reconnecting
    if (this.#ws) {
      this.#cleanupWebSocket()
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
  }

  // Add heartbeat methods
  #startHeartbeat(): void {
    if (this.#heartbeatInterval <= 0) return

    this.#stopHeartbeat() // Clear any existing timers

    this.#heartbeatTimer = setInterval(() => {
      if (this.#ws?.readyState === WebSocket.OPEN) {
        // If we're still waiting for a pong from previous ping, connection might be dead
        if (this.#awaitingPong) {
          console.warn('No pong received within timeout, triggering reconnection')
          this.#ws.terminate() // Force close the connection to trigger reconnect
          return
        }

        try {
          this.#awaitingPong = true
          this.#ws.ping()

          // Set timeout for pong response
          this.#pongTimeoutTimer = setTimeout(() => {
            if (this.#awaitingPong && this.#ws?.readyState === WebSocket.OPEN) {
              console.warn('Pong timeout reached, terminating connection')
              this.#ws.terminate() // Force close to trigger reconnect
            }
          }, this.#heartbeatTimeout)
        } catch (error) {
          console.error('Failed to send ping:', error)
          this.#handleReconnect()
        }
      }
    }, this.#heartbeatInterval)
  }

  #stopHeartbeat(): void {
    if (this.#heartbeatTimer) {
      clearInterval(this.#heartbeatTimer)
      this.#heartbeatTimer = null
    }

    if (this.#pongTimeoutTimer) {
      clearTimeout(this.#pongTimeoutTimer)
      this.#pongTimeoutTimer = null
    }

    this.#awaitingPong = false
  }

  // Add a helper method to properly clean up the WebSocket instance
  #cleanupWebSocket(): void {
    if (this.#ws) {
      try {
        // Remove all listeners to avoid memory leaks
        this.#ws.onopen = null
        this.#ws.onclose = null
        this.#ws.onerror = null
        this.#ws.onmessage = null
        this.#ws.removeAllListeners('pong')

        // Force close the connection
        this.#ws.terminate()
      } catch (error) {
        console.error('Error while cleaning up WebSocket:', error)
      } finally {
        // Always set the WebSocket to null to ensure it's garbage collected
        this.#ws = null
      }
    }
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
    // Set flag to prevent reconnection
    this.#isDestroying = true

    this.#stopHeartbeat()

    if (this.#ws) {
      this.#ws.close()
      this.#ws = null
    }

    this.#rejectAllPending(new Error('WebSocket disconnected'))
    this.#eventEmitter.removeAllListeners()
    this.#pendingRequests.clear()
  }
}

// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
import { EventSource } from 'eventsource'

type SubscriptionRequest<T = any> = {
  method: string
  params: any
  onMessage: (event: T) => void
  onError?: (error: Error) => void
  signal?: AbortSignal
}

/**
 * Configuration options for the SSE connection
 */
export type SSEClientOptions = {
  /**
   * Milliseconds before timing out while calling an RPC method
   */
  callTimeout?: number
  /**
   * Milliseconds between attempts to connect
   */
  reconnectTimeout?: number
  /**
   * Maximum number of times to try connecting before giving up
   */
  maxReconnects?: number
}

export const DEFAULT_CLIENT_OPTIONS = {
  callTimeout: 30000,
  reconnectTimeout: 3000,
  maxReconnects: 5,
} satisfies SSEClientOptions

export class SSEClient {
  endpoint: string
  options: Required<SSEClientOptions>
  #requestId = 0
  #disconnects = 0
  #eventSource: Map<number, EventSource> = new Map()
  #connectionPromise: Map<number, Promise<EventSource>> = new Map()
  #subscriptionRequests: Map<number, SubscriptionRequest> = new Map()
  #subscriptions = new Set<RpcSubscription>()

  constructor(endpoint: string, options: SSEClientOptions = {}) {
    this.endpoint = endpoint
    this.options = { ...DEFAULT_CLIENT_OPTIONS, ...options }
  }

  async disconnect(id: number) {
    const eventSource = this.#eventSource.get(id)
    if (eventSource) {
      eventSource.close()
      this.#eventSource.delete(id)
      this.#connectionPromise.delete(id)
    }

    return Promise.resolve(eventSource !== undefined)
  }

  getSubscribeId() {
    return this.#requestId
  }

  setupEventSource(request: SubscriptionRequest) {
    this.#requestId += 1
    this.#subscriptionRequests.set(this.#requestId, request)

    const promise = new Promise<EventSource>((resolve) => {
      if (this.#eventSource.has(this.#requestId)) {
        this.#eventSource.get(this.#requestId)?.close()
      }

      const url = new URL(`${this.endpoint}${request.method}`)
      url.searchParams.set('filter', JSON.stringify(request.params))

      const eventSource = new EventSource(url.toString())

      eventSource.onopen = () => {
        this.#disconnects = 0
        resolve(eventSource)
      }

      eventSource.onerror = (error) => {
        this.#disconnects++
        if (this.#disconnects <= this.options.maxReconnects) {
          setTimeout(() => {
            this.#reconnect(this.#requestId)
          }, this.options.reconnectTimeout)
        } else {
          console.error(
            `Failed to connect after ${this.options.maxReconnects} attempts`,
            error.message,
          )
          // Trigger onError callback
          this.#subscriptions.forEach((subscription) => {
            if (subscription.subscriptionId === this.#requestId) {
              subscription.onError?.(
                new Error(`Failed to connect after ${this.options.maxReconnects} attempts`),
              )
            }
          })
        }
      }

      eventSource.onmessage = (data) => {
        let json: any
        try {
          // Parse the data if it's a string
          json = typeof data.data === 'string' ? JSON.parse(data.data) : data.data
        } catch (error) {
          console.error(new Error(`Failed to parse RPC message: ${data.data}`, { cause: error }))
          return
        }

        this.#subscriptions.forEach((subscription) => {
          subscription.onMessage(json)
        })
      }
    })

    this.#connectionPromise.set(this.#requestId, promise)

    return promise
  }

  async #reconnect(id: number) {
    if (this.#eventSource.has(id)) {
      this.#eventSource.get(id)?.close()
      this.#eventSource.delete(id)
      this.#connectionPromise.delete(id)
    }

    try {
      if (!this.#subscriptionRequests.has(id)) {
        return
      }
      const eventSource = await this.setupEventSource(this.#subscriptionRequests.get(id)!)
      this.#eventSource.set(id, eventSource)
      return Promise.allSettled(
        [...this.#subscriptions]
          .filter((subscription) => subscription.subscriptionId === id)
          .map((subscription) => subscription.subscribe(this)),
      )
    } catch (error) {
      console.error(`Failed to reconnect: ${error}`)
      throw error
    }
  }

  async subscribe<T>(input: SubscriptionRequest<T>) {
    const subscription = new RpcSubscription(input)
    this.#subscriptions.add(subscription)
    await subscription.subscribe(this)
    return () => subscription.unsubscribe(this)
  }
}

class RpcSubscription {
  subscriptionId: number | null = null
  input: SubscriptionRequest<any>
  subscribed = false
  onError?: (error: Error) => void

  constructor(input: SubscriptionRequest) {
    this.input = input
    this.onError = input.onError
  }

  onMessage(message: unknown) {
    if (this.subscribed) {
      this.input.onMessage(message)
    }
  }

  async unsubscribe(client: SSEClient) {
    const { subscriptionId } = this
    this.subscribed = false
    if (subscriptionId == null) return false
    this.subscriptionId = null

    await client.disconnect(subscriptionId)
    return Promise.resolve(true)
  }

  async subscribe(client: SSEClient) {
    this.subscriptionId = null
    try {
      const es = await client.setupEventSource(this.input)
      if (es) {
        this.subscribed = true
        this.subscriptionId = client.getSubscribeId()
      }
    } catch (error) {
      this.subscribed = false
      throw error
    }
  }
}

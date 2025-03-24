// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

export interface SubscriptionCallbacks<T> {
  onMessage: (data: T) => void
  onError: (error: Error) => void
}

export interface Subscription {
  unsubscribe: () => void
}

export interface Provider {
  /**
   * Check if the provider supports WebSocket connections
   */
  isWebSocket(): boolean

  /**
   * Subscribe to a WebSocket event stream
   * @param method The subscription method name
   * @param params Parameters for the subscription
   * @param callbacks Callbacks for handling messages and errors
   */
  subscribe<T>(method: string, params: any[], callbacks: SubscriptionCallbacks<T>): Subscription
} 
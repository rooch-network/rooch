// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { JsonRpcRequest } from './types/index.js'

export interface Subscription {
  id: string // Subscription ID
  unsubscribe: () => void // Method to cancel the subscription
}

export interface RoochSubscriptionTransport {
  /**
   * Subscribe to a specific request
   * @param request JSON-RPC request object containing method and parameters
   * @returns A Subscription object containing the subscription ID and an unsubscribe method
   */
  subscribe(request: JsonRpcRequest): Promise<Subscription>

  /**
   * Unsubscribe from a specific subscription
   * @param subscriptionId The subscription ID
   */
  unsubscribe(subscriptionId: string): void

  /**
   * Register a callback to handle subscription events
   * @param callback Function to handle subscription events
   */
  onMessage(callback: (msg: any) => void): void

  /**
   * Register a callback to handle reconnection events
   * @param callback Function called when the connection is re-established
   */
  onReconnected(callback: () => void): void

  /**
   * Register a callback to handle transport-level errors
   * @param callback Function to handle errors such as failed reconnections or network issues
   */
  onError(callback: (error: Error) => void): void

  /**
   * Destroy transport layer resources and close the connection
   */
  destroy(): void
}

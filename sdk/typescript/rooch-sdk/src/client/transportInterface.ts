// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

export interface RoochTransportRequestOptions {
  method: string
  params: unknown[]
}

export interface RoochSSETransportSubscribeOptions<T> {
  method: string
  params: unknown
  onMessage: (event: T) => void
  onError?: (error: Error) => void
  signal?: AbortSignal
}

export interface RoochTransportSubscribeOptions<T> {
  method: string
  params: unknown[]
  onMessage: (event: T) => void
  signal?: AbortSignal
}

export interface RoochTransport {
  /**
   * Send a request to the Rooch node
   * @param input Request options containing method and parameters
   * @returns Promise resolving to the response
   */
  request<T = unknown>(input: RoochTransportRequestOptions): Promise<T>

  subscribeWithSSE<T = unknown>(
    input: RoochSSETransportSubscribeOptions<T>,
  ): Promise<() => Promise<boolean>>

  subscribe<T = unknown>(input: RoochTransportSubscribeOptions<T>): Promise<() => Promise<boolean>>

  /**
   * Clean up resources and close connections
   * Should be called when the transport is no longer needed
   */
  destroy(): void
}

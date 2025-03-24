// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { IndexerEventView, SubscriptionOptions, TransactionWithInfoView } from './types'
import { Provider } from './providers/provider'

export class RoochClient {
  private readonly provider: Provider

  constructor(provider: Provider) {
    this.provider = provider
  }

  /**
   * Subscribe to events matching the given filter
   * @param options Subscription options including filter and callbacks
   * @returns Unsubscribe function
   */
  public subscribeEvents(options: SubscriptionOptions<IndexerEventView>): () => void {
    if (!this.provider.isWebSocket()) {
      throw new Error('Event subscription requires WebSocket transport')
    }

    const subscription = this.provider.subscribe('rooch_subscribeEvents', [options.filter], {
      onMessage: options.onMessage,
      onError: options.onError,
    })

    return () => {
      subscription.unsubscribe()
    }
  }

  /**
   * Subscribe to transactions matching the given filter
   * @param options Subscription options including filter and callbacks
   * @returns Unsubscribe function
   */
  public subscribeTransactions(options: SubscriptionOptions<TransactionWithInfoView>): () => void {
    if (!this.provider.isWebSocket()) {
      throw new Error('Transaction subscription requires WebSocket transport')
    }

    const subscription = this.provider.subscribe('rooch_subscribeTransactions', [options.filter], {
      onMessage: options.onMessage,
      onError: options.onError,
    })

    return () => {
      subscription.unsubscribe()
    }
  }
}

export type {
  IndexerEventView,
  TransactionWithInfoView,
  SubscriptionOptions,
} 
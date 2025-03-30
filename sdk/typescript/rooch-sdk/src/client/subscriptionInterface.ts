// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { EventFilterView, TransactionFilterView } from './types/index.js'

interface IndexerEventView {
  indexer_event_id: { tx_order: string; event_index: string }
  event_id: { event_handle_id: string; event_seq: string }
  event_type: string
  event_data: string
  tx_hash: string
  sender: string
  created_at: string
  decoded_event_data?: any
}

interface TransactionWithInfoView {
  transaction: {
    data:
      | {
          type: 'l1_block'
          data: {
            chain_id: string
            block_height: string
            block_hash: string
            bitcoin_block_hash?: string
          }
        }
      | {
          type: 'l1_tx'
          data: {
            chain_id: string
            block_hash: string
            bitcoin_block_hash?: string
            txid: string
            bitcoin_txid?: string
          }
        }
      | { type: 'l2_tx'; data: { sender: string /* + other fields */ } }
    sequence_info: { tx_order: string; tx_timestamp: string /* + other fields */ }
  }
  execution_info?: { tx_hash: string /* + other fields */ }
}

type SubscriptionEvent =
  | { type: 'event'; data: IndexerEventView }
  | { type: 'transaction'; data: TransactionWithInfoView }

export interface SubscriptionOptions {
  type: 'event' | 'transaction' // Subscription type
  filter?: EventFilterView | TransactionFilterView // Optional Rust-defined filter
  onEvent: (event: SubscriptionEvent) => void // Callback for received events
  onError?: (error: Error) => void // Optional callback for errors
}

export interface Subscription {
  id: string // Unique subscription identifier
  unsubscribe: () => void // Method to cancel the subscription
}

export interface RoochSubscription {
  /**
   * Subscribe to events or transactions
   * @param options Subscription options including type, filter, onEvent, onError
   * @returns A Subscription object containing the subscription ID and unsubscribe method
   */
  subscribe(options: SubscriptionOptions): Subscription

  /**
   * Unsubscribe from a specific subscription
   * @param subscriptionId The subscription ID
   */
  unsubscribe(subscriptionId: string): void

  /**
   * Destroy the transport layer resources and close the connection
   */
  destroy(): void
}

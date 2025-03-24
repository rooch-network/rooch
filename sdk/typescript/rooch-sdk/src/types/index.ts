// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

export * from './bytes.js'
export * from './rooch.js'

export interface IndexerEventID {
  tx_order: string;
  event_index: string;
}

export interface IndexerEventView {
  indexer_event_id: IndexerEventID;
  event_id: string;
  event_type: string;
  event_data: string;
  tx_hash: string;
  sender: string;
  created_at: string;
  decoded_event_data?: any;
}

export interface TransactionData {
  type: string;
  // Add other transaction data fields as needed
}

export interface TransactionView {
  data: TransactionData;
  sequence_info: {
    tx_order: string;
  };
}

export interface TransactionWithInfoView {
  transaction: TransactionView;
  execution_info?: {
    tx_hash: string;
    status: {
      type: string;
    };
  };
}

export interface SubscriptionOptions<T> {
  filter: any;
  onMessage: (data: T) => void;
  onError: (error: Error) => void;
}

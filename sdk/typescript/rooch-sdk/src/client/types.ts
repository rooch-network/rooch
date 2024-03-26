// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { RoochMultiChainID } from '../constants'
import {
  Arg,
  FunctionId,
  GlobalStateFilterView,
  IndexerStateID,
  InscriptionFilterView,
  RoochAccountAddress,
  TableStateFilterView,
  TransactionFilterView,
  TypeTag,
  u64,
  usize,
  UTXOFilterView,
} from '../types'

export interface ExecuteViewFunctionParams {
  funcId: FunctionId
  tyArgs?: TypeTag[]
  args?: Arg[]
}

export interface ResoleRoochAddressParams {
  address: string
  multiChainID: RoochMultiChainID
}

export interface ListStatesParams {
  accessPath: string
  cursor: string | null
  limit: number
}

export interface QueryGlobalStatesParams {
  filter: GlobalStateFilterView
  cursor: IndexerStateID | null
  limit: number
  descending_order: boolean
}

export interface QueryTableStatesParams {
  filter: TableStateFilterView
  cursor?: IndexerStateID | null
  limit: number
  descending_order: boolean
}

export interface QueryInscriptionsParams {
  filter?: InscriptionFilterView | null
  cursor?: IndexerStateID | null
  limit: number
  descending_order: boolean
}

export interface QueryUTXOsParams {
  filter?: UTXOFilterView | null
  cursor?: IndexerStateID | null
  limit: number
  descending_order: boolean
}

export interface GetTransactionsParams {
  cursor: number
  limit: number
  descending_order: boolean
}

export interface GetEventsParams {
  eventHandleType: string
  cursor: number
  limit: number
  descending_order: boolean
}

export type QueryTransactionFilterParams =
  | { sender: RoochAccountAddress }
  | { original_address: string }
  | { tx_hashes: string[] }
  | { time_range: { end_time: number; start_time: number } }
  | { tx_order_range: { from_order: number; to_order: number } }

export interface QueryTransactionParams {
  filter: TransactionFilterView
  cursor: u64
  limit: usize
  descending_order: boolean
}

export type QueryEventFilterParams =
  | { event_type: string }
  | { sender: string }
  | { tx_hash: string }
  | { time_range: { end_time: number; start_time: number } }
  | { tx_order_range: { from_order: number; to_order: number } }

export interface QueryEventParams {
  filter: QueryEventFilterParams
  cursor: { event_index: number; tx_order: number }
  limit: usize
  descending_order: boolean
}

export interface GetBalanceParams {
  address: string
  coinType: string
}

export interface GetBalancesParams {
  address: string
  cursor: string
  limit: usize
}

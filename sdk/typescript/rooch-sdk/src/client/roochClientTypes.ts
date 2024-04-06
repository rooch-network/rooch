// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { RoochMultiChainID } from '../constants'

import {
  Arg,
  FunctionId,
  IndexerStateID,
  InscriptionFilterView,
  RoochAccountAddress,
  TransactionFilterView,
  ObjectStateFilterView,
  FieldStateFilterView,
  TypeTag,
  u64,
  usize,
  UTXOFilterView,
} from '../types'

import { RoochTransactionData } from '../generated/runtime/rooch_types/mod'

import { IAuthorizer } from '../auth'

export type SendRawTransactionOpts = {
  maxGasAmount?: number
}

export type SendTransactionParams = {
  address: string
  authorizer: IAuthorizer
  funcId: FunctionId
  args?: Arg[]
  tyArgs?: TypeTag[]
  opts?: SendRawTransactionOpts
}

export type SendTransactionDataParams = {
  authorizer: IAuthorizer
  data: RoochTransactionData
}

export type SendRawTransactionParams =
  | SendTransactionParams
  | SendTransactionDataParams
  | Uint8Array

export type ExecuteViewFunctionParams = {
  funcId: FunctionId
  tyArgs?: TypeTag[]
  args?: Arg[]
}

export type ResoleRoochAddressParams = {
  address: string
  multiChainID: RoochMultiChainID
}

export type ListStatesParams = {
  accessPath: string
  cursor: string | null
  limit: number
}

export type QueryObjectStatesParams = {
  filter: ObjectStateFilterView
  cursor: IndexerStateID | null
  limit: number
  descending_order: boolean
}

export type QueryFieldStatesParams = {
  filter: FieldStateFilterView
  cursor: IndexerStateID | null
  limit: number
  descending_order: boolean
}

export type QueryInscriptionsParams = {
  filter?: InscriptionFilterView | null
  cursor?: IndexerStateID | null
  limit: number
  descending_order: boolean
}

export type QueryUTXOsParams = {
  filter?: UTXOFilterView | null
  cursor?: IndexerStateID | null
  limit: number
  descending_order: boolean
}

export type GetTransactionsParams = {
  cursor: number
  limit: number
  descending_order: boolean
}

export type GetEventsParams = {
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

export type QueryTransactionParams = {
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

export type QueryEventParams = {
  filter: QueryEventFilterParams
  cursor: { event_index: number; tx_order: number }
  limit: usize
  descending_order: boolean
}

export type GetBalanceParams = {
  address: string
  coinType: string
}

export type GetBalancesParams = {
  address: string
  cursor: string
  limit: string
}

export type SessionInfo = {
  authentication_key: string
  scopes: Array<string>
  create_time: number
  last_active_time: number
  max_inactive_interval: number
}

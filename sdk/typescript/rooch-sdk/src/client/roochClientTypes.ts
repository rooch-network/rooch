// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { RoochMultiChainID } from '../constants'

import {
  Arg,
  FunctionId,
  IndexerStateID,
  InscriptionFilterView,
  ObjectStateFilterView,
  FieldStateFilterView,
  TypeTag,
  UTXOFilterView,
} from '../types'

import { RoochTransactionData } from '../generated/runtime/rooch_types/mod'

import { IAuthorizer } from '../auth'

export const DEFAULT_LIMIT = '10'
export const DEFAULT_NULL_CURSOR = null as any

export const DEFAULT_DISPLAY = {
  decode: true,
  showDisplay: true,
}

export type DisplayOpts = {
  decode: boolean
  showDisplay: boolean
}

export type PagesOpts<T> = {
  cursor?: T | null
  limit?: number
  descending_order?: boolean
}

export type TransactionDataParams = {
  authorizer: IAuthorizer
  data: RoochTransactionData
}

export type SendTransactionOpts = {
  maxGasAmount?: number
}

export type SendTransactionInfoParams = ExecuteViewFunctionParams & {
  address: string
  authorizer: IAuthorizer
  opts?: SendTransactionOpts
}

export type SendTransactionParams = SendTransactionInfoParams | TransactionDataParams | Uint8Array

export type ExecuteTransactionOpts = SendTransactionOpts & {
  withOutput?: boolean
}

export type ExecuteTransactionInfoParams = SendTransactionInfoParams & {
  opts?: ExecuteTransactionOpts
}

export type ExecuteTransactionParams =
  | ExecuteTransactionInfoParams
  | TransactionDataParams
  | Uint8Array

// export type ExecuteTransactionParams = ExecuteTransactionParams | TransactionDataParams | Uint8Array

export type ExecuteViewFunctionParams = {
  funcId: FunctionId
  tyArgs?: TypeTag[]
  args?: Arg[]
}

export type ResoleRoochAddressParams = {
  address: string
  multiChainID: RoochMultiChainID
}

export type GetStatesParams = {
  accessPath: string
  display?: DisplayOpts
}

export type ListStatesParams = PagesOpts<string> & {
  accessPath: string
  display?: DisplayOpts
}

export type QueryObjectStatesParams = PagesOpts<IndexerStateID> & {
  filter: ObjectStateFilterView
  showDisplay?: boolean
}

export type QueryFieldStatesParams = PagesOpts<IndexerStateID> & {
  filter: FieldStateFilterView
  showDisplay?: boolean
}

export type QueryInscriptionsParams = PagesOpts<IndexerStateID> & {
  filter: InscriptionFilterView
}

export type QueryUTXOsParams = PagesOpts<IndexerStateID> & {
  filter: UTXOFilterView
}

export type GetTransactionsParams = PagesOpts<number>

export type GetEventsParams = PagesOpts<number> & {
  eventHandleType: string
}

export type QueryTransactionFilterParams =
  | { sender: string }
  | { original_address: string }
  | { tx_hashes: string[] }
  | { time_range: { end_time: number; start_time: number } }
  | { tx_order_range: { from_order: number; to_order: number } }

export type QueryTransactionParams = PagesOpts<number> & {
  filter: QueryTransactionFilterParams
  showDisplay?: boolean
}

export type QueryEventFilterParams =
  | { event_type: string }
  | { sender: string }
  | { tx_hash: string }
  | { time_range: { end_time: number; start_time: number } }
  | { tx_order_range: { from_order: number; to_order: number } }

export type QueryEventParams = PagesOpts<string> & {
  filter: QueryEventFilterParams
  showDisplay?: boolean
}

export type GetBalanceParams = {
  address: string
  coinType: string
}

export type GetBalancesParams = PagesOpts<string> & {
  address: string
}

export type QuerySessionKeysParams = PagesOpts<string> & {
  address: string
}

export type SessionInfoResult = {
  appName: string
  appUrl: string
  authenticationKey: string
  scopes: Array<string>
  createTime: number
  lastActiveTime: number
  maxInactiveInterval: number
}

// export type MoveAbort = {
//   abort_code: u64
//   location: string
//   type: string
// }
//
// export type ExecutionFailure = {
//   code_offset: number
//   function: number
//   location: string
//   type: string
// }
//
// export type TransactionExecutionResult = TransactionExecutionInfoView & {
//   status: 'executed' | 'outOfGas' | 'MiscellaneousError' | MoveAbort | ExecutionFailure
// }
//
// export type ExecuteTransactionResult = {
//   execution_info: TransactionExecutionResult
//   output: TransactionOutputView | null
//   sequence_info: TransactionSequenceInfoView
// }

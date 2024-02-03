// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { RoochMultiChainID } from '../constants'
import {
  Arg,
  FunctionId,
  GlobalStateFilterView,
  IndexerStateID,
  InscriptionFilterView,
  TableStateFilterView,
  TypeTag,
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

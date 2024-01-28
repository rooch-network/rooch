// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
import { AnnotatedFunctionResultView, StatePageView, StateView, Bytes } from '../types'

import { ExecuteViewFunctionParams, ListStatesParams } from './types.ts'

export interface IClient {
  getRpcApiVersion(): Promise<string | undefined>

  getChainId(): number

  executeViewFunction(params: ExecuteViewFunctionParams): Promise<AnnotatedFunctionResultView>

  sendRawTransaction(playload: Bytes): Promise<string>

  getStates(accessPath: string): Promise<StateView | null[]>

  listStates(listStatesParams: ListStatesParams): Promise<StatePageView>
}

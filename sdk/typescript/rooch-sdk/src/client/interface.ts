// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
import {
  Arg,
  TypeTag,
  FunctionId,
  AnnotatedFunctionResultView,
  StatePageView,
  StateView,
  Bytes,
  SimpleKeyStateView,
} from '../types'

export interface IClient {
  getRpcApiVersion(): Promise<string | undefined>

  getChainId(): number

  executeViewFunction(
    funcId: FunctionId,
    tyArgs?: TypeTag[],
    args?: Arg[],
  ): Promise<AnnotatedFunctionResultView>

  sendRawTransaction(playload: Bytes): Promise<string>

  getStates(accessPath: string): Promise<StateView | null[]>

  listStates(
    access_path: string,
    cursor: SimpleKeyStateView | null,
    limit: number,
  ): Promise<StatePageView>
}

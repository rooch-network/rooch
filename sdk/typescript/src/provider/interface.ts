// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
import {
  Arg,
  TypeTag,
  FunctionId,
  AnnotatedFunctionResultView,
  ListAnnotatedStateResultPageView,
  AnnotatedStateView,
  Bytes,
} from '../types'

export interface IProvider {
  getRpcApiVersion(): Promise<string | undefined>

  getChainId(): number

  executeViewFunction(
    funcId: FunctionId,
    tyArgs?: TypeTag[],
    args?: Arg[],
  ): Promise<AnnotatedFunctionResultView>

  sendRawTransaction(playload: Bytes): Promise<string>

  getAnnotatedStates(accessPath: string): Promise<AnnotatedStateView | null[]>

  listAnnotatedStates(
    access_path: string,
    cursor: Bytes | null,
    limit: number,
  ): Promise<ListAnnotatedStateResultPageView>
}

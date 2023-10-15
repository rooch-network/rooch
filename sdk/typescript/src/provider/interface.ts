// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
import {
  Arg,
  TypeTag,
  FunctionId,
  AnnotatedFunctionViewResult,
  Bytes,
  AnnotatedStateViewResult,
  AnnotatedStatePageViewResult,
} from '../types'

export interface IProvider {
  getRpcApiVersion(): Promise<string | undefined>

  getChainId(): number

  executeViewFunction(
    funcId: FunctionId,
    tyArgs?: TypeTag[],
    args?: Arg[],
  ): Promise<AnnotatedFunctionViewResult>

  sendRawTransaction(playload: Bytes): Promise<string>

  getAnnotatedStates(accessPath: string): Promise<AnnotatedStateViewResult | null[]>

  listAnnotatedStates(
    access_path: string,
    cursor: Bytes | null,
    limit: number,
  ): Promise<AnnotatedStatePageViewResult>
}

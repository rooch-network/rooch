// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
import { FunctionId, TypeTag, Arg } from '../types'

export interface IAccount {
  callFunction(
    funcId: FunctionId,
    tyArgs?: TypeTag[],
    args?: Arg[],
  ): Promise<string>
}

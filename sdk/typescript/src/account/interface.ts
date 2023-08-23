// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
import { FunctionId, TypeTag, Arg } from '../types'

export interface CallOption {
  maxGasAmount?: number
}

export interface IAccount {
  callFunction(
    funcId: FunctionId,
    tyArgs?: TypeTag[],
    args?: Arg[],
    opts?: CallOption,
  ): Promise<string>
}

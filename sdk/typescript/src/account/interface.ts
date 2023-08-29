// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { FunctionId, TypeTag, Arg } from '../types'

export interface CallOption {
  maxGasAmount?: number
}

export interface IAccount {
  runFunction(
    funcId: FunctionId,
    tyArgs?: TypeTag[],
    args?: Arg[],
    opts?: CallOption,
  ): Promise<string>

  /**
   * createSessionAccount
   *
   * create a sub account with session key
   *
   * @param scope
   */
  createSessionAccount(scope: string): Promise<IAccount>
}

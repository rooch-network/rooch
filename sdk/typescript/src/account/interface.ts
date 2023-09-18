// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { FunctionId, TypeTag, Arg } from '../types'

export interface CallOption {
  maxGasAmount?: number
}

export interface IAccount {
  /**
   * Run move function by current account
   *
   * @param funcId FunctionId the function like '0x3::empty::empty'
   * @param tyArgs Generic parameter list
   * @param args parameter list
   * @param opts Call option
   */
  runFunction(
    funcId: FunctionId,
    tyArgs?: TypeTag[],
    args?: Arg[],
    opts?: CallOption,
  ): Promise<string>

  /**
   * Create session account with scope
   *
   * @param scope string the scope of created account
   * @param maxInactiveInterval  number The max inactive interval
   * @param opts CallOption
   */
  createSessionAccount(
    scope: Array<string>,
    maxInactiveInterval: number,
    opts?: CallOption,
  ): Promise<IAccount>
}

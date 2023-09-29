// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { AccountAddress, FunctionId, TypeTag, Arg, Bytes, IPage } from '../types'

export interface CallOption {
  maxGasAmount?: number
}

export interface ISessionKey {
  authentication_key: string
  scopes: Array<string>
  create_time: number
  last_active_time: number
  max_inactive_interval: number
}

export interface IAccount {
  /**
   * Get account address
   */
  getAddress(): string

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

  /**
   * Create session key
   *
   * @param authKey
   * @param scopes
   * @param maxInactiveInterval
   * @param opts
   */
  registerSessionKey(
    authKey: AccountAddress,
    scopes: Array<string>,
    maxInactiveInterval: number,
    opts?: CallOption,
  ): Promise<void>

  /**
   * Remove session key
   *
   * @param authKey
   * @param scopes
   * @param maxInactiveInterval
   * @param opts
   */
  removeSessionKey(authKey: AccountAddress, opts?: CallOption): Promise<string>

  /**
   * Query account's sessionKey
   *
   * @param cursor The page cursor
   * @param limit The page limit
   */
  querySessionKeys(cursor: Bytes | null, limit: number): Promise<IPage<ISessionKey>>
}

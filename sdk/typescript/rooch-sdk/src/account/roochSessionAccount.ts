// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { FunctionId, TypeTag, Arg, IPage } from '../types'
import { addressToListTuple, addressToSeqNumber, encodeArg, encodeFunctionCall } from '../utils'

import { RoochAccount } from './roochAccount'
import { RoochClient } from '../client/roochClient'
import { SendTransactionOpts, SessionInfoResult } from '../client/roochClientTypes'
import { IAccount } from '../account/interface.ts'
import { IAuthorizer } from '../auth'
import {
  RoochTransactionData,
  AccountAddress as BCSAccountAddress,
} from '../generated/runtime/rooch_types/mod'
import { DEFAULT_MAX_GAS_AMOUNT } from '../constants'

const SCOPE_LENGTH = 3
const SCOPE_MODULE_ADDRESSS = 0
const SCOPE_MODULE_NAMES = 1
const SCOPE_FUNCTION_NAMES = 2

const requiredScope = '0x3::session_key::remove_session_key_entry'

export class RoochSessionAccount implements IAccount {
  protected readonly client: RoochClient
  protected readonly scopes: string[]
  protected readonly maxInactiveInterval: number
  protected readonly account?: IAccount
  protected readonly sessionAccount: RoochAccount
  protected readonly localCreateSessionTime: number
  protected readonly appName: string
  protected readonly appUrl: string
  protected constructor(
    client: RoochClient,
    appName: string,
    appUrl: string,
    scopes: string[],
    maxInactiveInterval: number,
    account?: IAccount,
    authInfo?: string,
    sessionAccount?: RoochAccount,
    localCreateSessionTime?: number,
  ) {
    this.client = client
    this.appName = appName
    this.appUrl = appUrl
    this.scopes = scopes
    this.maxInactiveInterval = maxInactiveInterval
    this.account = account
    this.authInfo = authInfo
    this.sessionAccount = sessionAccount || new RoochAccount(this.client)
    this.localCreateSessionTime = localCreateSessionTime ?? Date.now() / 1000

    // session must have the right to delete itself
    if (!this.scopes.find((item) => item === '0x3::*::*' || item === requiredScope)) {
      this.scopes.push(requiredScope)
    }
  }

  protected readonly authInfo?: string

  public static async CREATE(
    client: RoochClient,
    account: IAccount,
    appName: string,
    appUrl: string,
    scopes: string[],
    maxInactiveInterval: number,
    opts?: SendTransactionOpts,
  ): Promise<RoochSessionAccount> {
    return new RoochSessionAccount(
      client,
      appName,
      appUrl,
      scopes,
      maxInactiveInterval,
      account,
    ).build(opts)
  }

  public toJSON(): any {
    return {
      account: this.account,
      session: this.sessionAccount,
      scopes: this.scopes,
      maxInactiveInterval: this.maxInactiveInterval,
      localCreateSessionTime: this.localCreateSessionTime,
      authInfo: this.authInfo,
      appName: this.appName,
      appUrl: this.appUrl,
    }
  }

  public static formJson(jsonObj: any, client: RoochClient) {
    const {
      account,
      session,
      scopes,
      maxInactiveInterval,
      authInfo,
      localCreateSessionTime,
      appName,
      appUrl,
    } = jsonObj

    const roochAccount = RoochAccount.formJson(account, client)
    const sessionAccount = RoochAccount.formJson(session, client)

    return new RoochSessionAccount(
      client,
      appName,
      appUrl,
      scopes,
      maxInactiveInterval,
      roochAccount,
      authInfo,
      sessionAccount,
      localCreateSessionTime,
    )
  }

  public getAuthKey(): string {
    return this.sessionAccount.getAddress()
  }

  getAddress(): string {
    return this.account!.getAddress()
  }

  getAuthorizer(): IAuthorizer {
    return this.sessionAccount.getAuthorizer()
  }

  async sendTransaction(
    funcId: FunctionId,
    args?: Arg[],
    tyArgs?: TypeTag[],
    opts?: SendTransactionOpts,
  ) {
    return this.client.sendRawTransaction({
      address: this.getAddress(),
      authorizer: this.getAuthorizer(),
      funcId,
      args,
      tyArgs,
      opts,
    })
  }

  async executeTransaction(
    funcId: FunctionId,
    args?: Arg[],
    tyArgs?: TypeTag[],
    opts?: SendTransactionOpts,
  ) {
    return this.client.executeTransaction({
      address: this.getAddress(),
      authorizer: this.getAuthorizer(),
      funcId,
      args,
      tyArgs,
      opts,
    })
  }

  public async isExpired(): Promise<boolean> {
    // if (this.localCreateSessionTime + this.maxInactiveInterval > Date.now() / 1000) {
    //   return Promise.resolve(true)
    // }

    return this.client.sessionIsExpired(this.getAddress(), this.getAuthKey())
  }

  public async getSessionKey() {
    const result = await this.client.executeViewFunction({
      funcId: '0x3::session_key::get_session_key',
      tyArgs: [],
      args: [
        {
          type: 'Address',
          value: this.getAddress(),
        },
        {
          type: { Vector: 'U8' },
          value: addressToSeqNumber(this.getAuthKey()),
        },
      ],
    })

    if ((result.return_values![0].value.value as any) === '0x00') {
      return null
    }

    const parseScopes = (data: Array<any>) => {
      const result = new Array<string>()

      for (const scope of data) {
        const value = scope.value
        result.push(`${value.module_name}::${value.module_address}::${value.function_name}`)
      }

      return result
    }

    const val = (result.return_values![0].decoded_value as any).value.vec[0].value as any

    return {
      appName: val.app_name,
      appUrl: val.app_url,
      authenticationKey: val.authentication_key,
      scopes: parseScopes(val.scopes),
      createTime: parseInt(val.create_time),
      lastActiveTime: parseInt(val.last_active_time),
      maxInactiveInterval: parseInt(val.max_inactive_interval),
    } as SessionInfoResult
  }

  public async querySessionKeys(
    cursor?: string,
    limit?: number,
  ): Promise<IPage<SessionInfoResult, string>> {
    return this.client.querySessionKeys({
      address: this.getAddress(),
      cursor,
      limit,
    })
  }

  public async destroy(opts?: SendTransactionOpts) {
    await this.client.executeTransaction({
      funcId: '0x3::session_key::remove_session_key_entry',
      args: [
        {
          type: { Vector: 'U8' },
          value: addressToSeqNumber(this.getAuthKey()),
        },
      ],
      tyArgs: [],
      address: this.getAddress(),
      authorizer: this.getAuthorizer(),
      opts: opts,
    })
  }

  protected async build(opts?: SendTransactionOpts): Promise<RoochSessionAccount> {
    const [scopeModuleAddresss, scopeModuleNames, scopeFunctionNames] = this.scopes
      .map((scope: string) => {
        const parts = scope.split('::')
        if (parts.length !== SCOPE_LENGTH) {
          throw new Error('invalid scope')
        }

        const scopeModuleAddress = parts[SCOPE_MODULE_NAMES]
        const scopeModuleName = parts[SCOPE_MODULE_ADDRESSS]
        const scopeFunctionName = parts[SCOPE_FUNCTION_NAMES]
        return [scopeModuleAddress, scopeModuleName, scopeFunctionName]
      })
      .reduce(
        (acc: Array<Array<string>>, val: Array<string>) => {
          acc[0].push(val[SCOPE_MODULE_NAMES])
          acc[1].push(val[SCOPE_MODULE_ADDRESSS])
          acc[2].push(val[SCOPE_FUNCTION_NAMES])
          return acc
        },
        [[], [], []],
      )

    const args: Arg[] = [
      {
        type: 'Ascii',
        value: this.appName,
      },
      {
        type: 'Ascii',
        value: this.appUrl,
      },
      {
        type: { Vector: 'U8' },
        value: addressToSeqNumber(this.getAuthKey()),
      },
      {
        type: { Vector: 'Address' },
        value: scopeModuleAddresss,
      },
      {
        type: { Vector: 'Ascii' },
        value: scopeModuleNames,
      },
      {
        type: { Vector: 'Ascii' },
        value: scopeFunctionNames,
      },
      {
        type: 'U64',
        value: BigInt(this.maxInactiveInterval),
      },
    ]
    const sequenceNumber = await this.client.getSequenceNumber(this.getAddress())
    const bcsArgs = args.map((arg) => encodeArg(arg))
    const scriptFunction = encodeFunctionCall(
      '0x3::session_key::create_session_key_with_multi_scope_entry',
      [],
      bcsArgs,
    )
    const txData = new RoochTransactionData(
      new BCSAccountAddress(addressToListTuple(this.getAddress())),
      BigInt(sequenceNumber),
      BigInt(this.client.getChainId()),
      BigInt(opts?.maxGasAmount ?? DEFAULT_MAX_GAS_AMOUNT),
      scriptFunction,
    )

    return this.register(txData)
  }

  protected async register(txData: RoochTransactionData): Promise<RoochSessionAccount> {
    const result = await this.client.executeTransaction({
      authorizer: this.account!.getAuthorizer(),
      data: txData,
    })

    if (result.execution_info.status.type !== 'executed') {
      console.log(result.execution_info.status)
      throw new Error('create session error')
    }

    return this
  }
}

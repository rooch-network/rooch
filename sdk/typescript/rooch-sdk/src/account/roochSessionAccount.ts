// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { FunctionId, TypeTag, Arg, IPage } from '../types'
import { addressToListTuple, addressToSeqNumber, encodeArg, encodeFunctionCall } from '../utils'

import { RoochAccount } from './roochAccount'
import { RoochClient } from '../client/roochClient'
import { SendRawTransactionOpts, SessionInfo } from '../client/roochClientTypes'
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
  protected constructor(
    client: RoochClient,
    scopes: string[],
    maxInactiveInterval: number,
    account?: IAccount,
    authInfo?: string,
    sessionAccount?: RoochAccount,
    localCreateSessionTime?: number,
  ) {
    this.client = client
    this.account = account
    this.scopes = scopes
    this.maxInactiveInterval = maxInactiveInterval
    this.localCreateSessionTime = localCreateSessionTime ?? Date.now() / 1000
    this.sessionAccount = sessionAccount ?? new RoochAccount(this.client)
    this.authInfo = authInfo

    // session must have the right to delete itself
    if (!this.scopes.find((item) => item === '0x3::*::*' || item === requiredScope)) {
      this.scopes.push(requiredScope)
    }
  }

  protected readonly authInfo?: string

  public static async CREATE(
    client: RoochClient,
    account: IAccount,
    scopes: string[],
    maxInactiveInterval: number,
    opts?: SendRawTransactionOpts,
  ): Promise<RoochSessionAccount> {
    return new RoochSessionAccount(client, scopes, maxInactiveInterval, account).build(opts)
  }

  public toJSON(): any {
    return {
      account: this.account,
      session: this.sessionAccount,
      scopes: this.scopes,
      maxInactiveInterval: this.maxInactiveInterval,
      localCreateSessionTime: this.localCreateSessionTime,
      authInfo: this.authInfo,
    }
  }

  public static formJson(jsonObj: any, client: RoochClient) {
    const { account, session, scopes, maxInactiveInterval, authInfo, localCreateSessionTime } =
      jsonObj

    const roochAccount = RoochAccount.formJson(account, client)
    const sessionAccount = RoochAccount.formJson(session, client)

    return new RoochSessionAccount(
      client,
      scopes,
      maxInactiveInterval,
      roochAccount,
      authInfo,
      sessionAccount,
      localCreateSessionTime,
    )
  }

  protected async build(opts?: SendRawTransactionOpts): Promise<RoochSessionAccount> {
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
    const s = await this.client.sendRawTransaction({
      authorizer: this.account!.getAuthorizer(),
      data: txData,
    })
    console.log(s)

    return this
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

  /**
   * Run move function by current account
   *
   * @param funcId FunctionId the function like '0x3::empty::empty'
   * @param tyArgs Generic parameter list
   * @param args parameter list
   * @param opts Call option
   */
  async sendTransaction(
    funcId: FunctionId,
    args?: Arg[],
    tyArgs?: TypeTag[],
    opts?: SendRawTransactionOpts,
  ): Promise<string> {
    return this.client.sendRawTransaction({
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
    return this.client.executeViewFunction({
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
  }

  public async querySessionKeys(
    cursor: string | null,
    limit: number,
  ): Promise<IPage<SessionInfo, string>> {
    return this.client.querySessionKeys(this.getAddress(), cursor, limit)
  }

  public async destroy(opts?: SendRawTransactionOpts): Promise<string> {
    return await this.client.sendRawTransaction({
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

  // public toJson(): string {
  //
  // }
  //
  // public fromJson(): RoochSessionAccount {
  //
  // }
}

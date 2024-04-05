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
  RoochTransaction,
} from '../generated/runtime/rooch_types/mod'
import { DEFAULT_MAX_GAS_AMOUNT } from '../constants'
import { BcsSerializer } from '../generated/runtime/bcs/bcsSerializer'

const SCOPE_LENGTH = 3
const SCOPE_MODULE_ADDRESSS = 0
const SCOPE_MODULE_NAMES = 1
const SCOPE_FUNCTION_NAMES = 2

export class RoochSessionAccount implements IAccount {
  protected readonly client: RoochClient
  protected readonly scopes: string[]
  protected readonly maxInactiveInterval: number
  protected readonly account: IAccount
  protected readonly sessionAccount: RoochAccount
  protected readonly localCreateSessionTime: number
  protected readonly authInfo?: string

  protected constructor(
    client: RoochClient,
    account: IAccount,
    scopes: string[],
    maxInactiveInterval: number,
    authInfo?: string,
  ) {
    this.client = client
    this.account = account
    this.scopes = scopes
    this.maxInactiveInterval = maxInactiveInterval
    this.localCreateSessionTime = Date.now() / 1000
    this.sessionAccount = new RoochAccount(this.client)
    this.authInfo = authInfo
  }

  public static async CREATE(
    client: RoochClient,
    account: IAccount,
    scopes: string[],
    maxInactiveInterval: number,
    opts?: SendRawTransactionOpts,
  ): Promise<RoochSessionAccount> {
    return new RoochSessionAccount(client, account, scopes, maxInactiveInterval).build(opts)
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
        value: addressToSeqNumber(await this.getAuthKey()),
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
    const number = await this.client.getSequenceNumber(await this.getRoochAddress())
    const bcsArgs = args.map((arg) => encodeArg(arg))
    const scriptFunction = encodeFunctionCall(
      '0x3::session_key::create_session_key_with_multi_scope_entry',
      [],
      bcsArgs,
    )
    const txData = new RoochTransactionData(
      new BCSAccountAddress(addressToListTuple(await this.getRoochAddress())),
      BigInt(number),
      BigInt(this.client.getChainId()),
      BigInt(opts?.maxGasAmount ?? DEFAULT_MAX_GAS_AMOUNT),
      scriptFunction,
    )

    return this.register(txData)
  }

  protected async register(txData: RoochTransactionData): Promise<RoochSessionAccount> {
    const transactionDataPayload = (() => {
      const se = new BcsSerializer()
      txData.serialize(se)
      return se.getBytes()
    })()

    const authResult = await this.account.getAuthorizer().auth(transactionDataPayload)
    const transaction = new RoochTransaction(txData, authResult)

    const transactionPayload = (() => {
      const se = new BcsSerializer()
      transaction.serialize(se)
      return se.getBytes()
    })()

    const s = await this.client.sendRawTransaction(transactionPayload)
    console.log(s)

    return this
  }

  public getAuthKey(): Promise<string> {
    return this.sessionAccount.getRoochAddress()
  }

  getAddress(): string | undefined {
    return this.account.getAddress()
  }

  getRoochAddress(): Promise<string> {
    return this.account.getRoochAddress()
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
      address: await this.account.getRoochAddress(),
      authorizer: this.sessionAccount.getAuthorizer(),
      funcId,
      args,
      tyArgs,
      opts,
    })
  }

  public async isExpired(): Promise<boolean> {
    if (this.localCreateSessionTime + this.maxInactiveInterval > Date.now() / 1000) {
      return Promise.resolve(true)
    }

    return this.client.sessionIsExpired(
      await this.account.getRoochAddress(),
      await this.getAuthKey(),
    )
  }

  public async querySessionKeys(
    cursor: string | null,
    limit: number,
  ): Promise<IPage<SessionInfo, string>> {
    return this.client.querySessionKeys(await this.getRoochAddress(), cursor, limit)
  }

  public async destroy(opts?: SendRawTransactionOpts): Promise<string> {
    return await this.sendTransaction(
      '0x3::session_key::remove_session_key_entry',
      [
        {
          type: { Vector: 'U8' },
          value: addressToSeqNumber(await this.getAuthKey()),
        },
      ],
      [],
      opts || {
        maxGasAmount: 100000000,
      },
    )
  }
}

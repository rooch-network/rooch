// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { DEFAULT_MAX_GAS_AMOUNT } from '../constants'
import { IAccount, CallOption, ISessionKey } from './interface'
import { IClient } from '../client'
import { IAuthorizer, IAuthorization, PrivateKeyAuth } from '../auth'
import { AccountAddress, FunctionId, TypeTag, Arg, StatePageView, Bytes, IPage } from '../types'
import { BcsSerializer } from '../types/bcs'
import {
  RoochTransaction,
  RoochTransactionData,
  AccountAddress as BCSAccountAddress,
  Authenticator,
} from '../generated/runtime/rooch_types/mod'
import {
  encodeArg,
  encodeFunctionCall,
  addressToListTuple,
  uint8Array2SeqNumber,
  addressToSeqNumber,
  encodeStructTypeTag,
} from '../utils'
import { Ed25519Keypair } from '../utils/keypairs'

const SCOPE_LENGTH = 3
const SCOPE_MODULE_ADDRESSS = 0
const SCOPE_MODULE_NAMES = 1
const SCOPE_FUNCTION_NAMES = 2

/**
 * Rooch Account
 * all write calls in here
 */
export class Account implements IAccount {
  private readonly client: IClient
  private readonly address: AccountAddress
  private readonly authorizer: IAuthorizer

  public constructor(client: IClient, address: AccountAddress, authorizer: IAuthorizer) {
    this.client = client
    this.address = address
    this.authorizer = authorizer
  }

  private async makeAuth(tsData: RoochTransactionData): Promise<IAuthorization> {
    const payload = (() => {
      const se = new BcsSerializer()
      tsData.serialize(se)
      return se.getBytes()
    })()

    return this.authorizer.auth(payload)
  }

  private parseStateToSessionKey(data: StatePageView): Array<ISessionKey> {
    const result = new Array<ISessionKey>()

    for (const state of data.data as any) {
      const moveValue = state?.state.decoded_value as any

      if (moveValue) {
        const val = moveValue.value

        result.push({
          authentication_key: val.authentication_key,
          scopes: this.parseScopes(val.scopes),
          create_time: parseInt(val.create_time),
          last_active_time: parseInt(val.last_active_time),
          max_inactive_interval: parseInt(val.max_inactive_interval),
        } as ISessionKey)
      }
    }

    return result
  }

  private parseScopes(data: Array<any>): Array<string> {
    const result = new Array<string>()

    for (const scope of data) {
      result.push(`${scope.module_name}::${scope.module_address}::${scope.function_name}`)
    }

    return result
  }

  private async getSequenceNumber(): Promise<number> {
    const resp = await this.client.executeViewFunction({
      funcId: '0x3::account::sequence_number',
      tyArgs: [],
      args: [
        {
          type: 'Address',
          value: this.address,
        },
      ],
    })

    if (resp && resp.return_values) {
      return resp.return_values[0].decoded_value as number
    }

    return 0
  }

  /**
   * Get account address
   */
  public getAddress(): string {
    return this.address
  }

  /**
   * Run move function by current account
   *
   * @param funcId FunctionId the function like '0x3::empty::empty'
   * @param tyArgs Generic parameter list
   * @param args parameter list
   * @param opts Call option
   */
  public async runFunction(
    funcId: FunctionId,
    tyArgs: TypeTag[],
    args: Arg[],
    opts: CallOption,
  ): Promise<string> {
    const number = await this.getSequenceNumber()
    const bcsArgs = args.map((arg) => encodeArg(arg))
    const scriptFunction = encodeFunctionCall(funcId, tyArgs, bcsArgs)
    const txData = new RoochTransactionData(
      new BCSAccountAddress(addressToListTuple(this.address)),
      BigInt(number),
      BigInt(this.client.getChainId()),
      BigInt(opts.maxGasAmount ?? DEFAULT_MAX_GAS_AMOUNT),
      scriptFunction,
    )

    const authResult = await this.makeAuth(txData)

    const auth = new Authenticator(
      BigInt(authResult.scheme),
      uint8Array2SeqNumber(authResult.payload),
    )
    const ts = new RoochTransaction(txData, auth)

    const payload = (() => {
      const se = new BcsSerializer()
      ts.serialize(se)
      return se.getBytes()
    })()

    return this.client.sendRawTransaction(payload)
  }

  public async createSessionAccount(
    scope: Array<string>,
    maxInactiveInterval: number,
    opts?: CallOption,
  ): Promise<IAccount> {
    const kp = Ed25519Keypair.generate()
    await this.registerSessionKey(
      kp.getPublicKey().toRoochAddress(),
      scope,
      maxInactiveInterval,
      opts,
    )
    const auth = new PrivateKeyAuth(kp)
    return new Account(this.client, this.address, auth)
  }

  public async registerSessionKey(
    authKey: AccountAddress,
    scopes: Array<string>,
    maxInactiveInterval: number,
    opts?: CallOption,
  ): Promise<void> {
    const [scopeModuleAddresss, scopeModuleNames, scopeFunctionNames] = scopes
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

    await this.runFunction(
      '0x3::session_key::create_session_key_with_multi_scope_entry',
      [],
      [
        {
          type: { Vector: 'U8' },
          value: addressToSeqNumber(authKey),
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
          value: BigInt(maxInactiveInterval),
        },
      ],
      opts || {
        maxGasAmount: 100000000,
      },
    )
  }

  /**
   * Remove session key
   *
   * @param authKey
   * @param opts
   */
  public async removeSessionKey(authKey: AccountAddress, opts?: CallOption): Promise<string> {
    return await this.runFunction(
      '0x3::session_key::remove_session_key_entry',
      [],
      [
        {
          type: { Vector: 'U8' },
          value: addressToSeqNumber(authKey),
        },
      ],
      opts || {
        maxGasAmount: 100000000,
      },
    )
  }

  /**
   * Query account's sessionKey
   *
   * @param cursor The page cursor
   * @param limit The page limit
   */
  public async querySessionKeys(cursor: Bytes | null, limit: number): Promise<IPage<ISessionKey>> {
    const accessPath = `/resource/${this.address}/0x3::session_key::SessionKeys`
    const state = await this.client.getStates(accessPath)
    if (state) {
      const stateView = state as any
      const tableId = stateView[0].value

      const accessPath = `/table/${tableId}`
      const pageView = await this.client.listStates({
        accessPath,
        cursor,
        limit,
      })

      return {
        data: this.parseStateToSessionKey(pageView),
        nextCursor: pageView.next_cursor,
        hasNextPage: pageView.has_next_page,
      }
    }

    throw new Error('not found state')
  }

  /**
   * Check session key whether expired
   *
   * @param authKey the auth key
   */
  async isSessionKeyExpired(authKey: AccountAddress): Promise<boolean> {
    const result = await this.client.executeViewFunction({
      funcId: '0x3::session_key::is_expired_session_key',
      tyArgs: [],
      args: [
        {
          type: 'Address',
          value: this.address,
        },
        {
          type: { Vector: 'U8' },
          value: addressToSeqNumber(authKey),
        },
      ],
    })

    if (result && result.vm_status !== 'Executed') {
      throw new Error('view 0x3::session_key::is_expired_session_key fail')
    }

    return result.return_values![0].decoded_value as boolean
  }

  async gasCoinBalance(): Promise<bigint> {
    const result = await this.client.executeViewFunction({
      funcId: '0x3::gas_coin::balance',
      tyArgs: [],
      args: [
        {
          type: 'Address',
          value: this.getAddress(),
        },
      ],
    })

    if (result && result.vm_status !== 'Executed') {
      throw new Error('view 0x3::gas_coin::balance fail')
    }

    return BigInt(result.return_values![0].decoded_value as string)
  }

  async coinBalance(coinType: string): Promise<bigint> {
    const structType = encodeStructTypeTag(coinType)
    const result = await this.client.executeViewFunction({
      funcId: '0x3::account_coin_store::balance',
      tyArgs: [structType],
      args: [
        {
          type: 'Address',
          value: this.getAddress(),
        },
      ],
    })

    if (result && result.vm_status !== 'Executed') {
      throw new Error('view 0x3::account_coin_store::balance fail')
    }

    return BigInt(result.return_values![0].decoded_value as string)
  }
}

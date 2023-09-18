// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { DEFAULT_MAX_GAS_AMOUNT } from '../constants'
import { IAccount, CallOption } from './interface'
import { IProvider } from '../provider'
import { IAuthorizer, IAuthorization, PrivateKeyAuth } from '../auth'
import { AccountAddress, FunctionId, TypeTag, Arg } from '../types'
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
} from '../utils'
import { Ed25519Keypair } from '../utils/keypairs'

export class Account implements IAccount {
  private provider: IProvider

  private address: AccountAddress

  private authorizer: IAuthorizer

  public constructor(provider: IProvider, address: AccountAddress, authorizer: IAuthorizer) {
    this.provider = provider
    this.address = address
    this.authorizer = authorizer
  }

  public async runFunction(
    funcId: FunctionId,
    tyArgs: TypeTag[],
    args: Arg[],
    opts: CallOption,
  ): Promise<string> {
    const number = await this.sequenceNumber()
    const bcsArgs = args.map((arg) => encodeArg(arg))
    const scriptFunction = encodeFunctionCall(funcId, tyArgs, bcsArgs)
    const txData = new RoochTransactionData(
      new BCSAccountAddress(addressToListTuple(this.address)),
      BigInt(number),
      BigInt(this.provider.getChainId()),
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

    return this.provider.sendRawTransaction(payload)
  }

  private async makeAuth(tsData: RoochTransactionData): Promise<IAuthorization> {
    const payload = (() => {
      const se = new BcsSerializer()
      tsData.serialize(se)
      return se.getBytes()
    })()

    return this.authorizer.auth(payload)
  }

  async sequenceNumber(): Promise<number> {
    const resp = await this.provider.executeViewFunction(
      '0x3::account::sequence_number',
      [],
      [
        {
          type: 'Address',
          value: this.address,
        },
      ],
    )

    if (resp && resp.return_values) {
      return resp.return_values[0].move_value as number
    }

    return 0
  }

  async createSessionAccount(
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
    return new Account(this.provider, this.address, auth)
  }

  async registerSessionKey(
    authKey: AccountAddress,
    scopes: Array<string>,
    maxInactiveInterval: number,
    opts?: CallOption,
  ): Promise<void> {
    const [scopeModuleAddresss, scopeModuleNames, scopeFunctionNames] = scopes
      .map((scope: string) => {
        const parts = scope.split('::')
        if (parts.length !== 3) {
          throw new Error('invalid scope')
        }

        const scopeModuleAddress = parts[0]
        const scopeModuleName = parts[1]
        const scopeFunctionName = parts[2]
        return [scopeModuleAddress, scopeModuleName, scopeFunctionName]
      })
      .reduce(
        (acc: Array<Array<string>>, val: Array<string>) => {
          acc[0].push(val[0])
          acc[1].push(val[1])
          acc[2].push(val[2])
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
}

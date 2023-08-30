// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { DEFAULT_MAX_GAS_AMOUNT } from '../constants'
import { IAccount, CallOption } from './interface'
import { IProvider } from '../provider'
import { IAuthorizer, IAuthorization, PrivateKeyAuth } from '../auth'
import { AccountAddress, FunctionId, TypeTag, Arg } from '../types'
import { BcsSerializer } from '../generated/runtime/bcs/mod'
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
import { bigint } from 'superstruct'

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

  async createSessionAccount(scope: string): Promise<IAccount> {
    const kp = Ed25519Keypair.generate()
    await this.registerSessionKey(kp.getPublicKey().toRoochAddress(), scope)
    const auth = new PrivateKeyAuth(kp)
    return new Account(this.provider, this.address, auth)
  }

  private async registerSessionKey(authKey: AccountAddress, scope: string): Promise<void> {
    const parts = scope.split('::')
    if (parts.length !== 3) {
      throw new Error('invalid scope')
    }

    await this.runFunction(
      '0x3::session_key::create_session_key_entry',
      [],
      [
        {
          type: { Vector: 'U8' },
          value: addressToSeqNumber(authKey),
        },
        {
          type: { Vector: 'Address' },
          value: Array.from(parts[0]),
        },
        {
          type: { Vector: 'U8' },
          value: Array.from(parts[1]),
        },
        {
          type: { Vector: 'U8' },
          value: Array.from(parts[2]),
        },
        {
          type: { Vector: 'U64' },
          value: BigInt(3600),
        },
        {
          type: { Vector: 'U64' },
          value: BigInt(300),
        }
      ],
      {
        maxGasAmount: 1000000,
      },
    )
  }
}

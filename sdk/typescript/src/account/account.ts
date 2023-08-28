// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { DEFAULT_MAX_GAS_AMOUNT } from '../constants'
import { IAccount, CallOption } from './interface'
import { IProvider } from '../provider'
import { IAuthorizer, IAuthorization } from '../auth'
import { AccountAddress, FunctionId, TypeTag, Arg } from '../types'
import { BcsSerializer } from '../generated/runtime/bcs/mod'
import {
  RoochTransaction,
  RoochTransactionData,
  AccountAddress as BCSAccountAddress,
  Authenticator,
} from '../generated/runtime/rooch_types/mod'
import { encodeArgs, encodeFunctionCall, addressToListTuple, uint8Array2SeqNumber } from '../utils'

export class Account implements IAccount {
  private provider: IProvider

  private address: AccountAddress

  private authorizer: IAuthorizer

  private sequenceNumber: bigint

  public constructor(provider: IProvider, address: AccountAddress, authorizer: IAuthorizer) {
    this.provider = provider
    this.address = address
    this.authorizer = authorizer
    this.sequenceNumber = BigInt('0')
  }

  public async callFunction(
    funcId: FunctionId,
    tyArgs: TypeTag[],
    args: Arg[],
    opts: CallOption,
  ): Promise<string> {
    const bcsArgs = args.map((arg) => encodeArgs(arg))
    const scriptFunction = encodeFunctionCall(funcId, tyArgs, bcsArgs)
    const data = new RoochTransactionData(
      new BCSAccountAddress(addressToListTuple(this.address)),
      this.sequenceNumber,
      BigInt(this.provider.getChainId()),
      BigInt(opts.maxGasAmount ?? DEFAULT_MAX_GAS_AMOUNT),
      scriptFunction,
    )

    const authPayload = await this.makeAuth(data)
    const auth = new Authenticator(
      BigInt(authPayload.scheme),
      uint8Array2SeqNumber(authPayload.payload),
    )
    const ts = new RoochTransaction(data, auth)

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
}

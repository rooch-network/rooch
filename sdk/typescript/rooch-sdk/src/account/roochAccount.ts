// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { IAuthorizer, PrivateKeyAuth } from '../auth'
import { Ed25519Keypair } from '../utils/keypairs'
import { RoochClient } from '../client/roochClient'
import { Arg, BalanceInfoPageView, BalanceInfoView, FunctionId, TypeTag } from '../types'
import { SendRawTransactionOpts } from '../client/roochClientTypes'
import { IAccount } from '../account/interface'

/**
 * Rooch Account
 */
export class RoochAccount implements IAccount {
  private readonly keypair: Ed25519Keypair
  private readonly client: RoochClient
  private address?: string
  private authorizer?: IAuthorizer

  public constructor(client: RoochClient, keyPair?: Ed25519Keypair) {
    this.client = client
    this.keypair = keyPair ?? Ed25519Keypair.generate()
  }

  public toJSON(): any {
    return {
      pk: this.keypair.export(),
    }
  }

  public static formJson(jsonObj: any, client: RoochClient) {
    return new RoochAccount(client, Ed25519Keypair.fromSecretKeyStr(jsonObj.pk.privateKey)!)
  }

  public getKeypar(): Ed25519Keypair {
    return this.keypair
  }

  public getAddress(): string {
    if (!this.address) {
      this.address = this.keypair.getPublicKey().toRoochAddress()
    }

    return this.address
  }

  public getAuthorizer(): IAuthorizer {
    if (!this.authorizer) {
      this.authorizer = new PrivateKeyAuth(this.keypair)
    }

    return this.authorizer
  }

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

  async getBalance(coinType: string): Promise<BalanceInfoView> {
    return this.client.getBalance({
      address: this.getAddress(),
      coinType,
    })
  }

  async getBalances(cursor: string, limit: string): Promise<BalanceInfoPageView> {
    return this.client.getBalances({
      address: this.getAddress(),
      cursor,
      limit,
    })
  }
}

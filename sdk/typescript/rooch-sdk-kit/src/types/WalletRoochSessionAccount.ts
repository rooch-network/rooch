// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import {
  IAccount,
  RoochAccount,
  RoochClient,
  RoochSessionAccount,
  SendRawTransactionOpts,
} from '@roochnetwork/rooch-sdk'
import { runtime, bcs } from '@roochnetwork/rooch-sdk'
import { WalletAccount } from '../types/WalletAccount'

export class WalletRoochSessionAccount extends RoochSessionAccount {
  private roochAddress?: string

  protected constructor(
    client: RoochClient,
    scopes: string[],
    maxInactiveInterval: number,
    account?: IAccount,
    authInfo?: string,
    sessionAccount?: RoochAccount,
    localCreateSessionTime?: number,
  ) {
    super(
      client,
      scopes,
      maxInactiveInterval,
      account,
      authInfo ??
        `Welcome to ${window.location.hostname}\nYou will authorize session:\n${
          'Scope:\n' +
          scopes
            .filter((v) => !v.startsWith('0x1') && !v.startsWith('0x3'))
            .map((v) => {
              console.log(v)
              return v
            }) +
          '\nTimeOut:' +
          maxInactiveInterval
        }`,
      sessionAccount,
      localCreateSessionTime,
    )
  }

  public static async CREATE(
    client: RoochClient,
    account: WalletAccount,
    scopes: string[],
    maxInactiveInterval: number,
    opts?: SendRawTransactionOpts,
  ): Promise<RoochSessionAccount> {
    return new WalletRoochSessionAccount(client, scopes, maxInactiveInterval, account).build(opts)
  }

  public static formJson(jsonObj: any, client: RoochClient) {
    const { session, scopes, maxInactiveInterval, authInfo, localCreateSessionTime, roochAddress } =
      jsonObj

    const sessionAccount = RoochAccount.formJson(session, client)

    const rsa = new WalletRoochSessionAccount(
      client,
      scopes,
      maxInactiveInterval,
      undefined,
      authInfo,
      sessionAccount,
      localCreateSessionTime,
    )

    rsa.roochAddress = roochAddress
    return rsa
  }

  toJSON(): any {
    return {
      roochAddress: this.roochAddress,
      session: this.sessionAccount,
      scopes: this.scopes,
      maxInactiveInterval: this.maxInactiveInterval,
      localCreateSessionTime: this.localCreateSessionTime,
      authInfo: this.authInfo,
    }
  }

  protected override async build(opts?: SendRawTransactionOpts): Promise<RoochSessionAccount> {
    this.roochAddress = await (this.account as WalletAccount).resoleRoochAddress()
    return super.build(opts)
  }

  getAddress(): string {
    return this.roochAddress!
  }

  protected override async register(
    txData: runtime.RoochTransactionData,
  ): Promise<RoochSessionAccount> {
    const transactionDataPayload = (() => {
      const se = new bcs.BcsSerializer()
      txData.serialize(se)
      return se.getBytes()
    })()

    const auth = await this.account!.getAuthorizer().auth(transactionDataPayload, this.authInfo)
    const transaction = new runtime.RoochTransaction(txData, auth)
    const transactionPayload = (() => {
      const se = new bcs.BcsSerializer()
      transaction.serialize(se)
      return se.getBytes()
    })()

    await this.client.sendRawTransaction(transactionPayload)

    return this
  }
}

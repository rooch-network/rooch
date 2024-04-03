// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { RoochClient, RoochSessionAccount, SendRawTransactionOpts } from '@roochnetwork/rooch-sdk'
import { runtime, bcs } from '@roochnetwork/rooch-sdk'
import { WalletAccount } from '../types/WalletAccount'

export class WalletRoochSessionAccount extends RoochSessionAccount {
  constructor(
    client: RoochClient,
    account: WalletAccount,
    scopes: string[],
    maxInactiveInterval: number,
    authInfo?: string,
  ) {
    super(
      client,
      account,
      scopes,
      maxInactiveInterval,
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
    )
  }

  public static async CREATE(
    client: RoochClient,
    account: WalletAccount,
    scopes: string[],
    maxInactiveInterval: number,
    opts?: SendRawTransactionOpts,
  ): Promise<RoochSessionAccount> {
    return new WalletRoochSessionAccount(client, account, scopes, maxInactiveInterval).build(opts)
  }

  override async register(txData: runtime.RoochTransactionData): Promise<RoochSessionAccount> {
    const transactionDataPayload = (() => {
      const se = new bcs.BcsSerializer()
      txData.serialize(se)
      return se.getBytes()
    })()

    const auth = await this.account.getAuthorizer().auth(transactionDataPayload, this.authInfo)
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

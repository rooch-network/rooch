// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { ETHWallet } from './ethWallet'
import { WalletAccount } from '../WalletAccount'
import { RoochClient } from '@roochnetwork/rooch-sdk'

export class Metamask extends ETHWallet {
  constructor(client: RoochClient) {
    super(client)
    this.name = 'metamask'
  }

  getTarget(): any {
    return (window as any).ethereum
  }

  getScheme(): number {
    return 1
  }

  async sign(msg: string): Promise<string> {
    return await this.getTarget().request({
      method: 'personal_sign',
      params: [msg, this.account?.address],
    })
  }

  async connect(): Promise<WalletAccount[]> {
    const accounts: string[] = await this.getTarget()
      .request({
        method: 'eth_requestAccounts',
      })
      .then((accounts: any) => {
        return accounts
      })

    return accounts.map((v) => new WalletAccount(this.getChain(), this, v, this.client))
  }
}

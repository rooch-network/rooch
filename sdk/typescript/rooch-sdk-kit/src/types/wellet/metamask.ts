// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { ETHWallet } from './ethWallet'
import { WalletAccount } from '../WalletAccount'
import { SupportChain } from '../../feature'

export class Metamask extends ETHWallet {

  getTarget(): any {
    return (window as any).ethereum
  }

  getScheme(): number {
    return 1
  }

  async sign(msg: string): Promise<string> {
    return await this.getTarget().request({
      method: 'personal_sign',
      params: [msg, this.account?.getAddress()],
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

    return accounts.map((v) => new WalletAccount(v, SupportChain.ETH))
  }
}

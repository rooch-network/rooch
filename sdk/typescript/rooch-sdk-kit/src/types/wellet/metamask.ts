// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { ETHWallet } from './ethWallet'
import { WalletAccount } from '../WalletAccount'
import { SupportWallet } from '../../feature'

export class Metamask extends ETHWallet {
  getTarget(): any {
    return (window as any).ethereum
  }

  getScheme(): number {
    return 1
  }

  async sign(msg: string, from: string): Promise<string> {
    return await this.getTarget().request({
      method: 'personal_sign',
      params: [msg, from],
    })
  }

  async connect(): Promise<WalletAccount[]> {
    // const chainId = (await window.ethereum?.request({ method: 'eth_chainId' })) as string

    // if (chainId !== chainInfo.chainId) {
    //   try {
    //     await this.switchChain({ ...chainInfo })
    //   } catch (e: any) {
    //     console.log('connect error', e.toString())
    //     return []
    //   }
    // }

    const accounts: string[] = await this.getTarget()
      .request({
        method: 'eth_requestAccounts',
      })
      .then((accounts: any) => {
        return accounts
      })

    return accounts.map((v) => new WalletAccount(v, SupportWallet.ETH))
  }
}

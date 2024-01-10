// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { WalletAccount } from '../WalletAccount'
import { BitcoinWallet } from './bitcoinWallet'
import { SupportWallet } from '../../feature'

export class UniSatWallet extends BitcoinWallet {
  async sign(msg: string, _: string): Promise<string> {
    return await this.getTarget().signMessage(msg)
  }

  getTarget(): any {
    return (window as any).unisat
  }

  getScheme(): number {
    return 2
  }

  async connect(): Promise<WalletAccount[]> {
    let accounts = await this.getTarget().getAccounts()

    if (!accounts || accounts.length === 0) {
      await this.getTarget().requestAccounts()
      return this.connect()
    }
    let publicKey = await this.getTarget().getPublicKey()

    return [new WalletAccount(accounts[0], SupportWallet.BITCOIN, publicKey)]
  }
}

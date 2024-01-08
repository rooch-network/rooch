// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { WalletAccount } from '../WalletAccount'
import { BitcoinWallet } from './bitcoinWallet'

export class UniSatWallet extends BitcoinWallet {
  async sign(msg: string, _: string): Promise<string> {
    return this.getTarget().signMessage(msg)
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

    return [new WalletAccount(accounts[0], publicKey)]
  }
}

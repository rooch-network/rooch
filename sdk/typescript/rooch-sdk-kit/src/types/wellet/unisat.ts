// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { BaseWallet } from './baseWallet'
import { WalletAccount } from '../WalletAccount'

export class UniSatWallet extends BaseWallet {
  async sign(msg: string): Promise<string> {
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

    if (!accounts) {
      await this.getTarget().requestAccounts()
      return this.connect()
    }
    let publicKey = await this.getTarget().getPublicKey()

    return [new WalletAccount(accounts[0], publicKey)]
  }
}

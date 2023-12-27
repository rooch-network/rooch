// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { BaseWallet } from './baseWallet'
import { WalletAccount } from '../WalletAccount'

export class OkxWallet extends BaseWallet {
  private address?: string

  async sign(msg: string): Promise<string> {
    return this.getTarget().signMessage(msg, {
      from: this.address,
    })
  }

  getScheme(): number {
    return 2
  }

  getTarget(): any {
    return (window as any).okxwallet.bitcoin
  }

  async connect(): Promise<WalletAccount[]> {
    const account = await this.getTarget().connect()
    this.address = account.address

    return [new WalletAccount(account.address, account.publicKey, account.compressedPublicKey)]
  }
}

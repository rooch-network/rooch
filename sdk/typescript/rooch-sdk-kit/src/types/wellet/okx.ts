// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { WalletAccount } from '../WalletAccount'
import { BitcoinWallet } from './bitcoinWallet'
import { SupportWallet } from '../../feature'

export class OkxWallet extends BitcoinWallet {
  async sign(msg: string, fromAddress: string): Promise<string> {
    return this.getTarget().signMessage(msg, {
      from: fromAddress,
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

    return [
      new WalletAccount(
        account.address,
        SupportWallet.ETH,
        account.publicKey,
        account.compressedPublicKey,
      ),
    ]
  }
}
